use crate::{error::Error, reader::RecordIndex};
use byteorder::{LittleEndian, ReadBytesExt};
use crc::crc32;
use futures::io::{AsyncReadExt, AsyncSeekExt};
use std::io::{prelude::*, SeekFrom};

fn checksum(buf: &[u8]) -> u32 {
    let cksum = crc32::checksum_castagnoli(buf);
    ((cksum >> 15) | (cksum << 17)).wrapping_add(0xa282ead8u32)
}

fn verify_checksum(buf: &[u8], expect: u32) -> Result<(), Error> {
    let found = checksum(&buf);
    if expect == found {
        Ok(())
    } else {
        Err(Error::ChecksumMismatchError {
            expect: format!("{:#010x}", expect),
            found: format!("{:#010x}", found),
        })
    }
}

pub mod async_ {
    use super::*;

    pub async fn try_read_record<R>(
        reader: &mut R,
        check_integrity: bool,
    ) -> Result<Option<Vec<u8>>, Error>
    where
        R: AsyncReadExt + Unpin,
    {
        let len = match try_read_len(reader, check_integrity).await? {
            Some(len) => len,
            None => return Ok(None),
        };
        let data = try_read_record_data(reader, len, check_integrity).await?;
        Ok(Some(data))
    }

    pub async fn try_read_len<R>(
        reader: &mut R,
        check_integrity: bool,
    ) -> Result<Option<usize>, Error>
    where
        R: AsyncReadExt + Unpin,
    {
        let len_buf = {
            let mut len_buf = [0u8; std::mem::size_of::<u64>()];
            match reader.read(&mut len_buf).await {
                Ok(0) => return Ok(None),
                Ok(n) if n == len_buf.len() => (),
                Ok(_) => return Err(Error::UnexpectedEofError),
                Err(error) => return Err(error.into()),
            };
            len_buf
        };
        let len = (&len_buf[..]).read_u64::<LittleEndian>()?;

        let expect_cksum = {
            let mut buf = [0; std::mem::size_of::<u32>()];
            reader.read_exact(&mut buf).await?;
            (&buf[..]).read_u32::<LittleEndian>()?
        };

        if check_integrity {
            verify_checksum(&len_buf, expect_cksum)?;
        }

        Ok(Some(len as usize))
    }

    pub async fn try_read_record_data<R>(
        reader: &mut R,
        len: usize,
        check_integrity: bool,
    ) -> Result<Vec<u8>, Error>
    where
        R: AsyncReadExt + Unpin,
    {
        let buf = {
            let mut buf = vec![0u8; len];
            reader.read_exact(&mut buf).await?;
            buf
        };
        let expect_cksum = {
            let mut buf = [0u8; std::mem::size_of::<u32>()];
            reader.read_exact(&mut buf).await?;
            (&buf[..]).read_u32::<LittleEndian>()?
        };

        if check_integrity {
            verify_checksum(&buf, expect_cksum)?;
        }
        Ok(buf)
    }

    pub async fn try_build_record_index<R>(
        reader: &mut R,
        check_integrity: bool,
    ) -> Result<Vec<RecordIndex>, Error>
    where
        R: AsyncReadExt + AsyncSeekExt + Unpin,
    {
        let mut indexes = vec![];

        while let Some(len) = try_read_len(reader, check_integrity).await? {
            let offset = reader.seek(SeekFrom::Current(0)).await?;
            try_read_record_data(reader, len, check_integrity).await?;
            let index = RecordIndex { offset, len };
            indexes.push(index);
        }

        Ok(indexes)
    }
}

pub mod blocking {
    use super::*;

    pub fn try_read_record<R>(
        reader: &mut R,
        check_integrity: bool,
    ) -> Result<Option<Vec<u8>>, Error>
    where
        R: Read,
    {
        let len = match try_read_len(reader, check_integrity)? {
            Some(len) => len,
            None => return Ok(None),
        };
        let data = try_read_record_data(reader, len, check_integrity)?;
        Ok(Some(data))
    }

    pub fn try_read_len<R>(reader: &mut R, check_integrity: bool) -> Result<Option<usize>, Error>
    where
        R: Read,
    {
        let len_buf = {
            let mut len_buf = [0u8; std::mem::size_of::<u64>()];
            match reader.read(&mut len_buf) {
                Ok(0) => return Ok(None),
                Ok(n) if n == len_buf.len() => (),
                Ok(_) => return Err(Error::UnexpectedEofError),
                Err(error) => return Err(error.into()),
            }
            len_buf
        };
        let len = (&len_buf[..]).read_u64::<LittleEndian>()?;
        let expect_cksum = reader.read_u32::<LittleEndian>()?;

        if check_integrity {
            verify_checksum(&len_buf, expect_cksum)?;
        }

        Ok(Some(len as usize))
    }

    pub fn try_read_record_data<R>(
        reader: &mut R,
        len: usize,
        check_integrity: bool,
    ) -> Result<Vec<u8>, Error>
    where
        R: Read,
    {
        let buf = {
            let mut buf = vec![0u8; len];
            reader.read_exact(&mut buf)?;
            buf
        };
        let expect_cksum = reader.read_u32::<LittleEndian>()?;

        if check_integrity {
            verify_checksum(&buf, expect_cksum)?;
        }
        Ok(buf)
    }

    pub fn try_build_record_index<R>(
        reader: &mut R,
        check_integrity: bool,
    ) -> Result<Vec<RecordIndex>, Error>
    where
        R: Read + Seek,
    {
        let mut indexes = vec![];

        while let Some(len) = try_read_len(reader, check_integrity)? {
            let offset = reader.seek(SeekFrom::Current(0))?;
            try_read_record_data(reader, len, check_integrity)?;
            let record_index = RecordIndex { offset, len };
            indexes.push(record_index);
        }

        Ok(indexes)
    }
}
