use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    path::{Path, PathBuf, MAIN_SEPARATOR},
};

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

pub fn split_prefix<'a>(prefix: impl Into<Cow<'a, str>>) -> (PathBuf, OsString) {
    let prefix = prefix.into();
    if prefix.ends_with(MAIN_SEPARATOR) {
        let dir = PathBuf::from(prefix.into_owned());
        (dir, OsString::from(""))
    } else {
        let prefix = Path::new(prefix.as_ref());
        let file_name_prefix = prefix.file_name().unwrap_or_else(|| OsStr::new(""));
        let dir = prefix.parent().unwrap(); // TODO
        (dir.to_owned(), file_name_prefix.to_owned())
    }
}
