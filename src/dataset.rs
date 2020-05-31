#![cfg(feature = "dataset")]

use crate::error::Error;
use async_std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};
use futures::{
    io::{AsyncReadExt, AsyncSeekExt},
    stream::{Stream, StreamExt, TryStream, TryStreamExt},
};
use std::{io::SeekFrom, num::NonZeroUsize, sync::Arc};
use tokio::sync::Semaphore;

struct RecordIndex {
    path: PathBuf,
    offset: u64,
    len: usize,
}

pub struct DatasetInit {
    pub check_integrity: bool,
    pub max_open_files: Option<NonZeroUsize>,
    pub max_workers: Option<NonZeroUsize>,
}

impl DatasetInit {
    pub async fn from_paths<P>(self, paths: &[P]) -> Result<Dataset, Error>
    where
        P: AsRef<Path>,
    {
        let Self {
            check_integrity,
            max_open_files,
            max_workers,
        } = self;

        let max_open_files = max_open_files.map(|num| num.get());
        let max_workers = max_workers
            .map(|num| num.get())
            .unwrap_or_else(|| num_cpus::get());
        let open_file_semaphore = Arc::new(max_open_files.map(|num| Semaphore::new(num)));

        // build record index
        let record_indexes = {
            let future_iter = paths
                .iter()
                .map(|path| path.as_ref().to_owned())
                .map(|path| {
                    let open_file_semaphore = Arc::clone(&open_file_semaphore);

                    async move {
                        // acquire open file permission
                        let permit = match &*open_file_semaphore {
                            Some(semaphore) => Some(semaphore.acquire().await),
                            None => None,
                        };

                        let index_stream = {
                            let reader = BufReader::new(File::open(&path).await?);
                            record_index_stream(reader, check_integrity).map_ok(
                                move |(offset, len)| RecordIndex {
                                    path: path.clone(),
                                    offset,
                                    len,
                                },
                            )
                        };

                        // let indexes = index_stream.try_collect::<Vec<_>>().await?;
                        // Result::<_, Error>::Ok(indexes)
                        Result::<_, Error>::Ok(index_stream)
                    }
                })
                .map(async_std::task::spawn);
            let future_stream = futures::stream::iter(future_iter).buffered(max_workers);
            let indexes = future_stream
                .try_flatten()
                .try_collect::<Vec<RecordIndex>>()
                .await?;
            indexes
        };

        let dataset = Dataset {
            state: Arc::new(DatasetState {
                record_indexes,
                max_workers,
                open_file_semaphore,
            }),
        };

        Ok(dataset)
    }
}

pub struct DatasetState {
    record_indexes: Vec<RecordIndex>,
    max_workers: usize,
    open_file_semaphore: Arc<Option<Semaphore>>,
}

pub struct Dataset {
    state: Arc<DatasetState>,
}

static_assertions::assert_impl_all!(Dataset: Send, Sync);

fn record_index_stream<R>(
    reader: R,
    check_integrity: bool,
) -> impl TryStream<Ok = (u64, usize), Error = Error>
where
    R: AsyncReadExt + AsyncSeekExt + Unpin,
{
    futures::stream::try_unfold((reader, check_integrity), |args| {
        async move {
            let (mut reader, check_integrity) = args;

            let len = match crate::io::async_::try_read_len(&mut reader, check_integrity).await? {
                Some(len) => len,
                None => return Ok(None),
            };

            let offset = reader.seek(SeekFrom::Current(0)).await?;
            crate::io::async_::try_read_record_data(&mut reader, len, check_integrity).await?;

            let index = (offset, len);
            let args = (reader, check_integrity);
            Result::<_, Error>::Ok(Some((index, args)))
        }
    })
}
