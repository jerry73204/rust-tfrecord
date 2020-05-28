use crate::{
    error::{Error as CrateError, Result as CrateResult},
    protos::Example,
};
use byteorder::{LittleEndian, ReadBytesExt};
use crc::crc32;
// use failure::Fallible;
// use log::debug;
use prost::Message;
use std::{
    fs::File,
    io::{prelude::*, BufReader, SeekFrom},
    marker::PhantomData,
    path::Path,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordIndex {
    pub offset: u64,
    pub len: usize,
}

pub trait CheckIntegrity {
    fn check_integrity(enable: Option<bool>, buf: &[u8], expect: u32) -> CrateResult<()>;
    fn is_dynamic() -> bool;
    fn do_check() -> Option<bool>;
}

pub struct DoCheck;

impl CheckIntegrity for DoCheck {
    fn check_integrity(enable: Option<bool>, buf: &[u8], expect: u32) -> CrateResult<()> {
        assert_eq!(None, enable, "please report this bug");

        let found = checksum(&buf);
        if expect == found {
            Ok(())
        } else {
            Err(CrateError::ChecksumMismatchError {
                expect: format!("{:#010x}", expect),
                found: format!("{:#010x}", found),
            })
        }
    }

    fn is_dynamic() -> bool {
        false
    }

    fn do_check() -> Option<bool> {
        Some(true)
    }
}

pub struct NoCheck;

impl CheckIntegrity for NoCheck {
    fn check_integrity(enable: Option<bool>, _buf: &[u8], _expect: u32) -> CrateResult<()> {
        assert_eq!(None, enable, "please report this bug");
        Ok(())
    }

    fn is_dynamic() -> bool {
        false
    }

    fn do_check() -> Option<bool> {
        Some(false)
    }
}

pub struct RuntimeCheck;

impl CheckIntegrity for RuntimeCheck {
    fn check_integrity(enable: Option<bool>, buf: &[u8], expect: u32) -> CrateResult<()> {
        if let Some(true) = enable {
            DoCheck::check_integrity(None, buf, expect)
        } else if let Some(false) = enable {
            Ok(())
        } else {
            panic!("please report this bug");
        }
    }

    fn is_dynamic() -> bool {
        true
    }

    fn do_check() -> Option<bool> {
        None
    }
}

fn checksum(buf: &[u8]) -> u32 {
    let cksum = crc32::checksum_castagnoli(buf);
    ((cksum >> 15) | (cksum << 17)).wrapping_add(0xa282ead8u32)
}

fn try_read_len<R, C>(reader: &mut R, check_integrity: Option<bool>) -> CrateResult<Option<u64>>
where
    R: Read,
    C: CheckIntegrity,
{
    let mut len_buf = [0u8; 8];

    match reader.read(&mut len_buf) {
        Ok(0) => Ok(None),
        Ok(n) if n == len_buf.len() => {
            let len = (&len_buf[..]).read_u64::<LittleEndian>()?;
            let expect_cksum = reader.read_u32::<LittleEndian>()?;
            C::check_integrity(check_integrity, &len_buf, expect_cksum)?;
            Ok(Some(len))
        }
        Ok(_) => Err(CrateError::UnexpectedEofError),
        Err(error) => Err(error.into()),
    }
}

fn try_read_record<R, C>(
    reader: &mut R,
    len: usize,
    check_integrity: Option<bool>,
) -> CrateResult<Vec<u8>>
where
    R: Read,
    C: CheckIntegrity,
{
    let mut buf = Vec::<u8>::new();
    buf.resize(len, 0);
    reader.read_exact(&mut buf)?;
    let expect_cksum = reader.read_u32::<LittleEndian>()?;
    C::check_integrity(check_integrity, &buf, expect_cksum)?;
    Ok(buf)
}

fn try_index_record<R, C>(
    reader: &mut R,
    len: usize,
    check_integrity: Option<bool>,
) -> CrateResult<RecordIndex>
where
    R: Read + Seek,
    C: CheckIntegrity,
{
    let offset = reader.seek(SeekFrom::Current(0))?;
    let index = RecordIndex { offset, len };

    match (C::do_check(), check_integrity) {
        (Some(true), None) | (None, Some(true)) => {
            let mut buf = Vec::<u8>::new();
            buf.resize(len, 0);
            reader.read_exact(&mut buf)?;
            let expect_cksum = reader.read_u32::<LittleEndian>()?;
            C::check_integrity(check_integrity, &buf, expect_cksum)?;
            Ok(index)
        }
        (Some(false), None) | (None, Some(false)) => {
            let new_offset = reader.seek(SeekFrom::Current(len as i64))?;
            if new_offset - offset == len as u64 {
                Ok(index)
            } else {
                Err(CrateError::UnexpectedEofError)
            }
        }
        _ => panic!("please report this bug"),
    }
}

pub struct ReaderOptions {
    check_integrity_opt: Option<bool>,
}

impl ReaderOptions {
    pub fn new() -> Self {
        Self {
            check_integrity_opt: None,
        }
    }

    pub fn check_integrity(mut self, enabled: bool) -> Self {
        self.check_integrity_opt = Some(enabled);
        self
    }

    pub fn record_reader_from_reader<C, R>(self, reader: R) -> CrateResult<RecordReader<R, C>>
    where
        C: CheckIntegrity,
        R: Read,
    {
        let Self {
            check_integrity_opt,
        } = self;

        if C::is_dynamic() {
            if let None = check_integrity_opt {
                let desc = "check_integrity() is not called".to_owned();
                return Err(CrateError::InvalidOptionsError { desc });
            }
        } else if let Some(_) = check_integrity_opt {
            let desc = "check_integrity() should not be called".to_owned();
            return Err(CrateError::InvalidOptionsError { desc });
        }

        let record_reader = RecordReader {
            reader_opt: Some(reader),
            check_integrity: check_integrity_opt,
            _phantom: PhantomData,
        };
        Ok(record_reader)
    }

    pub fn record_reader_open<C, P>(self, path: P) -> CrateResult<RecordReader<BufReader<File>, C>>
    where
        C: CheckIntegrity,
        P: AsRef<Path>,
    {
        let reader = BufReader::new(File::open(path.as_ref())?);
        let record_reader = self.record_reader_from_reader(reader)?;
        Ok(record_reader)
    }

    pub fn record_indexer_from_reader<C, R>(self, reader: R) -> CrateResult<RecordIndexer<R, C>>
    where
        C: CheckIntegrity,
        R: Read,
    {
        let Self {
            check_integrity_opt,
        } = self;

        if C::is_dynamic() {
            if let None = check_integrity_opt {
                let desc = "check_integrity() is not called".to_owned();
                return Err(CrateError::InvalidOptionsError { desc });
            }
        } else if let Some(_) = check_integrity_opt {
            let desc = "check_integrity() should not be called".to_owned();
            return Err(CrateError::InvalidOptionsError { desc });
        }

        let record_indexer = RecordIndexer {
            reader_opt: Some(reader),
            check_integrity: check_integrity_opt,
            _phantom: PhantomData,
        };
        Ok(record_indexer)
    }

    pub fn record_indexer_open<C, P>(
        self,
        path: P,
    ) -> CrateResult<RecordIndexer<BufReader<File>, C>>
    where
        C: CheckIntegrity,
        P: AsRef<Path>,
    {
        let reader = BufReader::new(File::open(path.as_ref())?);
        let record_indexer = self.record_indexer_from_reader(reader)?;
        Ok(record_indexer)
    }

    pub fn example_reader_from_reader<C, R>(self, reader: R) -> CrateResult<ExampleReader<R, C>>
    where
        C: CheckIntegrity,
        R: Read,
    {
        let record_reader = self.record_reader_from_reader(reader)?;
        let example_reader = ExampleReader { record_reader };
        Ok(example_reader)
    }

    pub fn example_reader_open<C, P>(
        self,
        path: P,
    ) -> CrateResult<ExampleReader<BufReader<File>, C>>
    where
        C: CheckIntegrity,
        P: AsRef<Path>,
    {
        let record_reader = self.record_reader_open(path)?;
        let example_reader = ExampleReader { record_reader };
        Ok(example_reader)
    }
}

pub struct RecordReader<R, C>
where
    R: Read,
    C: CheckIntegrity,
{
    check_integrity: Option<bool>,
    reader_opt: Option<R>,
    _phantom: PhantomData<C>,
}

impl<R, C> Iterator for RecordReader<R, C>
where
    R: Read,
    C: CheckIntegrity,
{
    type Item = CrateResult<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.reader_opt {
            Some(reader) => {
                let len = match try_read_len::<_, C>(reader, self.check_integrity) {
                    Ok(Some(len)) => len,
                    Ok(None) => {
                        self.reader_opt = None;
                        return None;
                    }
                    Err(error) => {
                        self.reader_opt = None;
                        return Some(Err(error));
                    }
                };

                let record =
                    try_read_record::<_, C>(reader, len as usize, self.check_integrity).unwrap();
                Some(Ok(record))
            }
            None => None,
        }
    }
}

