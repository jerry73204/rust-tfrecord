mod sync;
pub use sync::*;

#[cfg(feature = "async")]
mod r#async;
#[cfg(feature = "async")]
pub use r#async::*;

use crate::{error::Error, utils};
use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    path::PathBuf,
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

    // let (dir, file_name_prefix): (&Path, &OsStr) = if prefix.ends_with(MAIN_SEPARATOR) {
    //     let dir = Path::new(prefix.as_ref());
    //     (dir, OsStr::new(""))
    // } else {
    //     let prefix = Path::new(prefix.as_ref());
    //     let file_name_prefix = prefix.file_name().unwrap_or_else(|| OsStr::new(""));
    //     let dir = prefix.parent().unwrap(); // TODO
    //     (dir, file_name_prefix)
    // };
    let (dir, file_name_prefix) = utils::split_prefix(prefix);

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_micros();
    let host_name = hostname::get()?;
    let file_name: OsString = [
        &file_name_prefix,
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
