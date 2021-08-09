#![cfg(feature = "summary")]

//! Types of summaries and events and writers for TensorBoard.
//!
//! The [EventWriter] writes the file that can be recognized by TensorBoard.
//! See the document of [EventWriter] to understand the usage.

use crate::{
    error::Error,
    markers::TryInfoImageList,
    protos::{
        event::What,
        summary::{value::Value as ValueEnum, Audio, Image, Value},
        Event, HistogramProto, Summary, TensorProto,
    },
    writer::{RecordWriter, RecordWriterInit},
};
#[cfg(feature = "async")]
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
mod writer;

pub use event::*;
pub use writer::*;
