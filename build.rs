use failure::{bail, ensure, Error, Fallible};
use flate2::read::GzDecoder;
use std::{
    env::{self, VarError},
    fs::File,
    io::{prelude::*, BufReader, BufWriter},
    path::{Path, PathBuf},
    str::FromStr,
};
use tar::Archive;

const TENSORFLOW_VERSION: &str = "2.2.0";
const BUILD_METHOD_ENV: &str = "TFRECORD_BUILD_METHOD";

lazy_static::lazy_static! {
    static ref TENSORFLOW_URL: String = format!("https://github.com/tensorflow/tensorflow/archive/v{}.tar.gz", TENSORFLOW_VERSION);
    static ref TENSORFLOW_B3SUM: Vec<u8> = hex::decode("e10d1c18f528df623dd1df82968e1bd7c83104a1a11522e569f0839beb36c709").unwrap();
    static ref OUT_DIR: PathBuf = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    static ref CARGO_MANIFEST_DIR: PathBuf = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    static ref GENERATED_PROTOBUF_FILE: PathBuf = {
        let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
        out_dir.join("tensorflow.rs")
    };
    static ref PREBUILD_PROTOBUF_SRC_WITH_SERDE: PathBuf = {
        let cargo_manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
        cargo_manifest_dir.join("prebuild_src").join("tensorflow_with_serde.rs")
    };
    static ref PREBUILD_PROTOBUF_SRC_WITHOUT_SERDE: PathBuf = {
        let cargo_manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
        cargo_manifest_dir.join("prebuild_src").join("tensorflow_without_serde.rs")
    };
}

#[derive(Debug, Clone)]
enum BuildMethod {
    Url(String),
    SrcDir(PathBuf),
    SrcFile(PathBuf),
    InstallPrefix(PathBuf),
    PreBuild,
}

impl FromStr for BuildMethod {
    type Err = Error;

