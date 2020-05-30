use crate::{error::Error, markers::GenericRecord, protos::Example};
use async_std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    sync, task,
};
use flurry::HashMap;
use futures::{
    future,
    io::{AsyncRead, AsyncSeek, AsyncSeekExt},
    stream::{self, Stream, StreamExt, TryStreamExt},
};
use std::{
    cmp::Ordering,
    future::Future,
    io::{prelude::*, SeekFrom},
    marker::PhantomData,
    num::NonZeroUsize,
    sync::{atomic::AtomicUsize, Arc},
};

// pub type BytesIndexedReader<R> = IndexedReader<Vec<u8>, R>;
// pub type ExampleIndexedReader<R> = IndexedReader<Example, R>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordIndex {
    pub path: Arc<PathBuf>,
    pub offset: u64,
    pub len: usize,
}

impl RecordIndex {
    fn compare(&self, rhs: &Self) -> Ordering {
        self.path
            .cmp(&rhs.path)
            .then_with(|| self.offset.cmp(&rhs.offset))
    }
}

impl PartialOrd for RecordIndex {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.compare(rhs))
    }
}

impl Ord for RecordIndex {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.compare(rhs)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DatasetInit {
    pub check_integrity: bool,
    pub max_open_files: Option<NonZeroUsize>,
    pub max_workers: Option<NonZeroUsize>,
    pub follow_symlink: bool,
}

impl DatasetInit {
    pub async fn from_prefix<P>(self, prefix: P) -> Result<Dataset, Error>
    where
        P: AsRef<Path>,
    {
        let follow_symlink = self.follow_symlink;
        let prefix = prefix.as_ref();
        let file_name_prefix = match prefix.file_name() {
            Some(pref) => pref,
            None => {
                return self.from_dir(prefix, false).await;
            }
        };

        let dir = prefix.parent().unwrap();
        let paths = dir
            .read_dir()
            .await?
            .try_filter_map(|entry| {
                async move {
                    let file_name = entry.file_name();
                    if !PathBuf::from(file_name).starts_with(file_name_prefix) {
                        return Ok(None);
                    }

                    let path = entry.path();
                    let metadata = if follow_symlink {
                        entry.metadata().await?
                    } else {
                        async_std::fs::symlink_metadata(&path).await?
                    };
                    if !metadata.is_file() {
                        return Ok(None);
                    }

                    Ok(Some(path))
                }
            })
            .try_collect::<Vec<_>>()
            .await?;

        self.from_paths(&paths).await
    }

    pub async fn from_dir<P>(self, dir: P, recursive: bool) -> Result<Dataset, Error>
    where
        P: AsRef<Path>,
    {
        let dir = dir.as_ref();
        let follow_symlink = self.follow_symlink;
        let num_workers = self
            .max_workers
            .map(|num| num.get())
            .unwrap_or_else(|| num_cpus::get());
        let (tx, rx) = sync::channel::<PathBuf>(num_workers);

        // workers for dir traversal
        let worker_futures = (0..num_workers)
            .map(|_| {
                let tx = tx.clone();
                let rx = rx.clone();
                let mut paths = vec![];

                async move {
                    while let Ok(path) = rx.recv().await {
                        let metadata = if follow_symlink {
                            path.metadata().await?
                        } else {
                            path.symlink_metadata().await?
                        };
                        let file_type = metadata.file_type();

                        if file_type.is_dir() {
                            if recursive {
                                async_std::fs::read_dir(&path)
                                    .await?
                                    .try_for_each(|entry| {
                                        let tx = tx.clone();
                                        async move {
                                            tx.send(entry.path()).await;
                                            Ok(())
                                        }
                                    })
                                    .await?;
                            }
                        } else if file_type.is_file() {
                            paths.push(path);
                        }
                    }

                    Result::<_, Error>::Ok(paths)
                }
            })
            .map(task::spawn)
            .collect::<Vec<_>>();

        // enumerate files from root directory
        let master_future = async move {
            let is_dir = if follow_symlink {
                dir.metadata().await?.is_dir()
            } else {
                dir.symlink_metadata().await?.is_dir()
            };

            if !is_dir {
                return Err(Error::InvalidArgumentsError {
                    desc: format!("{} is not a directory", dir.display()),
                });
            }

            dir.read_dir()
                .await?
                .try_for_each(|entry| {
                    let tx = tx.clone();
                    async move {
                        tx.send(entry.path()).await;
                        Ok(())
                    }
                })
                .await?;

            Ok(())
        };

        // collect paths from workers
        let (_, paths_per_worker) =
            futures::try_join!(master_future, future::try_join_all(worker_futures),)?;
        let paths = paths_per_worker
            .into_iter()
            .flat_map(|paths| paths)
            .collect::<Vec<_>>();

        // build dataset
        self.from_paths(&paths).await
    }

