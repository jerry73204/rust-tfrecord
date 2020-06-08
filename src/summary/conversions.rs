use super::*;

impl From<&Histogram> for HistogramProto {
    fn from(from: &Histogram) -> Self {
        let Histogram {
            buckets,
            min,
            max,
            sum,
            sum_squares,
        } = from;

        let min = min.load(Ordering::Relaxed);
        let max = max.load(Ordering::Relaxed);
        let sum = sum.load(Ordering::Relaxed);
        let sum_squares = sum_squares.load(Ordering::Relaxed);

        let counts = buckets
            .iter()
            .map(|bucket| bucket.count.load(Ordering::Relaxed) as f64)
            .collect::<Vec<_>>();
        let limits = buckets
            .iter()
            .map(|bucket| bucket.limit.raw())
            .collect::<Vec<_>>();
        let total_count = counts.iter().sum();

        Self {
            min,
            max,
            num: total_count,
            sum,
            sum_squares,
            bucket_limit: limits,
            bucket: counts,
        }
    }
}

impl From<Histogram> for HistogramProto {
    fn from(from: Histogram) -> Self {
        Self::from(&from)
    }
}

impl From<&[f64]> for HistogramProto {
    fn from(from: &[f64]) -> Self {
        let histogram = Histogram::default();
        from.iter()
            .cloned()
            .for_each(|value| histogram.add(R64::new(value)));
        histogram.into()
    }
}

impl From<&Vec<f64>> for HistogramProto {
    fn from(from: &Vec<f64>) -> Self {
        Self::from(from.as_slice())
    }
}

impl From<Vec<f64>> for HistogramProto {
    fn from(from: Vec<f64>) -> Self {
        Self::from(from.as_slice())
    }
}

#[cfg(feature = "ndarray")]
mod ndarray_conv {
    use super::*;

    impl<S, D> TryFrom<&ndarray::ArrayBase<S, D>> for HistogramProto
    where
        S: ndarray::RawData<Elem = f64> + ndarray::Data,
        D: ndarray::Dimension,
    {
        type Error = Error;

        fn try_from(from: &ndarray::ArrayBase<S, D>) -> Result<Self, Self::Error> {
            let histogram = Histogram::default();
            let values_iter = from.iter().cloned().map(|value| {
                R64::try_new(value).ok_or_else(|| Error::ConversionError {
                    desc: "non-finite floating value found".into(),
                })
            });

            for result in values_iter {
                let value = result?;
                histogram.add(value);
            }

            Ok(histogram.into())
        }
    }

    impl<S, D> TryFrom<ndarray::ArrayBase<S, D>> for HistogramProto
    where
        S: ndarray::RawData<Elem = f64> + ndarray::Data,
        D: ndarray::Dimension,
    {
        type Error = Error;

        fn try_from(from: ndarray::ArrayBase<S, D>) -> Result<Self, Self::Error> {
            Self::try_from(&from)
        }
    }

    impl<D> From<&ndarray::Array<i32, D>> for TensorProto
    where
        D: ndarray::Dimension,
    {
        fn from(from: &ndarray::Array<i32, D>) -> Self {
            let tensor_content = from
                .iter()
                .flat_map(|value| value.to_le_bytes().iter().cloned().collect::<Vec<_>>())
                .collect::<Vec<_>>();
            let shape = TensorShapeProto {
                dim: from
                    .shape()
                    .iter()
                    .cloned()
                    .map(|sz| Dim {
                        size: sz as i64,
                        name: "".into(),
                    })
                    .collect::<Vec<_>>(),
                unknown_rank: false,
            };

            TensorProto {
                dtype: DataType::DtInt32 as i32,
                tensor_shape: Some(shape),
                version_number: 0,
                tensor_content,
                half_val: vec![],
                float_val: vec![],
                double_val: vec![],
                int_val: vec![],
                string_val: vec![],
                scomplex_val: vec![],
                int64_val: vec![],
                bool_val: vec![],
                dcomplex_val: vec![],
                resource_handle_val: vec![],
                variant_val: vec![],
                uint32_val: vec![],
                uint64_val: vec![],
            }
        }
    }

