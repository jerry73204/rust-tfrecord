use crate::error::Error;
use crc::crc32;

pub fn checksum(buf: &[u8]) -> u32 {
    let cksum = crc32::checksum_castagnoli(buf);
    ((cksum >> 15) | (cksum << 17)).wrapping_add(0xa282ead8u32)
}

pub fn verify_checksum(buf: &[u8], expect: u32) -> Result<(), Error> {
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
