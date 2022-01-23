use crate::error::Error;
use crc::Crc;

pub fn checksum(buf: &[u8]) -> u32 {
    const CASTAGNOLI: Crc<u32> = Crc::<u32>::new(&crc::CRC_32_ISCSI);
    let cksum = CASTAGNOLI.checksum(buf);
    ((cksum >> 15) | (cksum << 17)).wrapping_add(0xa282ead8u32)
}

pub fn verify_checksum(buf: &[u8], expect: u32) -> Result<(), Error> {
    let found = checksum(buf);
    if expect == found {
        Ok(())
    } else {
        Err(Error::ChecksumMismatch { expect, found })
    }
}
