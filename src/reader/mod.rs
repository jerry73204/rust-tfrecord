//! Reading TFRecord data format.
//!
//! The module provides both blocking [RecordReaderInit] and
//! asynchronous [RecordStreamInit] reader initializers to build
//! reader and stream types.
//!
//! The [RecordReader] type, constructed by [RecordReaderInit],
//! implements the [Iterator](std::iter::Iterator) such that you can work with loops.
//!
//! The [RecordStreamInit] initializer constructs streams from types with [AsyncRead](AsyncRead) trait.
//! The streams can integrated with [futures::stream] API.

#[cfg(feature = "async")]
mod r#async;
#[cfg(feature = "async")]
pub use r#async::*;

mod sync;
pub use sync::*;

/// The reader initializer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordReaderConfig {
    pub check_integrity: bool,
}

impl Default for RecordReaderConfig {
    fn default() -> Self {
        Self {
            check_integrity: true,
        }
    }
}
