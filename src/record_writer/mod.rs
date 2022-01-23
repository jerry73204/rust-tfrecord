//! TFRecord data writer.
//!
//! It provides several writers to serialize [GenericRecord](crate::markers::GenericRecord)
//! types.
//!
//! | Writer                                     | Record type                     |
//! | -------------------------------------------|---------------------------------|
//! | [BytesWriter](sync::BytesWriter)           | [Vec<u8>](Vec)                  |
//! | [RawExampleWriter](sync::RawExampleWriter) | [RawExample](crate::RawExample) |
//! | [ExampleWriter](sync::ExampleWriter)       | [Example](crate::Example)       |
//! | [RecordWriter](sync::RecordWriter)         | Type that implements [GenericRecord](crate::markers::GenericRecord) |
//!
//! The asynchronous counterparts are named in `AsyncWriter` suffix.
//!
//! | Writer                                                | Record type                     |
//! | ------------------------------------------------------|---------------------------------|
//! | [BytesAsyncWriter](async::BytesAsyncWriter)           | [Vec<u8>](Vec)                  |
//! | [RawExampleAsyncWriter](async::RawExampleAsyncWriter) | [RawExample](crate::RawExample) |
//! | [ExampleAsyncWriter](async::ExampleAsyncWriter)       | [Example](crate::Example)       |
//! | [RecordAsyncWriter](async::RecordAsyncWriter)         | Type that implements [GenericRecord](crate::markers::GenericRecord) |

#[cfg(feature = "async")]
mod r#async;
#[cfg(feature = "async")]
pub use r#async::*;

mod sync;
pub use sync::*;
