//! Reading TFRecord data format.
//!
//! The module provides both blocking [RecordReaderInit] and
//! asynchronous [RecordStreamInit] reader initializers to build
//! reader and stream types.
//!
//! The [RecordReader] type, constructed by [RecordReaderInit],
//! implements the [Iterator](std::iter::Iterator) such that you can work with loops.
//!
//! The [RecordStreamInit] initializer constructs streams from types with [AsyncRead](AsyncRead) trait.
//! The streams can integrated with [futures::stream] API.

use crate::{
    error::Error,
    markers::GenericRecord,
    protos::{Event, Example as RawExample},
    types::Example,
};
#[cfg(feature = "async_")]
use futures::{io::AsyncRead, stream::Stream};
use std::{io::prelude::*, marker::PhantomData, path::Path};

/// Alias to [RecordReader] which output record type is [Vec\<u8\>](Vec).
pub type BytesReader<R> = RecordReader<Vec<u8>, R>;
/// Alias to [RecordReader] which output record type is [RawExample](RawExample).
pub type RawExampleReader<R> = RecordReader<RawExample, R>;
/// Alias to [RecordReader] which output record type is [Example](Example).
pub type ExampleReader<R> = RecordReader<Example, R>;
/// Alias to [RecordReader] which output record type is [Event](Event).
pub type EventReader<R> = RecordReader<Event, R>;

#[cfg(feature = "async_")]
pub use async_::*;
pub use blocking::*;

mod blocking {
    use super::*;

    /// The reader initializer.
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct RecordReaderInit {
        pub check_integrity: bool,
    }

    impl Default for RecordReaderInit {
        fn default() -> Self {
            Self {
                check_integrity: true,
            }
        }
    }

    impl RecordReaderInit {
        /// Construct a [RecordReader] from a type implementing [Read](std::io::Read).
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

        /// Construct a [RecordReader] from a path.
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

    /// The generic reader type.
    ///
    /// We suggest type alias [BytesReader], [RawExampleReader] and [ExampleReader]
    /// for convenience. Otherwise you can fill the type parameters as
    /// `RecordReader<OutputType, ReaderType>` manually to obtain the complete type.
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
}

#[cfg(feature = "async_")]
mod async_ {
    use super::*;

    /// The stream initializer.
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct RecordStreamInit {
        pub check_integrity: bool,
    }

    impl Default for RecordStreamInit {
        fn default() -> Self {
            Self {
                check_integrity: true,
            }
        }
    }

    impl RecordStreamInit {
        /// Build a stream from a reader type with [AsyncRead] trait.
        ///
        /// Specify the output type while calling this method. For example,
        /// `from_reader<Example, _>()`, or you can use [bytes_from_reader](RecordStreamInit::bytes_from_reader),
        /// [raw_examples_from_reader](RecordStreamInit::raw_examples_from_reader) and
        /// [examples_from_reader](RecordStreamInit::examples_from_reader) aliases.
        pub async fn from_reader<T, R>(
            self,
            reader: R,
        ) -> Result<impl Stream<Item = Result<T, Error>>, Error>
        where
            T: GenericRecord,
            R: 'static + AsyncRead + Unpin + Send,
        {
            let RecordStreamInit { check_integrity } = self;

            let stream =
                futures::stream::unfold(Some((reader, check_integrity)), |state_opt| async move {
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
                });

            Ok(stream)
        }

        /// Build a stream from a path.
        ///
        /// Specify the output type while calling this method. For example,
        /// `open<Example, _>()`, or you can use [bytes_open](RecordStreamInit::bytes_open),
        /// [raw_examples_open](RecordStreamInit::raw_examples_open) and
        /// [examples_open](RecordStreamInit::examples_open) aliases.
        pub async fn open<T, P>(
            self,
            path: P,
        ) -> Result<impl Stream<Item = Result<T, Error>>, Error>
        where
            T: GenericRecord,
            P: AsRef<async_std::path::Path>,
        {
            use async_std::{fs::File, io::BufReader};
            let reader = BufReader::new(File::open(path).await?);
            Self::from_reader(self, reader).await
        }

        /// Alias to [from_reader<Vec<u8>, R>](RecordStreamInit::from_reader).
        pub async fn bytes_from_reader<R>(
            self,
            reader: R,
        ) -> Result<impl Stream<Item = Result<Vec<u8>, Error>>, Error>
        where
            R: 'static + AsyncRead + Unpin + Send,
        {
            self.from_reader::<Vec<u8>, _>(reader).await
        }

        /// Alias to [from_reader<Example, R>](RecordStreamInit::from_reader).
        pub async fn raw_examples_from_reader<R>(
            self,
            reader: R,
        ) -> Result<impl Stream<Item = Result<RawExample, Error>>, Error>
        where
            R: 'static + AsyncRead + Unpin + Send,
        {
            self.from_reader::<RawExample, _>(reader).await
        }

        /// Alias to [from_reader<Example, R>](RecordStreamInit::from_reader).
        pub async fn examples_from_reader<R>(
            self,
            reader: R,
        ) -> Result<impl Stream<Item = Result<Example, Error>>, Error>
        where
            R: 'static + AsyncRead + Unpin + Send,
        {
            self.from_reader::<Example, _>(reader).await
        }

        /// Alias to [open<Vec<u8>, R>](RecordStreamInit::open).
        pub async fn bytes_open<P>(
            self,
            path: P,
        ) -> Result<impl Stream<Item = Result<Vec<u8>, Error>>, Error>
        where
            P: AsRef<async_std::path::Path>,
        {
            Self::open::<Vec<u8>, _>(self, path).await
        }

        /// Alias to [open<Example, R>](RecordStreamInit::open).
        pub async fn raw_examples_open<P>(
            self,
            path: P,
        ) -> Result<impl Stream<Item = Result<RawExample, Error>>, Error>
        where
            P: AsRef<async_std::path::Path>,
        {
            Self::open::<RawExample, _>(self, path).await
        }

        /// Alias to [open<Example, R>](RecordStreamInit::open).
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
}
