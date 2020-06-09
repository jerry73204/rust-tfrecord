//! The crate provides the functionality to serialize and deserialize TFRecord data format from TensorFlow.
//!
//! The crate provides several cargo features that you can conditionally compile modules.
//!
//! Optional features:
//! - `full`: Enable all features.
//! - `async_`: Enable async/await feature.
//! - `dataset`: Enable the dataset API.
//! - `summary`: Enable the summary and event API, which is mainly targeted for TensorBoard.
//!
//! Third-party supports:
//! - `serde`: Enable interoperability with [serde](https://crates.io/crates/serde) to serialize and deserialize example types.
//! - `tch`: Enable [tch](https://crates.io/crates/tch) types support.
//! - `image`: Enable [image](https://crates.io/crates/image) types support.
//! - `ndarray`: Enable [ndarray](https://crates.io/crates/ndarray) types support.

#[cfg(feature = "dataset")]
pub mod dataset;
pub mod error;
pub mod io;
pub mod markers;
pub mod protos;
pub mod reader;
pub mod record;
pub mod summary;
mod utils;
pub mod writer;

pub use error::Error;
pub use markers::GenericRecord;

pub use protos::{Event, Example as RawExample, Summary};

#[cfg(feature = "async_")]
pub use reader::RecordStreamInit;
pub use reader::{BytesReader, ExampleReader, RawExampleReader, RecordReader, RecordReaderInit};
pub use record::{Example, Feature};
#[cfg(feature = "summary")]
pub use summary::{EventInit, EventWriter, EventWriterInit, Histogram, SummaryInit};
pub use writer::{BytesWriter, ExampleWriter, RawExampleWriter, RecordWriter, RecordWriterInit};

#[cfg(feature = "dataset")]
pub use dataset::{Dataset, DatasetInit};
