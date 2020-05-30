pub mod error;
pub mod io;
pub mod markers;
pub mod protos;
pub mod reader;
pub mod record;
mod utils;
pub mod writer;

pub use error::Error;
pub use markers::GenericRecord;
pub use protos::{feature::Kind, BytesList, Example, Feature, FloatList, Int64List};
pub use reader::{
    BytesReader, EasyExampleReader, ExampleReader, RecordReader, RecordReaderInit, RecordStreamInit,
};
pub use record::{EasyExample, EasyFeature};
pub use writer::{BytesWriter, EasyExampleWriter, ExampleWriter, RecordWriter, RecordWriterInit};
