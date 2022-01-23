use super::{Position, RecordIndex, RecordIndexerConfig};
use crate::{
    error::{Error, Result},
    markers::GenericRecord,
    utils,
};
use itertools::Itertools as _;
use std::{
    borrow::Cow,
    fs::File,
    io::{prelude::*, BufReader, SeekFrom},
    path::{Path, PathBuf},
    sync::Arc,
};

impl RecordIndex {
    pub fn load<T>(&self) -> Result<T>
    where
        T: GenericRecord,
    {
        let Self {
            ref path,
            offset,
            len,
        } = *self;
        let mut reader = BufReader::new(File::open(&**path)?);
        let bytes = read_record_at(&mut reader, offset, len)?;
        let record = T::from_bytes(bytes)?;
        Ok(record)
    }
}

/// Open TFRecord files by a path prefix.
///
/// If the path ends with "/", it searchs for all files under the directory.
/// Otherwise, it lists the files with the path prefix.
/// The enumerated paths will be sorted in alphabetical order.
pub fn load_prefix<'a, P>(
    prefix: P,
    config: RecordIndexerConfig,
) -> Result<impl Iterator<Item = Result<RecordIndex>>>
where
    P: Into<Cow<'a, str>>,
{
    let (dir, file_name_prefix) = utils::split_prefix(prefix);
    let dir = Path::new(&dir);
    let file_name_prefix = Arc::new(file_name_prefix);

    // filter paths
    let mut paths: Vec<_> = dir
        .read_dir()?
        .map(|result| result.map_err(Error::from))
        .filter_map(move |entry| {
            let file_name_prefix = file_name_prefix.clone();

            (move || -> Result<_> {
                let entry = entry?;
                if !entry.metadata()?.is_file() {
                    return Ok(None);
                }
                let file_name = PathBuf::from(entry.file_name());
                let path = file_name
                    .starts_with(&*file_name_prefix)
                    .then(|| entry.path());
                Ok(path)
            })()
            .transpose()
        })
        .try_collect()?;

    // sort paths
    // TODO: fix blocking?
    paths.sort();

    // construct dataset
    let indexes = load_paths(paths, config);
    Ok(indexes)
}

/// Open TFRecord files by a set of path.
///
/// It assumes every path is a TFRecord file, otherwise it returns error.
/// The order of the paths affects the order of record indexes..
pub fn load_paths<'a, P, I>(
    paths: I,
    config: RecordIndexerConfig,
) -> impl Iterator<Item = Result<RecordIndex>>
where
    I: IntoIterator<Item = P>,
    P: Into<Cow<'a, Path>>,
{
    paths
        .into_iter()
        .map(|path| path.into().into_owned())
        .map(move |path| load_file(path, config.clone()))
        .map(
            |iter| -> Box<dyn Iterator<Item = Result<RecordIndex>> + Send> {
                match iter {
                    Ok(iter) => Box::new(iter),
                    Err(err) => Box::new([Err(err)].into_iter()),
                }
            },
        )
        .flatten()
}

pub fn load_file<'a, P>(
    file: P,
    config: RecordIndexerConfig,
) -> Result<impl Iterator<Item = Result<RecordIndex>>>
where
    P: Into<Cow<'a, Path>>,
{
    let file = file.into().into_owned();
    let reader = BufReader::new(File::open(&file)?);
    let file = Arc::new(file);
    let iter = load_reader(reader, config).map(move |pos| {
        let Position { offset, len } = pos?;
        Ok(RecordIndex {
            path: file.clone(),
            offset,
            len,
        })
    });
    Ok(iter)
}

pub fn load_reader<R>(
    reader: R,
    config: RecordIndexerConfig,
) -> impl Iterator<Item = Result<Position>>
where
    R: Read + Seek,
{
    let RecordIndexerConfig { check_integrity } = config;

    itertools::unfold(Some(reader), move |reader_opt| {
        let mut reader = reader_opt.as_mut()?;
        let len = match crate::io::sync::try_read_len(&mut reader, check_integrity).transpose()? {
            Ok(len) => len,
            Err(err) => {
                *reader_opt = None;
                return Some(Err(err));
            }
        };

        let offset = (move || {
            let offset = reader.seek(SeekFrom::Current(0))?;
            skip_or_check(&mut reader, len, check_integrity)?;
            Ok(offset)
        })();
        let offset = match offset {
            Ok(offset) => offset,
            Err(err) => {
                *reader_opt = None;
                return Some(Err(err));
            }
        };

        let pos = Position { offset, len };
        Some(Ok(pos))
    })
}

fn read_record_at<R>(reader: &mut R, offset: u64, len: usize) -> Result<Vec<u8>>
where
    R: Read + Seek,
{
    reader.seek(SeekFrom::Start(offset))?;
    let bytes = crate::io::sync::try_read_record_data(reader, len, false)?;
    Ok(bytes)
}

fn skip_or_check<R>(reader: &mut R, len: usize, check_integrity: bool) -> Result<()>
where
    R: Read + Seek,
{
    if check_integrity {
        crate::io::sync::try_read_record_data(reader, len, check_integrity)?;
    } else {
        reader.seek(SeekFrom::Current(len as i64))?;
    }
    Ok(())
}
