use failure::{ensure, Fallible};
use flate2::read::GzDecoder;
use std::{
    env,
    fs::File,
    io::{prelude::*, BufReader, BufWriter},
    path::PathBuf,
};
use tar::Archive;

const TENSORFLOW_VERSION: &str = "2.2.0";

lazy_static::lazy_static! {
    static ref TENSORFLOW_URL: String = format!("https://github.com/tensorflow/tensorflow/archive/v{}.tar.gz", TENSORFLOW_VERSION);
    static ref TENSORFLOW_B3SUM: Vec<u8> = hex::decode("e10d1c18f528df623dd1df82968e1bd7c83104a1a11522e569f0839beb36c709").unwrap();
    static ref OUT_DIR: PathBuf = PathBuf::from(env::var_os("OUT_DIR").unwrap());
}

fn main() -> Fallible<()> {
    // download and extract tensorflow package
    let tensorflow_dir = download_tensorflow()?;

    // Generate .rs files from protobuf
    let include_dir = &tensorflow_dir;
    let proto_paths = glob::glob(
        tensorflow_dir
            .join("tensorflow")
            .join("core")
            .join("example")
            .join("*.proto")
            .to_str()
            .unwrap(),
    )?
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?;

    let mut config = prost_build::Config::new();
    if cfg!(feature = "serde") {
        config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");
    }
    config.compile_protos(&proto_paths, &[PathBuf::from(include_dir)])?;

    Ok(())
}

fn download_tensorflow() -> Fallible<PathBuf> {
    let working_dir = OUT_DIR.join("tensorflow");
    let tar_path = working_dir.join(format!("v{}.tar.gz", TENSORFLOW_VERSION));
    let src_dir = working_dir.join(format!("tensorflow-{}", TENSORFLOW_VERSION));

    // createw working dir
    std::fs::create_dir_all(&working_dir)?;

    // check if extracted directly exists
    if src_dir.is_dir() {
        return Ok(src_dir);
    }

    // download file
    {
        let mut file = BufWriter::new(File::create(&tar_path)?);
        reqwest::blocking::get(&*TENSORFLOW_URL)
            .unwrap()
            .copy_to(&mut file)?;
    }

    // verify file
    {
        let mut buf = vec![];
        let mut file = BufReader::new(File::open(&tar_path)?);
        file.read_to_end(&mut buf)?;
        ensure!(
            blake3::hash(&buf).as_bytes().as_ref() == TENSORFLOW_B3SUM.as_slice(),
            "the downloaded tensorflow package does not pass the integrity check"
        );
    }

    // extract package
    {
        let file = BufReader::new(File::open(&tar_path)?);
        let tar = GzDecoder::new(file);
        let mut archive = Archive::new(tar);
        archive.unpack(&working_dir)?;
    }

    Ok(src_dir)
}
