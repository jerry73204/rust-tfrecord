use anyhow::{bail, format_err, Context, Error, Result};
use flate2::read::GzDecoder;
use itertools::{chain, Itertools};
use once_cell::sync::Lazy;
use std::{
    env::{self, VarError},
    fs::{self, File},
    io::{self, BufReader, BufWriter},
    path::{Path, PathBuf},
};
use tar::Archive;

const BUILD_METHOD_ENV: &str = "TFRECORD_BUILD_METHOD";

static OUT_DIR: Lazy<PathBuf> = Lazy::new(|| PathBuf::from(env::var("OUT_DIR").unwrap()));
static GENERATED_PROTOBUF_FILE: Lazy<PathBuf> = Lazy::new(|| (*OUT_DIR).join("tensorflow.rs"));
const TENSORFLOW_VERSION: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tensorflow_version"));
const DEFAULT_TENSORFLOW_URL: &str = concat!(
    "https://github.com/tensorflow/tensorflow/archive/v",
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tensorflow_version")),
    ".tar.gz"
);

#[derive(Debug, Clone)]
pub enum BuildMethod {
    Url(String),
    SrcDir(PathBuf),
    SrcFile(PathBuf),
    InstallPrefix(PathBuf),
}

pub fn guess_build_method() -> Result<Option<BuildMethod>> {
    let build_method = match env::var(BUILD_METHOD_ENV) {
        Ok(text) => {
            const URL_PREFIX: &str = "url://";
            const SRC_DIR_PREFIX: &str = "src_dir://";
            const SRC_FILE_PREFIX: &str = "src_file://";
            const INSTALL_PREFIX_PREFIX: &str = "install_prefix://";

            let method = if let Some(url) = text.strip_prefix(URL_PREFIX) {
                match url {
                    "" => BuildMethod::Url(DEFAULT_TENSORFLOW_URL.to_string()),
                    _ => BuildMethod::Url(url.to_string()),
                }
            } else if let Some(dir) = text.strip_prefix(SRC_DIR_PREFIX) {
                BuildMethod::SrcDir(dir.into())
            } else if let Some(path) = text.strip_prefix(SRC_FILE_PREFIX) {
                BuildMethod::SrcFile(path.into())
            } else if let Some(prefix) = text.strip_prefix(INSTALL_PREFIX_PREFIX) {
                BuildMethod::InstallPrefix(prefix.into())
            } else {
                return Err(build_method_error());
            };

            method
        }
        Err(VarError::NotPresent) => return Err(build_method_error()),
        Err(VarError::NotUnicode(_)) => {
            bail!(
                r#"the value of environment variable "{}" is not Unicode"#,
                BUILD_METHOD_ENV
            );
        }
    };
    Ok(Some(build_method))
}

pub fn build_method_error() -> Error {
    format_err!(
        r#"By enabling the "generate_protobuf_src" feature,
the environment variable "{BUILD_METHOD_ENV}" must be set with the following format.

- "url://"
  Download the source from default URL "{DEFAULT_TENSORFLOW_URL}".

- "url://https://github.com/tensorflow/tensorflow/archive/vX.Y.Z.tar.gz"
  Download the source from specified URL.

- "src_dir:///path/to/tensorflow/dir"
  Specify unpacked TensorFlow source directory.

- "src_file:///path/to/tensorflow/file.tar.gz"
  Specify TensorFlow source package file.

- "install_prefix:///path/to/tensorflow/prefix"
  Specify the installed TensorFlow by install prefix.
"#,
    )
}

pub fn build_by_url<P>(url: &str, out_dir: P) -> Result<()>
where
    P: AsRef<Path>,
{
    eprintln!("download file {}", url);
    let src_file = download_tensorflow(url).with_context(|| format!("unable to download {url}"))?;
    build_by_src_file(&src_file, out_dir)
        .with_context(|| format!("remove {} and try again", src_file.display()))?;
    Ok(())
}

pub fn build_by_src_dir<P, P2>(src_dir: P, out_dir: P2) -> Result<()>
where
    P: AsRef<Path>,
    P2: AsRef<Path>,
{
    let src_dir = src_dir.as_ref();

    // re-run if the dir changes
    println!("cargo:rerun-if-changed={}", src_dir.display());

    compile_protobuf(src_dir, out_dir)?;
    Ok(())
}