    pub async fn from_paths<P>(self, paths: &[P]) -> Result<Dataset, Error>
    where
        P: AsRef<Path>,
    {
        let check_integrity = self.check_integrity;
        let max_workers = self
            .max_workers
            .map(|num| num.get())
            .unwrap_or_else(|| num_cpus::get());

        // sorted paths
        let paths = {
            let mut paths = stream::iter(paths)
                .map(AsRef::as_ref)
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
                .await;
            paths.sort();
            paths
        };

        let (tx, rx) = sync::channel(max_workers);

        // index building workers
        let worker_futures = (0..max_workers)
            .map(|_| {
                let rx = rx.clone();
                let mut record_indexes = vec![];

                async move {
                    while let Ok(path) = rx.recv().await {
                        let path = Arc::new(path);
                        let mut reader = BufReader::new(File::open(&*path).await?);
                        let indexes_iter =
                            crate::io::async_::try_build_record_index(&mut reader, check_integrity)
                                .await?
                                .into_iter()
                                .map(|(offset, len)| RecordIndex {
                                    path: Arc::clone(&path),
                                    offset,
                                    len,
                                });
                        record_indexes.extend(indexes_iter);
                    }

                    Result::<_, Error>::Ok(record_indexes)
                }
            })
            .map(task::spawn)
            .collect::<Vec<_>>();

        // the master that send paths to workers
        let master_future = async move {
            for path in paths.into_iter() {
                tx.send(path).await;
            }
            Result::<_, Error>::Ok(())
        };

        // collect indexes from workers
        let (_, record_indexes_per_worker) =
            futures::try_join!(master_future, future::try_join_all(worker_futures),)?;

        // sort indexes
        let record_indexes = {
            let mut indexes = record_indexes_per_worker
                .into_iter()
                .flat_map(|indexes| indexes)
                .collect::<Vec<_>>();
            indexes.sort();
            indexes
        };

        // build dataset
        let dataset = {
            let DatasetInit { max_open_files, .. } = self;

            Dataset {
                state: Arc::new(DatasetState {
                    record_indexes,
                    max_workers,
                    max_open_files: max_open_files.map(|num| num.get()),
                }),
            }
        };

        Ok(dataset)
    }
}

impl Default for DatasetInit {
    fn default() -> Self {
        Self {
            check_integrity: true,
            max_workers: None,
            max_open_files: None,
            follow_symlink: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Dataset {
    state: Arc<DatasetState>,
}

impl Dataset {
    pub fn get<T>(&self, index: usize) -> Result<Option<T>, Error>
    where
        T: GenericRecord,
    {
        self.state.get(index)
    }

    pub fn iter<T>(&self) -> DatasetIter<T>
    where
        T: GenericRecord,
    {
        let state = self.state.clone();
        DatasetIter {
            state,
            index: 0,
            _phantom: PhantomData,
        }
    }

    // pub fn stream<T>(&self) -> DatasetStream<T>
    // where
    //     T: GenericRecord,
    // {
    //     let state = self.state.clone();
    //     DatasetStream {
    //         state,
    //         index: 0,
    //         _phantom: PhantomData,
    //     }
    // }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct DatasetState {
    record_indexes: Vec<RecordIndex>,
    max_workers: usize,
    max_open_files: Option<usize>,
}

impl DatasetState {
    pub fn get<T>(&self, index: usize) -> Result<Option<T>, Error>
    where
        T: GenericRecord,
    {
        let RecordIndex { path, offset, len } = match self.record_indexes.get(index) {
            Some(index) => index,
            None => return Ok(None),
        };
        todo!();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DatasetIter<T>
where
    T: GenericRecord,
{
    state: Arc<DatasetState>,
    index: usize,
    _phantom: PhantomData<T>,
}

impl<T> Iterator for DatasetIter<T>
where
    T: GenericRecord,
{
    type Item = Result<T, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.state.record_indexes.get(self.index)?;
        let item = self.state.get(self.index).transpose();
        self.index += 1;
        item
    }
}

#[derive(Debug)]
struct FileSet<R, F, Fut, E>
where
    R: AsyncRead + Send,
    F: Fn() -> Fut,
    Fut: Future<Output = Result<R, E>>,
{
    max_open_files: Option<usize>,
    num_open_files: Arc<AtomicUsize>,
    idle: HashMap<PathBuf, Arc<R>>,
    open_fn: F,
}

impl<R, F, Fut, E> FileSet<R, F, Fut, E>
where
    R: AsyncRead + Send,
    F: Fn() -> Fut,
    Fut: Future<Output = Result<R, E>>,
{
    pub fn new(max_open_files: Option<usize>, open_fn: F) -> Self {
        Self {
            max_open_files,
            num_open_files: Arc::new(AtomicUsize::new(0)),
            idle: HashMap::new(),
            open_fn,
        }
    }

    pub fn open<P>(path: P) -> Arc<R> {
        todo!();
    }
}

static_assertions::assert_impl_all!(Dataset: Send, Sync);
