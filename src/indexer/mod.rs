//! The indexer that enumerate record locations from one or multiple TFRecord files.

mod sync;
pub use sync::*;

#[cfg(feature = "async")]
mod r#async;
#[cfg(feature = "async")]
pub use r#async::*;

use std::{path::PathBuf, sync::Arc};

/// The file path and record position in file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordIndex {
    pub path: Arc<PathBuf>,
    pub offset: u64,
    pub len: usize,
}

/// Record position in file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub offset: u64,
    pub len: usize,
}

/// Configuration for indexer methods.
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
