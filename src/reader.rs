use crate::{error::Error, markers::GenericRecord, protos::Example, record::EasyExample};
use futures::{io::AsyncRead, stream::Stream};
use std::{io::prelude::*, marker::PhantomData, path::Path};

pub type BytesReader<R> = RecordReader<Vec<u8>, R>;
pub type ExampleReader<R> = RecordReader<Example, R>;
pub type EasyExampleReader<R> = RecordReader<EasyExample, R>;

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

    pub async fn easy_examples_from_reader<R>(
        self,
        reader: R,
    ) -> Result<impl Stream<Item = Result<EasyExample, Error>>, Error>
    where
        R: 'static + AsyncRead + Unpin + Send,
    {
        self.from_reader::<EasyExample, _>(reader).await
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

    pub async fn easy_examples_open<P>(
        self,
        path: P,
    ) -> Result<impl Stream<Item = Result<EasyExample, Error>>, Error>
    where
        P: AsRef<async_std::path::Path>,
    {
        Self::open::<EasyExample, _>(self, path).await
    }
}