    impl<D> From<ndarray::Array<i32, D>> for TensorProto
    where
        D: ndarray::Dimension,
    {
        fn from(from: ndarray::Array<i32, D>) -> Self {
            Self::from(&from)
        }
    }

    impl<D> From<&ndarray::Array<i64, D>> for TensorProto
    where
        D: ndarray::Dimension,
    {
        fn from(from: &ndarray::Array<i64, D>) -> Self {
            let tensor_content = from
                .iter()
                .flat_map(|value| value.to_le_bytes().iter().cloned().collect::<Vec<_>>())
                .collect::<Vec<_>>();
            let shape = TensorShapeProto {
                dim: from
                    .shape()
                    .iter()
                    .cloned()
                    .map(|sz| Dim {
                        size: sz as i64,
                        name: "".into(),
                    })
                    .collect::<Vec<_>>(),
                unknown_rank: false,
            };

            TensorProto {
                dtype: DataType::DtInt64 as i32,
                tensor_shape: Some(shape),
                version_number: 0,
                tensor_content,
                half_val: vec![],
                float_val: vec![],
                double_val: vec![],
                int_val: vec![],
                string_val: vec![],
                scomplex_val: vec![],
                int64_val: vec![],
                bool_val: vec![],
                dcomplex_val: vec![],
                resource_handle_val: vec![],
                variant_val: vec![],
                uint32_val: vec![],
                uint64_val: vec![],
            }
        }
    }

    impl<D> From<ndarray::Array<i64, D>> for TensorProto
    where
        D: ndarray::Dimension,
    {
        fn from(from: ndarray::Array<i64, D>) -> Self {
            Self::from(&from)
        }
    }

    impl<D> From<&ndarray::Array<f32, D>> for TensorProto
    where
        D: ndarray::Dimension,
    {
        fn from(from: &ndarray::Array<f32, D>) -> Self {
            let tensor_content = from
                .iter()
                .flat_map(|value| value.to_le_bytes().iter().cloned().collect::<Vec<_>>())
                .collect::<Vec<_>>();
            let shape = TensorShapeProto {
                dim: from
                    .shape()
                    .iter()
                    .cloned()
                    .map(|sz| Dim {
                        size: sz as i64,
                        name: "".into(),
                    })
                    .collect::<Vec<_>>(),
                unknown_rank: false,
            };

            TensorProto {
                dtype: DataType::DtFloat as i32,
                tensor_shape: Some(shape),
                version_number: 0,
                tensor_content,
                half_val: vec![],
                float_val: vec![],
                double_val: vec![],
                int_val: vec![],
                string_val: vec![],
                scomplex_val: vec![],
                int64_val: vec![],
                bool_val: vec![],
                dcomplex_val: vec![],
                resource_handle_val: vec![],
                variant_val: vec![],
                uint32_val: vec![],
                uint64_val: vec![],
            }
        }
    }

    impl<D> From<ndarray::Array<f32, D>> for TensorProto
    where
        D: ndarray::Dimension,
    {
        fn from(from: ndarray::Array<f32, D>) -> Self {
            Self::from(&from)
        }
    }

    impl<D> From<&ndarray::Array<f64, D>> for TensorProto
    where
        D: ndarray::Dimension,
    {
        fn from(from: &ndarray::Array<f64, D>) -> Self {
            let tensor_content = from
                .iter()
                .flat_map(|value| value.to_le_bytes().iter().cloned().collect::<Vec<_>>())
                .collect::<Vec<_>>();
            let shape = TensorShapeProto {
                dim: from
                    .shape()
                    .iter()
                    .cloned()
                    .map(|sz| Dim {
                        size: sz as i64,
                        name: "".into(),
                    })
                    .collect::<Vec<_>>(),
                unknown_rank: false,
            };

            TensorProto {
                dtype: DataType::DtDouble as i32,
                tensor_shape: Some(shape),
                version_number: 0,
                tensor_content,
                half_val: vec![],
                float_val: vec![],
                double_val: vec![],
                int_val: vec![],
                string_val: vec![],
                scomplex_val: vec![],
                int64_val: vec![],
                bool_val: vec![],
                dcomplex_val: vec![],
                resource_handle_val: vec![],
                variant_val: vec![],
                uint32_val: vec![],
                uint64_val: vec![],
            }
        }
    }

