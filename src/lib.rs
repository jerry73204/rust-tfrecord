//! The crate provides the functionality to serialize and deserialize TFRecord data format from TensorFlow.
//!
//! # Cargo Features
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
//!
//! # Manualy ProtocolBuffer Code Generation
//!
//! The crate compiles the pre-generated ProtocolBuffer code from TensorFlow. In the case of TensorFlow version update, you may generate the code manually. It accepts several ways to access the TensorFlow source code specified by `TFRECORD_BUILD_METHOD` environment variable. The generated code is placed under `prebuild_src` directory. See the examples below and change `X.Y.Z` to actual TensorFlow version.
//!
//! ## Build from a source tarball
//!
//! ```sh
//! export TFRECORD_BUILD_METHOD="src_file:///home/myname/tensorflow-X.Y.Z.tar.gz"
//! cargo build --release --features generate_protobuf_src
//! ```
//!
//! ## Build from a source directory
//!
//! ```sh
//! export TFRECORD_BUILD_METHOD="src_dir:///home/myname/tensorflow-X.Y.Z"
//! cargo build --release --features generate_protobuf_src
//! ```
//!
//! ## Build from a URL
//!
//! ```sh
//! export TFRECORD_BUILD_METHOD="url://https://github.com/tensorflow/tensorflow/archive/vX.Y.Z.tar.gz"
//! cargo build --release --features generate_protobuf_src
//! ```
//!
//! ## Build from installed TensorFlow on system
//!
//! The build script will search `${install_prefix}/include/tensorflow` directory for protobuf documents.
//!
//! ```sh
//! export TFRECORD_BUILD_METHOD="install_prefix:///usr"
//! cargo build --release --features generate_protobuf_src
//! ```

// mods

pub mod error;
pub mod event;
pub mod event_writer;
pub mod indexer;
pub mod io;
pub mod protobuf;
pub mod protobuf_ext;
pub mod record;
pub mod record_reader;
pub mod record_writer;
mod utils;

// re-exports

pub use error::*;
pub use event::*;
pub use event_writer::*;
pub use protobuf::{Event, Example, Feature, Summary};
pub use protobuf_ext::*;
pub use record::*;
pub use record_reader::*;
pub use record_writer::*;
