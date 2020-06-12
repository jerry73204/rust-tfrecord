#![cfg(feature = "with-image")]

use super::*;
use image::{
    flat::SampleLayout, png::PNGEncoder, ColorType, DynamicImage, FlatSamples, ImageBuffer, Pixel,
    Primitive,
};

// auxiliary types and traits

impl TryFrom<ColorType> for ColorSpace {
    type Error = Error;

    fn try_from(from: ColorType) -> Result<Self, Self::Error> {
        let color_space = match from {
            ColorType::L8 => ColorSpace::Luma,
            ColorType::La8 => ColorSpace::LumaA,
            ColorType::Rgb8 => ColorSpace::Rgb,
            ColorType::Rgba8 => ColorSpace::Rgba,
            ColorType::Bgra8 => ColorSpace::Bgra,
            ColorType::L16 => ColorSpace::Luma,
            ColorType::La16 => ColorSpace::LumaA,
            ColorType::Rgb16 => ColorSpace::Rgb,
            ColorType::Rgba16 => ColorSpace::Rgba,
            _ => {
                return Err(Error::ConversionError {
                    desc: "color space is not supported".into(),
                });
            }
        };
        Ok(color_space)
    }
}

// DynamicImage to histogram

impl TryFrom<&DynamicImage> for HistogramProto {
    type Error = Error;

    fn try_from(from: &DynamicImage) -> Result<Self, Self::Error> {
        use DynamicImage::*;
        let histogram = match from {
            ImageLuma8(buffer) => Self::try_from(buffer)?,
            ImageLumaA8(buffer) => Self::try_from(buffer)?,
            ImageRgb8(buffer) => Self::try_from(buffer)?,
            ImageRgba8(buffer) => Self::try_from(buffer)?,
            ImageBgr8(buffer) => Self::try_from(buffer)?,
            ImageBgra8(buffer) => Self::try_from(buffer)?,
            ImageLuma16(buffer) => Self::try_from(buffer)?,
            ImageLumaA16(buffer) => Self::try_from(buffer)?,
            ImageRgb16(buffer) => Self::try_from(buffer)?,
            ImageRgba16(buffer) => Self::try_from(buffer)?,
        };
        Ok(histogram)
    }
}

impl TryFrom<DynamicImage> for HistogramProto {
    type Error = Error;
    fn try_from(from: DynamicImage) -> Result<Self, Self::Error> {
        Self::try_from(&from)
    }
}

// ImageBuffer to histogram

impl<P, C> TryFrom<&ImageBuffer<P, C>> for HistogramProto
where
    P: 'static + Pixel,
    P::Subpixel: 'static,
    C: Deref<Target = [P::Subpixel]>,
    P::Subpixel: HistogramProtoElement,
{
    type Error = Error;

    fn try_from(from: &ImageBuffer<P, C>) -> Result<Self, Self::Error> {
        let components_iter = from
            .pixels()
            .flat_map(|pixel| pixel.channels().iter().cloned().collect::<Vec<_>>())
            .map(|component| {
                R64::try_new(component.to_f64()).ok_or_else(|| Error::ConversionError {
                    desc: "non-finite value found".into(),
                })
            });

        let histogram = Histogram::default();
        for result in components_iter {
            let component = result?;
            histogram.add(component);
        }

        Ok(histogram.into())
    }
}

impl<P, C> TryFrom<ImageBuffer<P, C>> for HistogramProto
where
    P: 'static + Pixel,
    P::Subpixel: 'static,
    C: Deref<Target = [P::Subpixel]>,
    P::Subpixel: HistogramProtoElement,
{
    type Error = Error;

    fn try_from(from: ImageBuffer<P, C>) -> Result<Self, Self::Error> {
        Self::try_from(&from)
    }
}

// FlatSamples to tensor

impl<T> TryFrom<&FlatSamples<&[T]>> for TensorProto
where
    T: TensorProtoElement,
{
    type Error = Error;

    fn try_from(from: &FlatSamples<&[T]>) -> Result<Self, Self::Error> {
        let FlatSamples {
            layout:
                SampleLayout {
                    width,
                    height,
                    channels,
                    ..
                },
            ..
        } = *from;
        let samples = (0..height)
            .flat_map(|y| (0..width).flat_map(move |x| (0..channels).map(move |c| (y, x, c))))
            .map(|(y, x, c)| *from.get_sample(c, x, y).unwrap())
            .collect::<Vec<_>>();
        let shape = vec![height as usize, width as usize, channels as usize];
        let proto = TensorProtoInit { shape: Some(shape) }.build_with_data(&samples);

        Ok(proto)
    }
}

impl<T> TryFrom<FlatSamples<&[T]>> for TensorProto
where
    T: TensorProtoElement,
{
    type Error = Error;

    fn try_from(from: FlatSamples<&[T]>) -> Result<Self, Self::Error> {
        Self::try_from(&from)
    }
}

impl<T> TryFrom<&FlatSamples<&Vec<T>>> for TensorProto
where
    T: TensorProtoElement,
{
    type Error = Error;

    fn try_from(from: &FlatSamples<&Vec<T>>) -> Result<Self, Self::Error> {
        Self::try_from(&from.as_ref())
    }
}

impl<T> TryFrom<FlatSamples<&Vec<T>>> for TensorProto
where
    T: TensorProtoElement,
{
    type Error = Error;

    fn try_from(from: FlatSamples<&Vec<T>>) -> Result<Self, Self::Error> {
        Self::try_from(&from)
    }
}

