use crate::{
    error::Error,
    protos::{
        event::What,
        summary::{value::Value as ValueEnum, Audio, Image, Value},
        tensor_shape_proto::Dim,
        DataType, Event, HistogramProto, Summary, TensorProto, TensorShapeProto,
    },
    record::Example,
    writer::{RecordWriter, RecordWriterInit},
};
use atomig::Atomic;
#[cfg(feature = "async_")]
use futures::io::AsyncWrite;
use noisy_float::types::R64;
use std::{
    convert::TryFrom,
    io::Cursor,
    io::Write,
    iter,
    ops::{Deref, Neg},
    path::Path,
    slice,
    string::ToString,
    sync::atomic::{AtomicUsize, Ordering},
    time::SystemTime,
};

mod conversions;
mod event;
mod histogram;
mod writer;

pub use event::*;
pub use histogram::*;
pub use writer::*;
