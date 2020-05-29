use crate::{error::Error, markers::GenericRecord, protos::Example};
use futures::{
    io::{AsyncRead, AsyncSeek, AsyncSeekExt},
    stream::Stream,
};
use std::{
    io::{prelude::*, SeekFrom},
    marker::PhantomData,
    path::Path,
};

pub type BytesReader<R> = RecordReader<Vec<u8>, R>;
pub type ExampleReader<R> = RecordReader<Example, R>;
pub type BytesIndexedReader<R> = IndexedReader<Vec<u8>, R>;
pub type ExampleIndexedReader<R> = IndexedReader<Example, R>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordIndex {
    pub offset: u64,
    pub len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordReaderInit {
    pub check_integrity: bool,
}

impl RecordReaderInit {
    pub fn from_reader<T, R>(self, reader: R) -> Result<RecordReader<T, R>, Error>
    where
        T: GenericRecord,
        R: Read,
    {
        let RecordReaderInit { check_integrity } = self;

        let record_reader = RecordReader {
            reader_opt: Some(reader),
            check_integrity,
            _phantom: PhantomData,
        };
        Ok(record_reader)
    }

    pub fn open<T, P>(
        self,
        path: P,
    ) -> Result<RecordReader<T, std::io::BufReader<std::fs::File>>, Error>
    where
        T: GenericRecord,
        P: AsRef<Path>,
    {
        use std::{fs::File, io::BufReader};
        let reader = BufReader::new(File::open(path.as_ref())?);
        let record_reader = self.from_reader(reader)?;
        Ok(record_reader)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordReader<T, R>
where
    T: GenericRecord,
    R: Read,
{
    check_integrity: bool,
    reader_opt: Option<R>,
    _phantom: PhantomData<T>,
}

impl<T, R> Iterator for RecordReader<T, R>
where
    T: GenericRecord,
    R: Read,
{
    type Item = Result<T, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let reader = self.reader_opt.as_mut()?;
        let bytes_opt: Option<Result<_, _>> =
            crate::io::blocking::try_read_record(reader, self.check_integrity).transpose();

        if let None = bytes_opt {
            self.reader_opt = None;
        }

        let bytes_result = bytes_opt?;
        let record_result = match bytes_result {
            Ok(bytes) => T::from_bytes(bytes),
            Err(err) => Err(err),
        };
        Some(record_result)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordStreamInit {
    pub check_integrity: bool,
}

impl RecordStreamInit {
    pub async fn from_reader<T, R>(
        self,
        reader: R,
    ) -> Result<impl Stream<Item = Result<T, Error>>, Error>
    where
        T: GenericRecord,
        R: 'static + AsyncRead + Unpin + Send,
    {
        let RecordStreamInit { check_integrity } = self;

        let stream = futures::stream::unfold(Some((reader, check_integrity)), |state_opt| {
            async move {
                let (mut reader, check_integrity) = state_opt?;
                let result = crate::io::async_::try_read_record(&mut reader, check_integrity)
                    .await
                    .transpose()?;
                let result = match result {
                    Ok(bytes) => T::from_bytes(bytes),
                    Err(err) => Err(err),
                };

                match result {
                    Ok(bytes) => Some((Ok(bytes), Some((reader, check_integrity)))),
                    Err(err) => Some((Err(err), None)),
                }
            }
        });

        Ok(stream)
    }

    pub async fn open<T, P>(self, path: P) -> Result<impl Stream<Item = Result<T, Error>>, Error>
    where
        T: GenericRecord,
        P: AsRef<async_std::path::Path>,
    {
        use async_std::{fs::File, io::BufReader};
        let reader = BufReader::new(File::open(path).await?);
        Self::from_reader(self, reader).await
    }

    pub async fn bytes_from_reader<R>(
        self,
        reader: R,
    ) -> Result<impl Stream<Item = Result<Vec<u8>, Error>>, Error>
    where
        R: 'static + AsyncRead + Unpin + Send,
    {
        self.from_reader::<Vec<u8>, _>(reader).await
    }

    pub async fn examples_from_reader<R>(
        self,
        reader: R,
    ) -> Result<impl Stream<Item = Result<Example, Error>>, Error>
    where
        R: 'static + AsyncRead + Unpin + Send,
    {
        self.from_reader::<Example, _>(reader).await
    }

    pub async fn bytes_open<P>(
        self,
        path: P,
    ) -> Result<impl Stream<Item = Result<Vec<u8>, Error>>, Error>
    where
        P: AsRef<async_std::path::Path>,
    {
        Self::open::<Vec<u8>, _>(self, path).await
    }

    pub async fn examples_open<P>(
        self,
        path: P,
    ) -> Result<impl Stream<Item = Result<Example, Error>>, Error>
    where
        P: AsRef<async_std::path::Path>,
    {
        Self::open::<Example, _>(self, path).await
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndexedReaderInit {
    pub check_integrity: bool,
}

impl IndexedReaderInit {
    pub fn from_reader<T, R>(self, mut reader: R) -> Result<IndexedReader<T, R>, Error>
    where
        T: GenericRecord,
        R: Read + Seek,
    {
        let IndexedReaderInit { check_integrity } = self;
        let indexes = crate::io::blocking::try_build_record_index(&mut reader, check_integrity)?;

        let indexed_reader = IndexedReader {
            indexes,
            reader,
            _phantom: PhantomData,
        };
        Ok(indexed_reader)
    }

    pub fn open<T, P>(
        self,
        path: P,
    ) -> Result<IndexedReader<T, std::io::BufReader<std::fs::File>>, Error>
    where
        T: GenericRecord,
        P: AsRef<Path>,
    {
        use std::{fs::File, io::BufReader};
        let reader = BufReader::new(File::open(path.as_ref())?);
        let indexed_reader = self.from_reader(reader)?;
        Ok(indexed_reader)
    }

    pub async fn from_async_reader<T, R>(self, mut reader: R) -> Result<IndexedReader<T, R>, Error>
    where
        T: GenericRecord,
        R: AsyncRead + AsyncSeek + Unpin,
    {
        let IndexedReaderInit { check_integrity } = self;
        let indexes =
            crate::io::async_::try_build_record_index(&mut reader, check_integrity).await?;

        let indexed_reader = IndexedReader {
            indexes,
            reader,
            _phantom: PhantomData,
        };
        Ok(indexed_reader)
    }

    pub async fn open_async<T, P>(
        self,
        path: P,
    ) -> Result<IndexedReader<T, async_std::io::BufReader<async_std::fs::File>>, Error>
    where
        T: GenericRecord,
        P: AsRef<async_std::path::Path>,
    {
        use async_std::{fs::File, io::BufReader};
        let reader = BufReader::new(File::open(path.as_ref()).await?);
        let indexed_reader = self.from_async_reader(reader).await?;
        Ok(indexed_reader)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndexedReader<T, R>
where
    T: GenericRecord,
{
    indexes: Vec<RecordIndex>,
    reader: R,
    _phantom: PhantomData<T>,
}

impl<T, R> IndexedReader<T, R>
where
    T: GenericRecord,
{
    pub fn num_records(&self) -> usize {
        self.indexes.len()
    }

    pub fn indexes(&self) -> &[RecordIndex] {
        self.indexes.as_slice()
    }
}

impl<T, R> IndexedReader<T, R>
where
    T: GenericRecord,
    R: Read + Seek,
{
    pub fn get(&mut self, index: usize) -> Result<Option<T>, Error> {
        let RecordIndex { offset, len } = *match self.indexes.get(index) {
            Some(record_index) => record_index,
            None => return Ok(None),
        };
        self.reader.seek(SeekFrom::Start(offset))?;
        let bytes = crate::io::blocking::try_read_record_data(&mut self.reader, len, false)?;
        let record = T::from_bytes(bytes)?;
        Ok(Some(record))
    }
}

impl<T, R> IndexedReader<T, R>
where
    T: GenericRecord,
    R: AsyncRead + AsyncSeekExt + Unpin,
{
    pub async fn get_async(&mut self, index: usize) -> Result<Option<T>, Error> {
        let RecordIndex { offset, len } = *match self.indexes.get(index) {
            Some(record_index) => record_index,
            None => return Ok(None),
        };
        self.reader.seek(SeekFrom::Start(offset)).await?;
        let bytes = crate::io::async_::try_read_record_data(&mut self.reader, len, false).await?;
        let record = T::from_bytes(bytes)?;
        Ok(Some(record))
    }
}
