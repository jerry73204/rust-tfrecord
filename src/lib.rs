//! The crate provides the functionality to serialize and deserialize TFRecord data format from TensorFlow.
//!
//! The crate provides several cargo features that you can conditionally compile modules.
//!
//! Optional features:
//! - `full`: Enable all features.
//! - `async`: Enable async/await feature.
//!
//! Third-party crate supports:
//! - `with-serde`: Enable interoperability with [serde](https://crates.io/crates/serde) to serialize and deserialize example types.
//! - `with-tch`: Enable [tch](https://crates.io/crates/tch) types support.
//! - `with-image`: Enable [image](https://crates.io/crates/image) types support.
//! - `with-ndarray`: Enable [ndarray](https://crates.io/crates/ndarray) types support.

// mods

pub mod error;
pub mod event;
pub mod event_writer;
pub mod indexer;
pub mod io;
pub mod markers;
pub mod protobuf;
pub mod protobuf_ext;
pub mod record_reader;
pub mod record_writer;
pub mod types;
mod utils;

// re-exports

pub use error::{Error, Result};
pub use event::*;
pub use event_writer::*;
pub use markers::GenericRecord;
pub use protobuf::{Event, Example as RawExample, Summary};
pub use record_reader::*;
pub use record_writer::*;
pub use types::*;
