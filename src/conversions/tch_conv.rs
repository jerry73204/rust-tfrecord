#![cfg(feature = "with-tch")]

use super::*;
use image::{png::PngEncoder, ColorType};
use tch::{IndexOp, Kind, Tensor};

// macros

macro_rules! tensor_to_vec {
    ($tensor:ident, $ty:ident) => {
        unsafe {
            let numel = $tensor.numel();
            let mut data: Vec<$ty> = Vec::with_capacity(numel);
            let slice = slice::from_raw_parts_mut(data.as_mut_ptr(), numel);
            $tensor.copy_data(slice, numel);
            data.set_len(numel);
            data
        }
    };
}

macro_rules! tensor_to_r64_vec {
    ($tensor:ident, $ty:ident) => {{
        tensor_to_vec!($tensor, $ty)
            .into_iter()
            .map(|value| {
                let f64_value = value as f64;
                let r64_value = R64::try_new(f64_value).ok_or_else(|| Error::ConversionError {
                    desc: "non-finite floating point value found".into(),
                })?;
                Ok(r64_value)
            })
            .collect::<Result<Vec<_>, Error>>()
    }};
}

macro_rules! tensor_to_proto {
    ($tensor:ident, $ty:ident) => {{
        let values = tensor_to_vec!($tensor, $ty);
        let size = $tensor
            .size()
            .into_iter()
            .map(|sz| sz as usize)
            .collect::<Vec<_>>();
        TensorProtoInit { shape: Some(size) }.build_with_data(&values)
    }};
}

// auxiliary functions

fn normalized_tensor(tensor: &Tensor) -> Result<Tensor, Error> {
    let kind = tensor.f_kind()?;

    let normalized_tensor = match kind {
        Kind::Uint8 => tensor.shallow_clone(),
        Kind::Float | Kind::Double => {
            // determine the scale and offset by min/max values
            let valid_values_mask = tensor.f_isfinite()?;
            let valid_values = tensor.f_masked_select(&valid_values_mask)?;
            let min_value = f64::from(valid_values.f_min()?);
            let max_value = f64::from(valid_values.f_max()?);

            let (scale, offset) = if min_value >= 0.0 {
                let scale = 255.0 / max_value;
                let offset = 0.0;
                (scale, offset)
            } else {
                let scale = 127.0 / max_value.max(-min_value);
                let offset = 128.0;
                (scale, offset)
            };

            let normalized_tensor = tensor
                .f_mul1(scale)?
                .f_add1(offset)?
                .f_to_kind(Kind::Uint8)?;

            normalized_tensor
        }
        _ => {
            return Err(Error::ConversionError {
                desc: format!("the tensor with kind {:?} cannot converted to image", kind),
            });
        }
    };

    Ok(normalized_tensor)
}

fn guess_color_space_by_channels(channels: i64) -> Option<ColorSpace> {
    let color_space = match channels {
        1 => ColorSpace::Luma,
        3 => ColorSpace::Rgb,
        4 => ColorSpace::Rgba,
        _ => {
            return None;
        }
    };
    Some(color_space)
}

// to histogram

impl TryFrom<&Tensor> for HistogramProto {
    type Error = Error;

    fn try_from(from: &Tensor) -> Result<Self, Self::Error> {
        let kind = from.f_kind()?;
        let values = match kind {
            Kind::Uint8 => tensor_to_r64_vec!(from, u8)?,
            Kind::Int8 => tensor_to_r64_vec!(from, i8)?,
            Kind::Int16 => tensor_to_r64_vec!(from, i16)?,
            Kind::Int => tensor_to_r64_vec!(from, i32)?,
            Kind::Int64 => tensor_to_r64_vec!(from, i64)?,
            Kind::Float => tensor_to_r64_vec!(from, f32)?,
            Kind::Double => tensor_to_r64_vec!(from, f64)?,
            _ => {
                return Err(Error::ConversionError {
                    desc: format!("unsupported tensor kind {:?}", kind),
                })
            }
        };

        let histogram = Histogram::default();
        values.into_iter().for_each(|value| histogram.add(value));
        Ok(histogram.into())
    }
}

impl TryFrom<Tensor> for HistogramProto {
    type Error = Error;

    fn try_from(from: Tensor) -> Result<Self, Self::Error> {
        Self::try_from(&from)
    }
}

// to TensorProto