    impl<D> From<ndarray::Array<f64, D>> for TensorProto
    where
        D: ndarray::Dimension,
    {
        fn from(from: ndarray::Array<f64, D>) -> Self {
            Self::from(&from)
        }
    }

}

#[cfg(feature = "tch")]
mod tch_conv {
    use super::*;
    use tch::{Kind, Tensor};

    impl TryFrom<&Tensor> for HistogramProto {
        type Error = Error;

        fn try_from(from: &Tensor) -> Result<Self, Self::Error> {
            let numel = from.numel();
            let kind = from.kind();
            let values = match kind {
                Kind::Float => {
                    let raw_values = unsafe {
                        let mut data: Vec<f32> = Vec::with_capacity(numel);
                        let slice = slice::from_raw_parts_mut(data.as_mut_ptr(), numel);
                        from.copy_data(slice, numel);
                        data.set_len(numel);
                        data
                    };

                    let values = raw_values
                        .into_iter()
                        .map(|value| {
                            R64::try_new(value as f64).ok_or_else(|| Error::ConversionError {
                                desc: "non-finite floating point value found".into(),
                            })
                        })
                        .collect::<Result<Vec<_>, Error>>()?;
                    values
                }
                Kind::Double => {
                    let raw_values = unsafe {
                        let mut data: Vec<f64> = Vec::with_capacity(numel);
                        let slice = slice::from_raw_parts_mut(data.as_mut_ptr(), numel);
                        from.copy_data(slice, numel);
                        data.set_len(numel);
                        data
                    };
                    let values = raw_values
                        .into_iter()
                        .map(|value| {
                            R64::try_new(value).ok_or_else(|| Error::ConversionError {
                                desc: "non-finite floating point value found".into(),
                            })
                        })
                        .collect::<Result<Vec<_>, Error>>()?;
                    values
                }
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

    impl TryFrom<&Tensor> for TensorProto {
        type Error = Error;

        fn try_from(from: &Tensor) -> Result<Self, Self::Error> {
            let numel = from.numel();
            let size = from.size();
            let kind = from.kind();

            let (dtype, tensor_content) = match kind {
                Kind::Int => {
                    let dtype = DataType::DtInt32;
                    let values = unsafe {
                        let mut data: Vec<i32> = Vec::with_capacity(numel);
                        let slice = slice::from_raw_parts_mut(data.as_mut_ptr(), numel);
                        from.copy_data(slice, numel);
                        data.set_len(numel);
                        data
                    };
                    let content = values
                        .into_iter()
                        .flat_map(|value| value.to_le_bytes().iter().cloned().collect::<Vec<_>>())
                        .collect::<Vec<_>>();
                    (dtype, content)
                }
                Kind::Int64 => {
                    let dtype = DataType::DtInt64;
                    let values = unsafe {
                        let mut data: Vec<i64> = Vec::with_capacity(numel);
                        let slice = slice::from_raw_parts_mut(data.as_mut_ptr(), numel);
                        from.copy_data(slice, numel);
                        data.set_len(numel);
                        data
                    };
                    let content = values
                        .into_iter()
                        .flat_map(|value| value.to_le_bytes().iter().cloned().collect::<Vec<_>>())
                        .collect::<Vec<_>>();
                    (dtype, content)
                }
                Kind::Float => {
                    let dtype = DataType::DtFloat;
                    let values = unsafe {
                        let mut data: Vec<f32> = Vec::with_capacity(numel);
                        let slice = slice::from_raw_parts_mut(data.as_mut_ptr(), numel);
                        from.copy_data(slice, numel);
                        data.set_len(numel);
                        data
                    };
                    let content = values
                        .into_iter()
                        .flat_map(|value| value.to_le_bytes().iter().cloned().collect::<Vec<_>>())
                        .collect::<Vec<_>>();
                    (dtype, content)
                }
                Kind::Double => {
                    let dtype = DataType::DtDouble;
                    let values = unsafe {
                        let mut data: Vec<f64> = Vec::with_capacity(numel);
                        let slice = slice::from_raw_parts_mut(data.as_mut_ptr(), numel);
                        from.copy_data(slice, numel);
                        data.set_len(numel);
                        data
                    };
                    let content = values
                        .into_iter()
                        .flat_map(|value| value.to_le_bytes().iter().cloned().collect::<Vec<_>>())
                        .collect::<Vec<_>>();
                    (dtype, content)
                }
                _ => {
                    return Err(Error::ConversionError {
                        desc: format!("unsupported tensor kind {:?}", kind),
                    })
                }
            };

            let shape = TensorShapeProto {
                dim: size
                    .into_iter()
                    .map(|sz| Dim {
                        size: sz,
                        name: "".into(),
                    })
                    .collect::<Vec<_>>(),
                unknown_rank: false,
            };

            Ok(TensorProto {
                dtype: dtype as i32,
                tensor_shape: Some(shape),
                version_number: 0,
                tensor_content,
                half_val: vec![],
                float_val: vec![],
                double_val: vec![],
                int_val: vec![],
                string_val: vec![],
                scomplex_val: vec![],
                int64_val: vec![],
                bool_val: vec![],
                dcomplex_val: vec![],
                resource_handle_val: vec![],
                variant_val: vec![],
                uint32_val: vec![],
                uint64_val: vec![],
            })
        }
    }

    impl TryFrom<Tensor> for TensorProto {
        type Error = Error;
        fn try_from(from: Tensor) -> Result<Self, Self::Error> {
            Self::try_from(&from)
        }
    }
}

#[cfg(feature = "image")]
mod image_conv {
    use super::*;
    use image::{
        flat::SampleLayout, png::PNGEncoder, Bgra, ColorType, DynamicImage, FlatSamples,
        ImageBuffer, Luma, LumaA, Pixel, Rgb, Rgba,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[repr(i32)]
    enum ColorSpace {
        Luma = 1,
        LumaA = 2,
        Rgb = 3,
        Rgba = 4,
        DigitalYuv = 5,
        Bgra = 6,
    }

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

    impl<P, C> TryFrom<&ImageBuffer<P, C>> for HistogramProto
    where
        P: 'static + Pixel,
        P::Subpixel: 'static,
        C: Deref<Target = [P::Subpixel]>,
        f64: TryFrom<P::Subpixel>,
    {
        type Error = Error;

        fn try_from(from: &ImageBuffer<P, C>) -> Result<Self, Self::Error> {
            let components_iter = from
                .pixels()
                .flat_map(|pixel| pixel.channels().iter().cloned().collect::<Vec<_>>())
                .map(|component| {
                    R64::try_new(
                        f64::try_from(component).map_err(|_| Error::ConversionError {
                            desc: "unsupported pixel type".into(),
                        })?,
                    )
                    .ok_or_else(|| Error::ConversionError {
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
        f64: TryFrom<P::Subpixel>,
    {
        type Error = Error;

        fn try_from(from: ImageBuffer<P, C>) -> Result<Self, Self::Error> {
            Self::try_from(&from)
        }
    }

    // impl From<&DynamicImage> for TensorProto {
    //     fn from(from: &DynamicImage) -> Self {
    //         todo!();
    //     }
    // }

    // impl From<DynamicImage> for TensorProto {
    //     fn from(from: DynamicImage) -> Self {
    //         Self::from(&from)
    //     }
    // }

    // impl<P, C> From<&ImageBuffer<P, C>> for TensorProto
    // where
    //     P: Pixel,
    // {
    //     fn from(from: &ImageBuffer<P, C>) -> Self {
    //         todo!();
    //     }
    // }

    // impl<P, C> From<ImageBuffer<P, C>> for TensorProto
    // where
    //     P: Pixel,
    // {
    //     fn from(from: ImageBuffer<P, C>) -> Self {
    //         Self::from(&from)
    //     }
    // }

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

    impl<B> TryFrom<&FlatSamples<B>> for Image
    where
        B: AsRef<[u8]>,
    {
        type Error = Error;

        fn try_from(from: &FlatSamples<B>) -> Result<Self, Self::Error> {
            let FlatSamples {
                layout: SampleLayout { width, height, .. },
                color_hint,
                ..
            } = *from;
            let samples = from.samples.as_ref();
            let color_type = color_hint.ok_or_else(|| Error::ConversionError {
                desc: "color_hint must not be None".into(),
            })?;
            let colorspace = ColorSpace::try_from(color_type)?;

            let encoded_image_string = {
                let mut cursor = Cursor::new(vec![]);
                PNGEncoder::new(&mut cursor)
                    .encode(samples, width, height, color_type)
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

    impl<C> TryFrom<&ImageBuffer<Luma<u8>, C>> for Image
    where
        C: Deref<Target = [u8]> + AsRef<[u8]>,
    {
        type Error = Error;

        fn try_from(from: &ImageBuffer<Luma<u8>, C>) -> Result<Self, Self::Error> {
            Self::try_from(from.as_flat_samples())
        }
    }

    impl<C> TryFrom<ImageBuffer<Luma<u8>, C>> for Image
    where
        C: Deref<Target = [u8]> + AsRef<[u8]>,
    {
        type Error = Error;

        fn try_from(from: ImageBuffer<Luma<u8>, C>) -> Result<Self, Self::Error> {
            Self::try_from(&from)
        }
    }

    impl<C> TryFrom<&ImageBuffer<LumaA<u8>, C>> for Image
    where
        C: Deref<Target = [u8]> + AsRef<[u8]>,
    {
        type Error = Error;

        fn try_from(from: &ImageBuffer<LumaA<u8>, C>) -> Result<Self, Self::Error> {
            Self::try_from(from.as_flat_samples())
        }
    }

    impl<C> TryFrom<ImageBuffer<LumaA<u8>, C>> for Image
    where
        C: Deref<Target = [u8]> + AsRef<[u8]>,
    {
        type Error = Error;

        fn try_from(from: ImageBuffer<LumaA<u8>, C>) -> Result<Self, Self::Error> {
            Self::try_from(&from)
        }
    }

    impl<C> TryFrom<&ImageBuffer<Rgb<u8>, C>> for Image
    where
        C: Deref<Target = [u8]> + AsRef<[u8]>,
    {
        type Error = Error;

        fn try_from(from: &ImageBuffer<Rgb<u8>, C>) -> Result<Self, Self::Error> {
            Self::try_from(from.as_flat_samples())
        }
    }

    impl<C> TryFrom<ImageBuffer<Rgb<u8>, C>> for Image
    where
        C: Deref<Target = [u8]> + AsRef<[u8]>,
    {
        type Error = Error;

        fn try_from(from: ImageBuffer<Rgb<u8>, C>) -> Result<Self, Self::Error> {
            Self::try_from(&from)
        }
    }

    impl<C> TryFrom<&ImageBuffer<Rgba<u8>, C>> for Image
    where
        C: Deref<Target = [u8]> + AsRef<[u8]>,
    {
        type Error = Error;

        fn try_from(from: &ImageBuffer<Rgba<u8>, C>) -> Result<Self, Self::Error> {
            Self::try_from(from.as_flat_samples())
        }
    }

    impl<C> TryFrom<ImageBuffer<Rgba<u8>, C>> for Image
    where
        C: Deref<Target = [u8]> + AsRef<[u8]>,
    {
        type Error = Error;

        fn try_from(from: ImageBuffer<Rgba<u8>, C>) -> Result<Self, Self::Error> {
            Self::try_from(&from)
        }
    }

    impl<C> TryFrom<&ImageBuffer<Bgra<u8>, C>> for Image
    where
        C: Deref<Target = [u8]> + AsRef<[u8]>,
    {
        type Error = Error;

        fn try_from(from: &ImageBuffer<Bgra<u8>, C>) -> Result<Self, Self::Error> {
            Self::try_from(from.as_flat_samples())
        }
    }

    impl<C> TryFrom<ImageBuffer<Bgra<u8>, C>> for Image
    where
        C: Deref<Target = [u8]> + AsRef<[u8]>,
    {
        type Error = Error;

        fn try_from(from: ImageBuffer<Bgra<u8>, C>) -> Result<Self, Self::Error> {
            Self::try_from(&from)
        }
    }
}
