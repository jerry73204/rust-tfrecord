//! TFRecord data writer.
//!
//! It provides several writers to serialize [Record](crate::record::Record)
//! types.
//!
//! | Writer                                     | Record type                     |
//! | -------------------------------------------|---------------------------------|
//! | [BytesWriter](sync::BytesWriter)           | [Vec<u8>](Vec)                  |
//! | [ExampleWriter](sync::ExampleWriter)       | [Example](crate::Example)       |
//! | [RecordWriter](sync::RecordWriter)         | Type that implements [Record](crate::record::Record) |
//!
//! The asynchronous counterparts are named in `AsyncWriter` suffix.
//!
//! | Writer                                                | Record type                     |
//! | ------------------------------------------------------|---------------------------------|
//! | [BytesAsyncWriter](async::BytesAsyncWriter)           | [Vec<u8>](Vec)                  |
//! | [ExampleAsyncWriter](async::ExampleAsyncWriter)       | [Example](crate::Example)       |
//! | [RecordAsyncWriter](async::RecordAsyncWriter)         | Type that implements [Record](crate::record::Record) |

#[cfg(feature = "async")]
mod r#async;
#[cfg(feature = "async")]
pub use r#async::*;

mod sync;
pub use sync::*;