pub fn build_by_src_file<P, P2>(src_file: P, out_dir: P2) -> Result<()>
where
    P: AsRef<Path>,
    P2: AsRef<Path>,
{
    let src_file = src_file.as_ref();

    // re-run if the dir changes
    println!("cargo:rerun-if-changed={}", src_file.display());

    let src_dir = extract_src_file(src_file)?;
    compile_protobuf(src_dir, out_dir)?;
    Ok(())
}

pub fn build_by_install_prefix<P, P2>(prefix: P, out_dir: P2) -> Result<()>
where
    P: AsRef<Path>,
    P2: AsRef<Path>,
{
    let dir = prefix.as_ref().join("include").join("tensorflow");
    compile_protobuf(dir, out_dir)?;
    Ok(())
}

pub fn extract_src_file<P>(src_file: P) -> Result<PathBuf>
where
    P: AsRef<Path>,
{
    let working_dir = OUT_DIR.join("tensorflow");
    let src_file = src_file.as_ref();
    let src_dirname = format!("tensorflow-{TENSORFLOW_VERSION}");
    let src_dir = working_dir.join(&src_dirname);

    // remove previously extracted dir
    if src_dir.is_dir() {
        fs::remove_dir_all(&src_dir)?;
    }

    // extract package
    {
        let file = BufReader::new(
            File::open(src_file)
                .with_context(|| format!("unable to open {}", src_file.display()))?,
        );
        let tar = GzDecoder::new(file);
        let mut archive = Archive::new(tar);
        archive
            .unpack(&working_dir)
            .with_context(|| format!("unable to unpack {}", working_dir.display()))?;

        if !src_dir.is_dir() {
            bail!(
                r#"expect "{}" directory in source package. Did you download the correct version?"#,
                src_dirname
            );
        }
    }

    Ok(src_dir)
}

pub fn compile_protobuf<P1, P2>(src_dir: P1, out_dir: P2) -> Result<()>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    let dir = src_dir.as_ref();
    let include_dir = dir;
    let proto_paths = {
        let example_pattern = dir
            .join("tensorflow")
            .join("core")
            .join("example")
            .join("*.proto");
        let framework_pattern = dir
            .join("tensorflow")
            .join("core")
            .join("framework")
            .join("*.proto");
        let event_proto = dir
            .join("tensorflow")
            .join("core")
            .join("util")
            .join("event.proto");

        let example_iter = glob::glob(example_pattern.to_str().unwrap())
            .with_context(|| format!("unable to find {}", example_pattern.display()))?;
        let framework_iter = glob::glob(framework_pattern.to_str().unwrap())
            .with_context(|| format!("unable to find {}", framework_pattern.display()))?;
        let paths: Vec<_> =
            chain!(example_iter, framework_iter, [Ok(event_proto)]).try_collect()?;
        paths
    };

    let out_dir = out_dir.as_ref();
    let prebuild_src_dir = out_dir.join("prebuild_src");
    let w_serde_path = prebuild_src_dir.join("tensorflow_with_serde.rs");
    let wo_serde_path = prebuild_src_dir.join("tensorflow_without_serde.rs");

    fs::create_dir_all(prebuild_src_dir)?;

    // without serde
    {
        prost_build::compile_protos(&proto_paths, &[PathBuf::from(include_dir)])?;
        fs::copy(&*GENERATED_PROTOBUF_FILE, wo_serde_path)?;
    }

    // with serde
    {
        prost_build::Config::new()
            .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
            .compile_protos(&proto_paths, &[PathBuf::from(include_dir)])?;
        fs::copy(&*GENERATED_PROTOBUF_FILE, w_serde_path)?;
    }

    Ok(())
}

pub fn download_tensorflow(url: &str) -> Result<PathBuf> {
    let working_dir = OUT_DIR.join("tensorflow");
    let tar_path = working_dir.join(format!("v{}.tar.gz", TENSORFLOW_VERSION));

    // createw working dir
    fs::create_dir_all(&working_dir)?;

    // return if downloaded package exists
    if tar_path.is_file() {
        return Ok(tar_path);
    }

    // download file
    io::copy(
        &mut ureq::get(url).call()?.into_reader(),
        &mut BufWriter::new(File::create(&tar_path)?),
    )?;

    Ok(tar_path)
}