pub struct ExampleReader<R, C>
where
    R: Read,
    C: CheckIntegrity,
{
    record_reader: RecordReader<R, C>,
}

impl<R, C> Iterator for ExampleReader<R, C>
where
    R: Read,
    C: CheckIntegrity,
{
    type Item = CrateResult<Example>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.record_reader.next() {
            Some(Ok(record)) => match Example::decode(record.as_ref()) {
                Ok(example) => Some(Ok(example)),
                Err(error) => Some(Err(error.into())),
            },
            Some(Err(error)) => Some(Err(error)),
            None => None,
        }
    }
}

pub struct RecordIndexer<R, C>
where
    R: Read,
    C: CheckIntegrity,
{
    check_integrity: Option<bool>,
    reader_opt: Option<R>,
    _phantom: PhantomData<C>,
}

impl<R, C> Iterator for RecordIndexer<R, C>
where
    R: Read + Seek,
    C: CheckIntegrity,
{
    type Item = CrateResult<RecordIndex>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.reader_opt {
            Some(reader) => {
                let len = match try_read_len::<_, C>(reader, self.check_integrity) {
                    Ok(Some(len)) => len,
                    Ok(None) => {
                        self.reader_opt = None;
                        return None;
                    }
                    Err(error) => {
                        self.reader_opt = None;
                        return Some(Err(error));
                    }
                };

                let index =
                    try_index_record::<_, C>(reader, len as usize, self.check_integrity).unwrap();
                Some(Ok(index))
            }
            None => None,
        }
    }
}
