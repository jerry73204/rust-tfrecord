//! TFRecord data reader.
//!
//! The [RecordIter] iterator reads records from a file.
//!
//! The [RecordStream] reads records from a file and can cooperated with future's [stream](futures::stream) API..

#[cfg(feature = "async")]
mod r#async;
#[cfg(feature = "async")]
pub use r#async::*;

mod sync;
pub use sync::*;

/// Configuration for record reader.
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
