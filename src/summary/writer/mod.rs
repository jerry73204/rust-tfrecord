mod sync;
pub use sync::*;

#[cfg(feature = "async")]
mod r#async;
#[cfg(feature = "async")]
pub use r#async::*;

use crate::{error::Error, protobuf::Event, writer::RecordWriter};
use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    path::{Path, PathBuf, MAIN_SEPARATOR},
    string::ToString,
    time::SystemTime,
};

/// The event writer initializer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EventWriterConfig {
    /// If set, the writer flushes the buffer after writing a event.
    pub auto_flush: bool,
}

impl Default for EventWriterConfig {
    fn default() -> Self {
        Self { auto_flush: true }
    }
}

/// The event writer type.
///
/// The [EventWriter] is initialized by [EventWriterInit].
/// It provies a series `write_*` methods and `write_*_async` asynchronous
/// analogues to append events to the file recognized by TensorBoard.
///
/// The typical usage call the [EventWriterInit::from_prefix] with the log
/// directory to build a [EventWriter].
///
/// ```rust
/// # async_std::task::block_on(async move {
/// use anyhow::Result;
/// use std::time::SystemTime;
/// use tch::{kind::FLOAT_CPU, Tensor};
/// use tfrecord::EventWriter;
///
/// let mut writer = EventWriter::from_prefix("log_dir/myprefix-", "", Default::default()).unwrap();
///
/// // step = 0, scalar = 3.14
/// writer.write_scalar("my_scalar", 0, 3.14)?;
///
/// // step = 1, specified wall time, histogram of [1, 2, 3, 4]
/// writer.write_histogram("my_histogram", (1, SystemTime::now()), vec![1, 2, 3, 4])?;
///
/// // step = 2, specified raw UNIX time in nanoseconds, random tensor of shape [8, 3, 16, 16]
/// writer.write_tensor(
///     "my_tensor",
///     (2, 1.594449514712264e+18),
///     Tensor::randn(&[8, 3, 16, 16], FLOAT_CPU),
/// )?;
/// # anyhow::Ok(())
/// # }).unwrap();
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct EventWriter<W> {
    auto_flush: bool,
    events_writer: RecordWriter<Event, W>,
}

fn create_tf_style_path<'a, 'b, P, S>(
    prefix: P,
    file_name_suffix: S,
) -> Result<(PathBuf, OsString), Error>
where
    P: Into<Cow<'a, str>>,
    S: Into<Cow<'b, str>>,
{
    let prefix = prefix.into();
    let file_name_suffix = file_name_suffix.into();
    if prefix.is_empty() {
        return Err(Error::invalid_argument("the prefix must not be empty"));
    }

    let (dir, file_name_prefix): (&Path, &OsStr) = if prefix.ends_with(MAIN_SEPARATOR) {
        let dir = Path::new(prefix.as_ref());
        (dir, OsStr::new(""))
    } else {
        let prefix = Path::new(prefix.as_ref());
        let file_name_prefix = prefix.file_name().unwrap_or_else(|| OsStr::new(""));
        let dir = prefix.parent().unwrap(); // TODO
        (dir, file_name_prefix)
    };

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_micros();
    let host_name = hostname::get()?;
    let file_name: OsString = [
        file_name_prefix,
        OsStr::new(".out.tfevents."),
        OsStr::new(timestamp.to_string().as_str()),
        OsStr::new("."),
        &host_name,
        OsStr::new(file_name_suffix.as_ref()),
    ]
    .into_iter()
    .collect();

    Ok((dir.to_owned(), file_name))
}
