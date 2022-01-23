use super::{Position, RecordIndex, RecordIndexerConfig};
use crate::{
    error::{Error, Result},
    markers::GenericRecord,
    utils,
};
use async_std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};
use futures::{
    io::{AsyncRead, AsyncSeek, AsyncSeekExt as _},
    stream,
    stream::{Stream, StreamExt as _, TryStreamExt as _},
};
use std::{borrow::Cow, future::Future, io::SeekFrom, sync::Arc};

impl RecordIndex {
    /// Load the record data for the index.
    pub async fn load_async<T>(&self) -> Result<T>
    where
        T: GenericRecord,
    {
        let Self {
            ref path,
            offset,
            len,
        } = *self;
        let mut reader = BufReader::new(File::open(&**path).await?);
        let bytes = read_record_at(&mut reader, offset, len).await?;
        let record = T::from_bytes(bytes)?;
        Ok(record)
    }
}

/// Load record indexes from files specified by a prefix.
pub async fn load_prefix_async<'a, P>(
    prefix: P,
    config: RecordIndexerConfig,
) -> Result<impl Stream<Item = Result<RecordIndex>>>
where
    P: Into<Cow<'a, str>>,
{
    let stream = load_prefix_futures(prefix, config)
        .await?
        .then(|fut| fut)
        .try_flatten();
    Ok(stream)
}

/// Generate futures that load record indexes from files specified by a prefix.
pub async fn load_prefix_futures<'a, P>(
    prefix: P,
    config: RecordIndexerConfig,
) -> Result<impl Stream<Item = impl Future<Output = Result<impl Stream<Item = Result<RecordIndex>>>>>>
where
    P: Into<Cow<'a, str>>,
{
    let (dir, file_name_prefix) = utils::split_prefix(prefix);
    let dir = Path::new(&dir);
    let file_name_prefix = Arc::new(file_name_prefix);

    // filter paths
    let mut paths: Vec<_> = dir
        .read_dir()
        .await?
        .map(|result| result.map_err(Error::from))
        .try_filter_map(|entry| {
            let file_name_prefix = file_name_prefix.clone();

            async move {
                if !entry.metadata().await?.is_file() {
                    return Ok(None);
                }
                let file_name = PathBuf::from(entry.file_name());
                let path = file_name
                    .starts_with(&*file_name_prefix)
                    .then(|| std::path::PathBuf::from(entry.path().into_os_string()));
                Ok(path)
            }
        })
        .try_collect()
        .await?;

    // sort paths
    // TODO: fix blocking?
    paths.sort();

    // construct dataset
    let stream = load_paths_futures(paths, config);
    Ok(stream)
}

/// Load record indexes from file paths.
pub fn load_paths_async<'a, P, I>(
    paths: I,
    config: RecordIndexerConfig,
) -> impl Stream<Item = Result<RecordIndex>>
where
    I: IntoIterator<Item = P>,
    P: Into<Cow<'a, std::path::Path>>,
{
    load_paths_futures(paths, config)
        .then(|fut| fut)
        .try_flatten()
}

/// Generate futures that load record indexes from file paths.
pub fn load_paths_futures<'a, P, I>(
    paths: I,
    config: RecordIndexerConfig,
) -> impl Stream<Item = impl Future<Output = Result<impl Stream<Item = Result<RecordIndex>>>>>
where
    I: IntoIterator<Item = P>,
    P: Into<Cow<'a, std::path::Path>>,
{
    stream::iter(paths)
        .map(|path| path.into().into_owned())
        .map(move |path| load_file_async(path, config.clone()))
}

/// Load record indexes from a file.
pub async fn load_file_async<'a, P>(
    file: P,
    config: RecordIndexerConfig,
) -> Result<impl Stream<Item = Result<RecordIndex>>>
where
    P: Into<Cow<'a, std::path::Path>>,
{
    let file = file.into().into_owned();
    let reader = BufReader::new(File::open(&file).await?);

    let file = Arc::new(std::path::PathBuf::from(file.into_os_string()));
    let stream = load_reader_async(reader, config).map(move |pos| {
        let Position { offset, len } = pos?;
        Ok(RecordIndex {
            path: file.clone(),
            offset,
            len,
        })
    });
    Ok(stream)
}

/// Load record indexes from a reader.
pub fn load_reader_async<R>(
    reader: R,
    config: RecordIndexerConfig,
) -> impl Stream<Item = Result<Position>>
where
    R: AsyncRead + AsyncSeek + Unpin,
{
    let RecordIndexerConfig { check_integrity } = config;

    stream::try_unfold(reader, move |mut reader| async move {
        let len = match crate::io::r#async::try_read_len(&mut reader, check_integrity).await? {
            Some(len) => len,
            None => return Ok(None),
        };

        let offset = reader.seek(SeekFrom::Current(0)).await?;
        skip_or_check(&mut reader, len, check_integrity).await?;

        let pos = Position { offset, len };
        Result::<_, Error>::Ok(Some((pos, reader)))
    })
}

async fn read_record_at<R>(reader: &mut R, offset: u64, len: usize) -> Result<Vec<u8>>
where
    R: AsyncRead + AsyncSeek + Unpin,
{
    reader.seek(SeekFrom::Start(offset)).await?;
    let bytes = crate::io::r#async::try_read_record_data(reader, len, false).await?;

    Ok(bytes)
}

async fn skip_or_check<R>(reader: &mut R, len: usize, check_integrity: bool) -> Result<()>
where
    R: AsyncRead + AsyncSeek + Unpin,
{
    if check_integrity {
        crate::io::r#async::try_read_record_data(reader, len, check_integrity).await?;
    } else {
        reader.seek(SeekFrom::Current(len as i64)).await?;
    }
    Ok(())
}
