use std::mem;

use crate::error::Error;
use futures::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// async/await version analogous to blocking [try_read_record](super::blocking::try_read_record).
pub async fn try_read_record<R>(
    reader: &mut R,
    check_integrity: bool,
) -> Result<Option<Vec<u8>>, Error>
where
    R: AsyncRead + Unpin,
{
    let len = match try_read_len(reader, check_integrity).await? {
        Some(len) => len,
        None => return Ok(None),
    };
    let data = try_read_record_data(reader, len, check_integrity).await?;
    Ok(Some(data))
}

/// async/await version analogous to blocking [try_read_len](super::blocking::try_read_len).
pub async fn try_read_len<R>(reader: &mut R, check_integrity: bool) -> Result<Option<usize>, Error>
where
    R: AsyncRead + Unpin,
{
    let len_buf = {
        let len_buf = [0u8; mem::size_of::<u64>()];
        let len_buf = try_read_exact(reader, len_buf).await?;
        match len_buf {
            Some(buf) => buf,
            None => return Ok(None),
        }
    };
    let len = u64::from_le_bytes(len_buf);

    let expect_cksum = {
        let mut buf = [0; mem::size_of::<u32>()];
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
    R: AsyncRead + Unpin,
{
    let buf = {
        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf).await?;
        buf
    };
    let expect_cksum = {
        let mut buf = [0u8; mem::size_of::<u32>()];
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
    W: AsyncWrite + Unpin,
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

async fn try_read_exact<R, B>(reader: &mut R, mut buf: B) -> Result<Option<B>, Error>
where
    R: AsyncRead + Unpin,
    B: AsMut<[u8]>,
{
    let as_mut = buf.as_mut();
    let mut offset = 0;
    let len = as_mut.len();

    loop {
        match reader.read(&mut as_mut[offset..]).await {
            Ok(0) => {
                if offset == len {
                    return Ok(Some(buf));
                } else if offset == 0 {
                    return Ok(None);
                } else {
                    return Err(Error::UnexpectedEof);
                }
            }
            Ok(n) => {
                offset += n;
                if offset == len {
                    return Ok(Some(buf));
                }
            }
            Err(error) => return Err(error.into()),
        }
    }
}
