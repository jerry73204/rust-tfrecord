#![cfg(feature = "summary")]

//! Summary and event types and writers for TensorBoard.

use crate::{
    error::Error,
    protos::{
        event::What,
        summary::{value::Value as ValueEnum, Audio, Image, Value},
        Event, HistogramProto, Summary, TensorProto,
    },
    writer::{RecordWriter, RecordWriterInit},
};
#[cfg(feature = "async_")]
use futures::io::AsyncWriteExt;
use std::{
    convert::TryInto,
    fs,
    io::Write,
    path::{Path, PathBuf, MAIN_SEPARATOR},
    string::ToString,
    time::SystemTime,
};

mod event;
mod histogram;
mod writer;

pub use event::*;
pub use histogram::*;
pub use writer::*;