    fn from_str(text: &str) -> Fallible<Self> {
        const URL_PREFIX: &str = "url://";
        const SRC_DIR_PREFIX: &str = "src_dir://";
        const SRC_FILE_PREFIX: &str = "src_file://";
        const INSTALL_PREFIX_PREFIX: &str = "install_prefix://";
        const PREBUILD_PREFIX: &str = "prebuild://";

        let method = if text == PREBUILD_PREFIX {
            BuildMethod::PreBuild
        } else if text.starts_with(URL_PREFIX) {
            let url = text[URL_PREFIX.len()..].to_owned();
            Self::Url(url)
        } else if text.starts_with(SRC_DIR_PREFIX) {
            let dir = PathBuf::from(&text[SRC_DIR_PREFIX.len()..]);
            Self::SrcDir(dir)
        } else if text.starts_with(SRC_FILE_PREFIX) {
            let path = PathBuf::from(&text[SRC_FILE_PREFIX.len()..]);
            Self::SrcFile(path)
        } else if text.starts_with(INSTALL_PREFIX_PREFIX) {
            let prefix = PathBuf::from(&text[INSTALL_PREFIX_PREFIX.len()..]);
            Self::InstallPrefix(prefix)
        } else {
            bail!(r#"invalid build method specification "{}""#, text);
        };

        Ok(method)
    }
}

fn main() -> Fallible<()> {
    // re-run conditions
    println!(
        "cargo:rerun-if-changed={}",
        PREBUILD_PROTOBUF_SRC_WITHOUT_SERDE.display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        PREBUILD_PROTOBUF_SRC_WITH_SERDE.display()
    );
    println!("cargo:rerun-if-env-changed={}", BUILD_METHOD_ENV);

    let build_method = guess_build_method()?;

    match build_method {
        BuildMethod::PreBuild => copy_prebuild_src()?,
        BuildMethod::Url(url) => build_by_url(&url)?,
        BuildMethod::SrcDir(dir) => build_by_src_dir(dir)?,
        BuildMethod::SrcFile(file) => build_by_src_file(file)?,
        BuildMethod::InstallPrefix(prefix) => build_by_install_prefix(prefix)?,
    }

    Ok(())
}

fn guess_build_method() -> Fallible<BuildMethod> {
    let build_method = match env::var(BUILD_METHOD_ENV) {
        Ok(hint) => BuildMethod::from_str(&hint)?,
        Err(VarError::NotPresent) => BuildMethod::PreBuild,
        Err(VarError::NotUnicode(_)) => {
            bail!(
                r#"the value of environment variable "{}" is not Unicode"#,
                BUILD_METHOD_ENV
            );
        }
    };
    Ok(build_method)
}

fn build_by_url(url: &str) -> Fallible<()> {
    let src_file = download_tensorflow(url)?;
    build_by_src_file(src_file)?;
    Ok(())
}

fn build_by_src_dir<P>(src_dir: P) -> Fallible<()>
where
    P: AsRef<Path>,
{
    let src_dir = src_dir.as_ref();

    // re-run if the dir changes
    println!("cargo:rerun-if-changed={}", src_dir.display());

    compile_protobuf(src_dir)?;
    Ok(())
}

fn build_by_src_file<P>(src_file: P) -> Fallible<()>
where
    P: AsRef<Path>,
{
    let src_file = src_file.as_ref();

    // re-run if the dir changes
    println!("cargo:rerun-if-changed={}", src_file.display());

    let src_dir = extract_src_file(src_file)?;
    compile_protobuf(src_dir)?;
    Ok(())
}

fn build_by_install_prefix<P>(prefix: P) -> Fallible<()>
where
    P: AsRef<Path>,
{
    compile_protobuf(prefix.as_ref().join("include").join("tensorflow"))?;
    Ok(())
}

fn copy_prebuild_src() -> Fallible<()> {
    // check if conflicting "generate_protobuf_src" feature presents
    if cfg!(feature = "generate_protobuf_src") {
        bail!(r#"please specify the environment variable "{}" in combination with "generate_protobuf_src" feature"#, BUILD_METHOD_ENV);
    }

    // copy file
    let prebuild_file: &Path = if cfg!(feature = "serde") {
        &*PREBUILD_PROTOBUF_SRC_WITH_SERDE
    } else {
        &*PREBUILD_PROTOBUF_SRC_WITHOUT_SERDE
    };

    if !prebuild_file.is_file() {
        bail!(
            "Look like the protobuf code is not generated yet. Please read README for instructions"
        );
    }

    std::fs::copy(prebuild_file, &*GENERATED_PROTOBUF_FILE)?;

    Ok(())
}

fn extract_src_file<P>(src_file: P) -> Fallible<PathBuf>
where
    P: AsRef<Path>,
{
    let working_dir = OUT_DIR.join("tensorflow");
    let src_file = src_file.as_ref();
    let src_dirname = format!("tensorflow-{}", TENSORFLOW_VERSION);
    let src_dir = working_dir.join(&src_dirname);

    // remove previously extracted dir
    if src_dir.is_dir() {
        std::fs::remove_dir_all(&src_dir)?;
    }

    // verify source file
    {
        let mut buf = vec![];
        let mut file = BufReader::new(File::open(src_file)?);
        file.read_to_end(&mut buf)?;
        ensure!(
            blake3::hash(&buf).as_bytes().as_ref() == TENSORFLOW_B3SUM.as_slice(),
            "the downloaded tensorflow package does not pass the integrity check"
        );
    }

    // extract package
    {
        let file = BufReader::new(File::open(src_file)?);
        let tar = GzDecoder::new(file);
        let mut archive = Archive::new(tar);
        archive.unpack(&working_dir)?;

        if !src_dir.is_dir() {
            bail!(
                r#"expect "{}" directory in source package. Did you download the wrong version?"#,
                src_dirname
            );
        }
    }

    Ok(src_dir)
}

fn compile_protobuf<P>(dir: P) -> Fallible<()>
where
    P: AsRef<Path>,
{
    let dir = dir.as_ref();
    let include_dir = dir;
    let proto_paths = glob::glob(
        dir.join("tensorflow")
            .join("core")
            .join("example")
            .join("*.proto")
            .to_str()
            .unwrap(),
    )?
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?;

    // compile protobuf
    {
        let mut config = prost_build::Config::new();

        // conditionally use serde
        if cfg!(feature = "serde") {
            config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");
        }

        config.compile_protos(&proto_paths, &[PathBuf::from(include_dir)])?;

        // verify build
        if !GENERATED_PROTOBUF_FILE.is_file() {
            bail!(
                r#"expect a compiled protobuf code at "{}" but not found"#,
                GENERATED_PROTOBUF_FILE.display()
            );
        }
    }

    // save compiled protobuf code if "generate_protobuf_src" feature presents
    if cfg!(feature = "generate_protobuf_src") {
        let prebuild_dest: &Path = if cfg!(feature = "serde") {
            &*PREBUILD_PROTOBUF_SRC_WITH_SERDE
        } else {
            &*PREBUILD_PROTOBUF_SRC_WITHOUT_SERDE
        };

        std::fs::create_dir_all(prebuild_dest.parent().unwrap())?;
        std::fs::copy(&*GENERATED_PROTOBUF_FILE, prebuild_dest)?;
    }

    Ok(())
}

fn download_tensorflow(url: &str) -> Fallible<PathBuf> {
    let working_dir = OUT_DIR.join("tensorflow");
    let tar_path = working_dir.join(format!("v{}.tar.gz", TENSORFLOW_VERSION));

    // createw working dir
    std::fs::create_dir_all(&working_dir)?;

    // return if downloaded package exists
    if tar_path.is_file() {
        return Ok(tar_path);
    }

    // download file
    {
        let mut file = BufWriter::new(File::create(&tar_path)?);
        reqwest::blocking::get(url).unwrap().copy_to(&mut file)?;
    }

    Ok(tar_path)
}
