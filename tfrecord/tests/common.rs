pub use anyhow::{ensure, format_err, Error, Result};

use once_cell::sync::Lazy;
use std::{
    fs,
    fs::File,
    io,
    io::{prelude::*, BufWriter},
    path::{Path, PathBuf},
};

#[allow(dead_code)]
pub static IMAGE_URLS: Lazy<Vec<String>> = Lazy::new(|| {
    (move || -> Result<_> {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/image_links.txt");
        let lines: Vec<_> = fs::read_to_string(path)?
            .lines()
            .map(|line| line.to_string())
            .collect();
        Ok(lines)
    })()
    .unwrap()
});

pub static DATA_DIR: Lazy<&Path> = Lazy::new(|| {
    let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/test_data"));
    fs::create_dir_all(path).unwrap();
    path
});

pub static INPUT_TFRECORD_PATH: Lazy<PathBuf> = Lazy::new(|| {
    (move || {
        let url = include_str!("tfrecord_link.txt");
        let path = DATA_DIR.join("input.tfrecord");
        let mut writer = BufWriter::new(File::create(&path)?);
        io::copy(&mut ureq::get(url).call()?.into_reader(), &mut writer)?;
        writer.flush()?;
        anyhow::Ok(path)
    })()
    .unwrap()
});