impl TryFrom<&Tensor> for TensorProto {
    type Error = Error;

    fn try_from(from: &Tensor) -> Result<Self, Self::Error> {
        // let size = from.size();
        let kind = from.f_kind()?;
        let proto = match kind {
            Kind::Uint8 => tensor_to_proto!(from, u8),
            Kind::Int8 => tensor_to_proto!(from, i8),
            Kind::Int16 => tensor_to_proto!(from, i16),
            Kind::Int => tensor_to_proto!(from, i32),
            Kind::Int64 => tensor_to_proto!(from, i64),
            Kind::Float => tensor_to_proto!(from, f32),
            Kind::Double => tensor_to_proto!(from, f64),
            _ => {
                return Err(Error::ConversionError {
                    desc: format!("unsupported tensor kind {:?}", kind),
                })
            }
        };

        Ok(proto)
    }
}

impl TryFrom<Tensor> for TensorProto {
    type Error = Error;
    fn try_from(from: Tensor) -> Result<Self, Self::Error> {
        Self::try_from(&from)
    }
}

// to Image
impl TryFrom<&Tensor> for Image {
    type Error = Error;

    fn try_from(from: &Tensor) -> Result<Self, Self::Error> {
        let size = from.size();

        // verify tensor shape
        let (_channels, height, width, color_space) = match size.as_slice() {
            &[channels, height, width] => {
                let color_space = guess_color_space_by_channels(channels).ok_or_else(|| Error::ConversionError { desc: format!("cannot convert tensor with shape {:?} to image, it must have 1, 3 or 4 channels", size)} )?;
                (channels, height, width, color_space)
            }
            _ => {
                return Err(Error::ConversionError {
                        desc: format!("cannot convert tensor with shape {:?} to image, the shape must have exactly 3 dimensions", size)
                    });
            }
        };

        // CHW to HWC
        let hwc_tensor = from
            .f_permute(&[1, 2, 0])
            .map_err(|err| Error::ConversionError {
                desc: format!("tch error: {:?}", err),
            })?;

        // normalize values to [0, 255]
        let normalized_tensor = normalized_tensor(&hwc_tensor)?;

        // encode image
        let encoded_image_string = {
            let samples = tensor_to_vec!(normalized_tensor, u8);
            let color_type = match color_space {
                ColorSpace::Luma => ColorType::L8,
                ColorSpace::Rgb => ColorType::Rgb8,
                ColorSpace::Rgba => ColorType::Rgba8,
                _ => unreachable!("please report bug"),
            };
            let mut cursor = Cursor::new(vec![]);
            PngEncoder::new(&mut cursor)
                .encode(&samples, width as u32, height as u32, color_type)
                .map_err(|err| Error::ConversionError {
                    desc: format!("{:?}", err),
                })?;
            cursor.into_inner()
        };

        Ok(Image {
            height: height as i32,
            width: width as i32,
            colorspace: color_space as i32,
            encoded_image_string,
        })
    }
}

impl TryFrom<Tensor> for Image {
    type Error = Error;
    fn try_from(from: Tensor) -> Result<Self, Self::Error> {
        Self::try_from(&from)
    }
}

// to Vec<Image>
impl TryInfoImageList for &Tensor {
    type Error = Error;

    fn try_into_image_list(self) -> Result<Vec<Image>, Self::Error> {
        let size = self.size();

        // verify tensor shape
        let batch_size = match size.as_slice() {
            &[batch_size, channels, _, _] => {
                if ![1, 3, 4].contains(&channels) {
                    return Err(Error::ConversionError {
                            desc: format!("cannot convert tensor with shape {:?} to list of images, the channel size must be one of 1, 3, 4", size)
                        });
                }
                batch_size
            }
            _ => {
                return Err(Error::ConversionError {
                        desc: format!("cannot convert tensor with shape {:?} to list of images, the shape must have exactly 4 dimensions", size)
                    });
            }
        };

        let images = (0..batch_size)
            .map(|batch_index| {
                let sub_tensor = self.i(batch_index);
                let image = Image::try_from(sub_tensor)?;
                Ok(image)
            })
            .collect::<Result<Vec<_>, Error>>()?;

        Ok(images)
    }
}

impl TryInfoImageList for Tensor {
    type Error = Error;
    fn try_into_image_list(self) -> Result<Vec<Image>, Self::Error> {
        TryInfoImageList::try_into_image_list(&self)
    }
}
