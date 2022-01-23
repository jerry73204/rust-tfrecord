//! The dataset API that accesses multiple TFRecord files.
//!
//! The module is available when the `dataset` feature is enabled.
//! The [Dataset] type can be constructed using [DatasetInit] initializer.

mod sync;
pub use sync::*;

#[cfg(feature = "async")]
mod r#async;
#[cfg(feature = "async")]
pub use r#async::*;

use std::{path::PathBuf, sync::Arc};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordIndex {
    pub path: Arc<PathBuf>,
    pub offset: u64,
    pub len: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub offset: u64,
    pub len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordIndexerConfig {
    pub check_integrity: bool,
}

impl Default for RecordIndexerConfig {
    fn default() -> Self {
        Self {
            check_integrity: true,
        }
    }
}
