//! Utility traits and operators.

use crate::protos::DataType;

// macros

macro_rules! to_f64 {
    ($value:expr, u8) => {
        $value as f64
    };
    ($value:expr, u16) => {
        $value as f64
    };
    ($value:expr, u32) => {
        $value as f64
    };
    ($value:expr, u64) => {
        $value as f64
    };
    ($value:expr, i8) => {
        $value as f64
    };
    ($value:expr, i16) => {
        $value as f64
    };
    ($value:expr, i32) => {
        $value as f64
    };
    ($value:expr, i64) => {
        $value as f64
    };
    ($value:expr, f32) => {
        $value as f64
    };
    ($value:expr, f64) => {
        $value as f64
    };
}

// traits

/// A helper trait that converts primitive values to little-endian bytes.
pub trait ToLeBytes
where
    Self: Copy,
{
    const DATA_TYPE: DataType;

    fn to_bytes(&self) -> Vec<u8>;
}

macro_rules! impl_to_le_bytes {
    ($ty:ty, $dtype:expr) => {
        impl ToLeBytes for $ty {
            const DATA_TYPE: DataType = $dtype;

            fn to_bytes(&self) -> Vec<u8> {
                self.to_le_bytes().iter().cloned().collect()
            }
        }
    };
}

impl_to_le_bytes!(u8, DataType::DtUint8);
impl_to_le_bytes!(u16, DataType::DtUint16);
impl_to_le_bytes!(u32, DataType::DtUint32);
impl_to_le_bytes!(u64, DataType::DtUint64);
impl_to_le_bytes!(i8, DataType::DtInt8);
impl_to_le_bytes!(i16, DataType::DtInt16);
impl_to_le_bytes!(i32, DataType::DtInt32);
impl_to_le_bytes!(i64, DataType::DtInt64);
impl_to_le_bytes!(f32, DataType::DtFloat);
impl_to_le_bytes!(f64, DataType::DtDouble);

/// A helper trait that converts primitive values to [f64](f64).
pub trait ToF64
where
    Self: Copy,
{
    fn to_f64(&self) -> f64;
}

impl ToF64 for u8 {
    fn to_f64(&self) -> f64 {
        to_f64!(*self, u8)
    }
}

impl ToF64 for u16 {
    fn to_f64(&self) -> f64 {
        to_f64!(*self, u16)
    }
}

impl ToF64 for u32 {
    fn to_f64(&self) -> f64 {
        to_f64!(*self, u32)
    }
}

impl ToF64 for u64 {
    fn to_f64(&self) -> f64 {
        to_f64!(*self, u64)
    }
}

impl ToF64 for i8 {
    fn to_f64(&self) -> f64 {
        to_f64!(*self, i8)
    }
}

impl ToF64 for i16 {
    fn to_f64(&self) -> f64 {
        to_f64!(*self, i16)
    }
}

impl ToF64 for i32 {
    fn to_f64(&self) -> f64 {
        to_f64!(*self, i32)
    }
}

impl ToF64 for i64 {
    fn to_f64(&self) -> f64 {
        to_f64!(*self, i64)
    }
}

impl ToF64 for f32 {
    fn to_f64(&self) -> f64 {
        to_f64!(*self, f32)
    }
}

impl ToF64 for f64 {
    fn to_f64(&self) -> f64 {
        to_f64!(*self, f64)
    }
}
