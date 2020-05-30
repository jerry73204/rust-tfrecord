use crate::error::Error;
use futures::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use std::io::{prelude::*, SeekFrom};

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
        let len = u64::from_le_bytes(len_buf);

        let expect_cksum = {
            let mut buf = [0; std::mem::size_of::<u32>()];
            reader.read_exact(&mut buf).await?;
            u32::from_le_bytes(buf)
        };

        if check_integrity {
            crate::utils::verify_checksum(&len_buf, expect_cksum)?;
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
            u32::from_le_bytes(buf)
        };

        if check_integrity {
            crate::utils::verify_checksum(&buf, expect_cksum)?;
        }
        Ok(buf)
    }

    pub async fn try_build_record_index<R>(
        reader: &mut R,
        check_integrity: bool,
    ) -> Result<Vec<(u64, usize)>, Error>
    where
        R: AsyncReadExt + AsyncSeekExt + Unpin,
    {
        let mut indexes = vec![];

        while let Some(len) = try_read_len(reader, check_integrity).await? {
            let offset = reader.seek(SeekFrom::Current(0)).await?;
            try_read_record_data(reader, len, check_integrity).await?;
            indexes.push((offset, len));
        }

        Ok(indexes)
    }

    pub async fn try_write_record<W>(writer: &mut W, bytes: Vec<u8>) -> Result<(), Error>
    where
        W: AsyncWriteExt + Unpin,
    {
        // write data size
        {
            let len = bytes.len();
            let len_buf = len.to_le_bytes();
            let cksum = crate::utils::checksum(&len_buf);
            let cksum_buf = cksum.to_le_bytes();

            writer.write_all(&len_buf).await?;
            writer.write_all(&cksum_buf).await?;
        }

        // write data
        {
            let cksum = crate::utils::checksum(&bytes);
            let cksum_buf = cksum.to_le_bytes();

            writer.write_all(bytes.as_slice()).await?;
            writer.write_all(&cksum_buf).await?;
        }
        Ok(())
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
        let len = u64::from_le_bytes(len_buf);
        let expect_cksum = {
            let mut buf = [0; std::mem::size_of::<u32>()];
            reader.read_exact(&mut buf)?;
            u32::from_le_bytes(buf)
        };

        if check_integrity {
            crate::utils::verify_checksum(&len_buf, expect_cksum)?;
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
        let expect_cksum = {
            let mut buf = [0; std::mem::size_of::<u32>()];
            reader.read_exact(&mut buf)?;
            u32::from_le_bytes(buf)
        };

        if check_integrity {
            crate::utils::verify_checksum(&buf, expect_cksum)?;
        }
        Ok(buf)
    }

    pub fn try_build_record_index<R>(
        reader: &mut R,
        check_integrity: bool,
    ) -> Result<Vec<(u64, usize)>, Error>
    where
        R: Read + Seek,
    {
        let mut indexes = vec![];

        while let Some(len) = try_read_len(reader, check_integrity)? {
            let offset = reader.seek(SeekFrom::Current(0))?;
            try_read_record_data(reader, len, check_integrity)?;
            indexes.push((offset, len));
        }

        Ok(indexes)
    }

    pub fn try_write_record<W>(writer: &mut W, bytes: Vec<u8>) -> Result<(), Error>
    where
        W: Write,
    {
        // write data size
        {
            let len = bytes.len();
            let len_buf = len.to_le_bytes();
            let cksum = crate::utils::checksum(&len_buf);
            let cksum_buf = cksum.to_le_bytes();

            writer.write_all(&len_buf)?;
            writer.write_all(&cksum_buf)?;
        }

        // write data
        {
            let cksum = crate::utils::checksum(&bytes);
            let cksum_buf = cksum.to_le_bytes();

            writer.write_all(bytes.as_slice())?;
            writer.write_all(&cksum_buf)?;
        }
        Ok(())
    }
}
