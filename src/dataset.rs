#![cfg(feature = "dataset")]

//! The dataset API that accesses multiple TFRecord files.
//!
//! The module is available when the `dataset` feature is enabled.
//! The [Dataset] type can be constructed using [DatasetInit] initializer.

use crate::{error::Error, markers::GenericRecord};
use async_std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf, MAIN_SEPARATOR},
};
use futures::{
    io::{AsyncReadExt, AsyncSeekExt},
    stream::{StreamExt, TryStream, TryStreamExt},
};
use std::{io::SeekFrom, mem, num::NonZeroUsize, sync::Arc};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RecordIndex {
    path: Arc<PathBuf>,
    offset: u64,
    len: usize,
}

/// The dataset initializer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DatasetInit {
    /// Verify the checksum or not.
    pub check_integrity: bool,
    /// Maximum number of open files.
    ///
    /// Limit the number of open files if it is `Some(_)`
    /// It has no limit if it is `None`.
    pub max_open_files: Option<NonZeroUsize>,
    /// Maximum number of concurrent workers.
    ///
    /// If it is `None`, it defaults to [num_cpus::get].
    pub max_workers: Option<NonZeroUsize>,
}

impl Default for DatasetInit {
    fn default() -> Self {
        Self {
            check_integrity: true,
            max_open_files: None,
            max_workers: None,
        }
    }
}

impl DatasetInit {
    /// Open TFRecord files by a path prefix.
    ///
    /// If the path ends with "/", it searchs for all files under the directory.
    /// Otherwise, it lists the files with the path prefix.
    /// The enumerated paths will be sorted in alphabetical order.
    pub async fn from_prefix(self, prefix: &str) -> Result<Dataset, Error> {
        // get parent dir and file name prefix
        let prefix_path: &Path = prefix.as_ref();

        // assume the prefix is a directly if it ends with the separator
        let (dir, file_name_prefix_opt) = if prefix.ends_with(MAIN_SEPARATOR) {
            (prefix_path, None)
        } else {
            let dir = prefix_path.parent().expect("please report bug");
            let file_name_prefix = prefix_path
                .file_name()
                .expect("please report bug")
                .to_str()
                .expect("please report bug");
            (dir, Some(file_name_prefix))
        };

        // filter paths
        let mut paths = dir
            .read_dir()
            .await?
            .map(|result| result.map_err(|err| Error::from(err)))
            .try_filter_map(|entry| {
                async move {
                    if !entry.metadata().await?.is_file() {
                        return Ok(None);
                    }

                    let path = entry.path();
                    let file_name =
                        entry
                            .file_name()
                            .into_string()
                            .map_err(|_| Error::UnicodeError {
                                desc: format!(
                                    r#"the file path "{}" is not Unicode"#,
                                    path.display()
                                ),
                            })?;

                    match file_name_prefix_opt {
                        Some(file_name_prefix) => {
                            if file_name.starts_with(&file_name_prefix) {
                                Result::<_, Error>::Ok(Some(path))
                            } else {
                                Ok(None)
                            }
                        }
                        None => Ok(Some(path)),
                    }
                }
            })
            .try_collect::<Vec<_>>()
            .await?;

        // sort paths
        paths.sort();

        // construct dataset
        self.from_paths(&paths).await
    }

    /// Open TFRecord files by a set of path.
    ///
    /// It assumes every path is a TFRecord file, otherwise it returns error.
    /// The order of the paths affects the order of record indexes..
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
        let open_file_semaphore = max_open_files.map(|num| Arc::new(Semaphore::new(num)));

