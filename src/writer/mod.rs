//! Writing TFRecord data format.
//!
//! The [RecordWriter] is initialized by [RecordWriterInit]. It can write
//! either [Example], [RawExampple](crate::RawExample), [Vec\<u8\>](Vec), and many other record types.
//! that implements [GenericRecord], depending on your choice.
//!
//! The type aliases [ExampleWriter], [RawExampleWriter] and [BytesWriter]
//! are [RecordWriter] writing specific record types.

mod r#async;
mod sync;

use crate::{markers::GenericRecord, protobuf::Example as RawExample, types::Example};
use std::marker::PhantomData;

/// Alias to [RecordWriter] which input record type is [Vec<u8>](Vec).
pub type BytesWriter<W> = RecordWriter<Vec<u8>, W>;
/// Alias to [RecordWriter] which input record type is [RawExample].
pub type RawExampleWriter<W> = RecordWriter<RawExample, W>;
/// Alias to [RecordWriter] which input record type is [Example].
pub type ExampleWriter<W> = RecordWriter<Example, W>;

/// The writer type.
///
/// It provides blocing [RecordWriter::send] and analogues [RecordWriter::send_async] methods
/// to write records.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordWriter<T, W>
where
    T: GenericRecord,
{
    writer: W,
    _phantom: PhantomData<T>,
}
