//! Writing TFRecord data format.
//!
//! The [RecordWriter] is initialized by [RecordWriterInit]. It can write
//! either [Example], [RawExampple](crate::RawExample), [Vec\<u8\>](Vec), and many other record types.
//! that implements [GenericRecord], depending on your choice.
//!
//! The type aliases [ExampleWriter], [RawExampleWriter] and [BytesWriter]
//! are [RecordWriter] writing specific record types.

#[cfg(feature = "async")]
mod r#async;
#[cfg(feature = "async")]
pub use r#async::*;

mod sync;
pub use sync::*;
