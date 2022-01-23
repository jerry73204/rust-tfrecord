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
//! - `with-serde`: Enable interoperability with [serde](https://crates.io/crates/serde) to serialize and deserialize example types.
//! - `with-tch`: Enable [tch](https://crates.io/crates/tch) types support.
//! - `with-image`: Enable [image](https://crates.io/crates/image) types support.
//! - `with-ndarray`: Enable [ndarray](https://crates.io/crates/ndarray) types support.

// mods

#[cfg(feature = "dataset")]
pub mod dataset;

pub mod error;
pub mod io;
pub mod markers;
pub mod protobuf;
pub mod protobuf_ext;
pub mod reader;
pub mod summary;
pub mod types;
mod utils;
pub mod writer;

// re-exports

pub use error::Error;
pub use markers::GenericRecord;
pub use protobuf::{Event, Example as RawExample, Summary};

pub use reader::{
    BytesIter, EventIter, ExampleIter, RawExampleIter, RecordIter, RecordReaderConfig,
};
#[cfg(feature = "async")]
pub use reader::{BytesStream, EventStream, ExampleStream, RawExampleStream, RecordStream};
#[cfg(feature = "summary")]
pub use summary::{EventMeta, EventWriter, EventWriterConfig};
pub use types::{Example, Feature, Histogram};
pub use writer::{BytesWriter, ExampleWriter, RawExampleWriter, RecordWriter};

#[cfg(feature = "dataset")]
pub use dataset::{Dataset, DatasetInit};
