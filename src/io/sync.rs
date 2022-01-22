use crate::error::Error;
use std::io::prelude::*;

/// Try to extract raw bytes of a record from a generic reader.
///
/// It reads the record length and data from a generic reader,
/// and verifies the checksum if requested.
/// If the end of file is reached, it returns `Ok(None)`.
pub fn try_read_record<R>(reader: &mut R, check_integrity: bool) -> Result<Option<Vec<u8>>, Error>
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
        let len_buf = [0u8; std::mem::size_of::<u64>()];
        let len_buf = try_read_exact(reader, len_buf)?;
        match len_buf {
            Some(buf) => buf,
            None => return Ok(None),
        }
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

fn try_read_exact<R, B>(reader: &mut R, mut buf: B) -> Result<Option<B>, Error>
where
    R: Read,
    B: AsMut<[u8]>,
{
    let as_mut = buf.as_mut();
    let mut offset = 0;
    let len = as_mut.len();

    loop {
        match reader.read(&mut as_mut[offset..]) {
            Ok(0) => {
                if offset == len {
                    return Ok(Some(buf));
                } else if offset == 0 {
                    return Ok(None);
                } else {
                    return Err(Error::UnexpectedEofError);
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