        // build record index
        let record_indexes = {
            // spawn indexing worker per path
            let future_iter = paths
                .iter()
                .map(|path| Arc::new(path.as_ref().to_owned()))
                .map(|path| {
                    let open_file_semaphore = open_file_semaphore.clone();

                    async move {
                        // acquire open file permission
                        let permit = match open_file_semaphore {
                            Some(semaphore) => Some(Arc::new(semaphore.acquire_owned().await)),
                            None => None,
                        };

                        let index_stream = {
                            // open index stream
                            let reader = BufReader::new(File::open(&*path).await?);
                            let stream = record_index_stream(reader, check_integrity);

                            // add path to index
                            let stream = stream.map_ok(move |(offset, len)| RecordIndex {
                                path: Arc::clone(&path),
                                offset,
                                len,
                            });

                            // add semaphore permission
                            let stream = stream.map_ok(move |index| {
                                let permit_clone = permit.clone();
                                (permit_clone, index)
                            });

                            stream
                        };

                        Result::<_, Error>::Ok(index_stream)
                    }
                })
                .map(async_std::task::spawn);

            // limit workers by max_workers
            let future_stream = futures::stream::iter(future_iter).buffered(max_workers);

            // drop semaphore permission
            let indexes = future_stream
                .try_flatten()
                .map_ok(|(permit, index)| {
                    mem::drop(permit);
                    index
                })
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
            open_file: None,
        };

        Ok(dataset)
    }
}

#[derive(Debug)]
struct DatasetState {
    pub record_indexes: Vec<RecordIndex>,
    pub max_workers: usize,
    pub open_file_semaphore: Option<Arc<Semaphore>>,
}

/// The dataset type.
#[derive(Debug)]
pub struct Dataset {
    state: Arc<DatasetState>,
    open_file: Option<(PathBuf, BufReader<File>, Option<OwnedSemaphorePermit>)>,
}

impl Clone for Dataset {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            open_file: None,
        }
    }
}

impl Dataset {
    /// Get the number of indexed records.
    pub fn num_records(&self) -> usize {
        self.state.record_indexes.len()
    }

    /// Get an example by an index number.
    ///
    /// It returns `None` if the index number is greater than or equal to [num_records](Dataset::num_records).
    pub async fn get<T>(&mut self, index: usize) -> Result<Option<T>, Error>
    where
        T: GenericRecord,
    {
        // try to get record index
        let record_index = match self.state.record_indexes.get(index) {
            Some(record_index) => record_index.to_owned(),
            None => return Ok(None),
        };
        let RecordIndex { offset, len, path } = record_index;

        let reader = self.open_file(&*path).await?;
        let bytes = try_read_record_at(reader, offset, len).await?;
        let record = T::from_bytes(bytes)?;
        Ok(Some(record))
    }

    /// Gets the record stream.
    pub fn stream<T>(&self) -> impl TryStream<Ok = T, Error = Error> + Send
    where
        T: GenericRecord,
    {
        let dataset = self.clone();
        futures::stream::try_unfold((dataset, 0), |state| {
            async move {
                let (mut dataset, index) = state;
                Ok(dataset.get::<T>(index).await?.map(|record| {
                    let new_state = (dataset, index + 1);
                    (record, new_state)
                }))
            }
        })
    }

    async fn open_file<P>(&mut self, path: P) -> Result<&mut BufReader<File>, Error>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();

        // re-open file if path is distinct
        match self.open_file.take() {
            Some((opened_path, reader, permit)) if opened_path == path => {
                self.open_file = Some((opened_path, reader, permit));
            }
            args => {
                mem::drop(args); // drop previous permit and reader
                let semaphore_opt = self.state.open_file_semaphore.clone();
                let permit = match semaphore_opt {
                    Some(semaphore) => Some(semaphore.acquire_owned().await),
                    None => None,
                };
                let reader = BufReader::new(File::open(&path).await?);
                self.open_file = Some((path.to_owned(), reader, permit));
            }
        }

        Ok(&mut self.open_file.as_mut().unwrap().1)
    }
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

async fn try_read_record_at<R>(reader: &mut R, offset: u64, len: usize) -> Result<Vec<u8>, Error>
where
    R: AsyncReadExt + AsyncSeekExt + Unpin,
{
    reader.seek(SeekFrom::Start(offset)).await?;
    let bytes = crate::io::async_::try_read_record_data(reader, len, false).await?;

    Ok(bytes)
}