impl<T> TryFrom<&FlatSamples<Vec<T>>> for TensorProto
where
    T: TensorProtoElement,
{
    type Error = Error;

    fn try_from(from: &FlatSamples<Vec<T>>) -> Result<Self, Self::Error> {
        Self::try_from(&from.as_ref())
    }
}

impl<T> TryFrom<FlatSamples<Vec<T>>> for TensorProto
where
    T: TensorProtoElement,
{
    type Error = Error;

    fn try_from(from: FlatSamples<Vec<T>>) -> Result<Self, Self::Error> {
        Self::try_from(&from)
    }
}

// DynamicImage to tensor

impl TryFrom<&DynamicImage> for TensorProto {
    type Error = Error;

    fn try_from(from: &DynamicImage) -> Result<Self, Self::Error> {
        use DynamicImage::*;
        let tensor = match from {
            ImageLuma8(buffer) => Self::try_from(buffer)?,
            ImageLumaA8(buffer) => Self::try_from(buffer)?,
            ImageRgb8(buffer) => Self::try_from(buffer)?,
            ImageRgba8(buffer) => Self::try_from(buffer)?,
            ImageBgr8(buffer) => Self::try_from(buffer)?,
            ImageBgra8(buffer) => Self::try_from(buffer)?,
            ImageLuma16(buffer) => Self::try_from(buffer)?,
            ImageLumaA16(buffer) => Self::try_from(buffer)?,
            ImageRgb16(buffer) => Self::try_from(buffer)?,
            ImageRgba16(buffer) => Self::try_from(buffer)?,
        };
        Ok(tensor)
    }
}

impl TryFrom<DynamicImage> for TensorProto {
    type Error = Error;

    fn try_from(from: DynamicImage) -> Result<Self, Self::Error> {
        Self::try_from(&from)
    }
}

// ImageBuffer to tensor

impl<P, C, T> TryFrom<&ImageBuffer<P, C>> for TensorProto
where
    P: 'static + Pixel<Subpixel = T>,
    C: Deref<Target = [P::Subpixel]> + AsRef<[P::Subpixel]>,
    T: 'static + TensorProtoElement + Primitive,
{
    type Error = Error;

    fn try_from(from: &ImageBuffer<P, C>) -> Result<Self, Self::Error> {
        Self::try_from(from.as_flat_samples())
    }
}

impl<P, C, T> TryFrom<ImageBuffer<P, C>> for TensorProto
where
    P: 'static + Pixel<Subpixel = T>,
    C: Deref<Target = [P::Subpixel]> + AsRef<[P::Subpixel]>,
    T: 'static + TensorProtoElement + Primitive,
{
    type Error = Error;

    fn try_from(from: ImageBuffer<P, C>) -> Result<Self, Self::Error> {
        Self::try_from(&from)
    }
}

// DynamicImage to image

impl TryFrom<&DynamicImage> for Image {
    type Error = Error;

    fn try_from(from: &DynamicImage) -> Result<Self, Self::Error> {
        use DynamicImage::*;
        let image = match from {
            ImageLuma8(buffer) => Self::try_from(buffer)?,
            ImageLumaA8(buffer) => Self::try_from(buffer)?,
            ImageRgb8(buffer) => Self::try_from(buffer)?,
            ImageRgba8(buffer) => Self::try_from(buffer)?,
            ImageBgra8(buffer) => Self::try_from(buffer)?,
            _ => {
                return Err(Error::ConversionError {
                    desc: format!("unsupported image type"),
                });
            }
        };
        Ok(image)
    }
}

impl TryFrom<DynamicImage> for Image {
    type Error = Error;

    fn try_from(from: DynamicImage) -> Result<Self, Self::Error> {
        Self::try_from(&from)
    }
}

// FlatSamples to image

impl<B> TryFrom<&FlatSamples<B>> for Image
where
    B: AsRef<[u8]>,
{
    type Error = Error;

    fn try_from(from: &FlatSamples<B>) -> Result<Self, Self::Error> {
        let FlatSamples {
            layout:
                SampleLayout {
                    width,
                    height,
                    channels,
                    ..
                },
            color_hint,
            ..
        } = *from;
        let color_type = color_hint.ok_or_else(|| Error::ConversionError {
            desc: "color_hint must not be None".into(),
        })?;
        let colorspace = ColorSpace::try_from(color_type)?;
        let samples = (0..height)
            .flat_map(|y| (0..width).flat_map(move |x| (0..channels).map(move |c| (y, x, c))))
            .map(|(y, x, c)| *from.get_sample(c, x, y).unwrap())
            .collect::<Vec<_>>();

        let encoded_image_string = {
            let mut cursor = Cursor::new(vec![]);
            PNGEncoder::new(&mut cursor)
                .encode(&samples, width, height, color_type)
                .map_err(|err| Error::ConversionError {
                    desc: format!("{:?}", err),
                })?;
            cursor.into_inner()
        };

        Ok(Image {
            height: height as i32,
            width: width as i32,
            colorspace: colorspace as i32,
            encoded_image_string,
        })
    }
}

impl<B> TryFrom<FlatSamples<B>> for Image
where
    B: AsRef<[u8]>,
{
    type Error = Error;

    fn try_from(from: FlatSamples<B>) -> Result<Self, Self::Error> {
        Self::try_from(&from)
    }
}

// ImageBuffer to image

impl<P, C> TryFrom<&ImageBuffer<P, C>> for Image
where
    P: 'static + Pixel<Subpixel = u8>,
    C: Deref<Target = [P::Subpixel]> + AsRef<[P::Subpixel]>,
{
    type Error = Error;

    fn try_from(from: &ImageBuffer<P, C>) -> Result<Self, Self::Error> {
        Self::try_from(from.as_flat_samples())
    }
}
