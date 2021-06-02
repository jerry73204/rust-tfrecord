//! Low level synchronous and asynchronous I/O functions.
//!
//! The functions are used internally to work with generic readers and writers.
//! It is not intended for common users, while we encourage using high level API.

use crate::error::Error;
#[cfg(feature = "async_")]
use futures::io::{AsyncReadExt, AsyncWriteExt};
use std::io::prelude::*;
use std::io::{Error as IoError, ErrorKind};

/// Low level I/O functions with async/await.
#[cfg(feature = "async_")]
pub mod async_ {
    use super::*;

    /// async/await version analogous to blocking [try_read_record](super::blocking::try_read_record).
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

    /// async/await version analogous to blocking [try_read_len](super::blocking::try_read_len).
    pub async fn try_read_len<R>(
        reader: &mut R,
        check_integrity: bool,
    ) -> Result<Option<usize>, Error>
    where
        R: AsyncReadExt + Unpin,
    {
        let len_buf = {
            let mut len_buf = [0u8; std::mem::size_of::<u64>()];
            match reader.read_exact(&mut len_buf).await {
                Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(None),
                Err(e) => return Err(e.into()),
                _ => (),
            }
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

    /// async/await version analogous to blocking [try_read_record_data](super::blocking::try_read_record_data).
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

    /// async/await version analogous to blocking [try_write_record](super::blocking::try_write_record).
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

/// Low level blocking I/O functions.
pub mod blocking {
    use super::*;

    /// Try to extract raw bytes of a record from a generic reader.
    ///
    /// It reads the record length and data from a generic reader,
    /// and verifies the checksum if requested.
    /// If the end of file is reached, it returns `Ok(None)`.
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

    /// Try to read the record length from a generic reader.
    ///
    /// It is internally called by [try_read_record]. It returns `Ok(None)` if reaching the end of file.
    pub fn try_read_len<R>(reader: &mut R, check_integrity: bool) -> Result<Option<usize>, Error>
    where
        R: Read,
    {
        let len_buf = {
            let mut len_buf = [0u8; std::mem::size_of::<u64>()];
            match reader.read_exact(&mut len_buf) {
                Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(None),
                Err(e) => return Err(e.into()),
                _ => (),
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

    /// Read the record raw bytes with given length from a generic reader.
    ///
    /// It is internally called by [try_read_record].
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

    /// Write the raw record bytes to a generic writer.
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
