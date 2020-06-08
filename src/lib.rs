//! The crate provides the functionality to serialize and deserialize TFRecord data format from TensorFlow.
//!
//! The crate provides several cargo features that you can conditionally compile modules.
//!
//! - `serde`: Enable interoperability with [serde](https://github.com/serde-rs/serde) to serialize and deserialize example types.
//! - `async_`: Enable async/await feature.
//! - `dataset`: Enable the dataset API.
//! - `full`: Enable all features above.

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

/// The protobuf-generated example type from TensorFlow.
///
/// The type is re-exported from [protos::Example]. It is suggested
/// to use the [record::Example] alternative.
pub use protos::Example as RawExample;

#[cfg(feature = "async_")]
pub use reader::RecordStreamInit;
pub use reader::{BytesReader, ExampleReader, RawExampleReader, RecordReader, RecordReaderInit};
pub use record::{Example, Feature};
pub use writer::{BytesWriter, ExampleWriter, RawExampleWriter, RecordWriter, RecordWriterInit};

#[cfg(feature = "dataset")]
pub use dataset::{Dataset, DatasetInit};
