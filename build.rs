use anyhow::{bail, Context, Error, Result};
use flate2::read::GzDecoder;
use std::{
    env::{self, VarError},
    fs::File,
    io::{self, BufReader, BufWriter},
    path::{Path, PathBuf},
    str::FromStr,
};
use tar::Archive;

const DEFAULT_TENSORFLOW_VERSION: &str = "2.3.1";
const BUILD_METHOD_ENV: &str = "TFRECORD_BUILD_METHOD";

lazy_static::lazy_static! {
    static ref DEFAULT_TENSORFLOW_URL: String = format!("https://github.com/tensorflow/tensorflow/archive/v{}.tar.gz", DEFAULT_TENSORFLOW_VERSION);
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

    fn from_str(text: &str) -> Result<Self> {
        const URL_PREFIX: &str = "url://";
        const SRC_DIR_PREFIX: &str = "src_dir://";
        const SRC_FILE_PREFIX: &str = "src_file://";
        const INSTALL_PREFIX_PREFIX: &str = "install_prefix://";
        const PREBUILD_PREFIX: &str = "prebuild://";

        let method = if text == PREBUILD_PREFIX {
            BuildMethod::PreBuild
        } else if text.starts_with(URL_PREFIX) {
            let url = text[URL_PREFIX.len()..].to_owned();
            match url.as_str() {
                "" => Self::Url(DEFAULT_TENSORFLOW_URL.to_string()),
                _ => Self::Url(url),
            }
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

fn main() -> Result<()> {
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

fn guess_build_method() -> Result<BuildMethod> {
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

fn build_by_url(url: &str) -> Result<()> {
    eprintln!("download file {}", url);
    let src_file = download_tensorflow(url)?;
    build_by_src_file(&src_file)
        .with_context(|| format!("remove {} and try again", src_file.display()))?;
    Ok(())
}

fn build_by_src_dir<P>(src_dir: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let src_dir = src_dir.as_ref();

    // re-run if the dir changes
    println!("cargo:rerun-if-changed={}", src_dir.display());

    compile_protobuf(src_dir)?;
    Ok(())
}

fn build_by_src_file<P>(src_file: P) -> Result<()>
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

fn build_by_install_prefix<P>(prefix: P) -> Result<()>
where
    P: AsRef<Path>,
{
    compile_protobuf(prefix.as_ref().join("include").join("tensorflow"))?;
    Ok(())
}

fn copy_prebuild_src() -> Result<()> {
    // check if conflicting "generate_protobuf_src" feature presents
    if cfg!(feature = "generate_protobuf_src") {
        bail!(
            r#"please specify the environment variable "{}" in combination with "generate_protobuf_src" feature"#,
            BUILD_METHOD_ENV
        );
    }

    // copy file
    let prebuild_file: &Path = if cfg!(feature = "with-serde") {
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

fn extract_src_file<P>(src_file: P) -> Result<PathBuf>
where
    P: AsRef<Path>,
{
    let working_dir = OUT_DIR.join("tensorflow");
    let src_file = src_file.as_ref();
    let src_dirname = format!("tensorflow-{}", DEFAULT_TENSORFLOW_VERSION);
    let src_dir = working_dir.join(&src_dirname);

    // remove previously extracted dir
    if src_dir.is_dir() {
        std::fs::remove_dir_all(&src_dir)?;
    }

    // extract package
    {
        let file = BufReader::new(File::open(src_file)?);
        let tar = GzDecoder::new(file);
        let mut archive = Archive::new(tar);
        archive.unpack(&working_dir)?;

        if !src_dir.is_dir() {
            bail!(
                r#"expect "{}" directory in source package. Did you download the correct version?"#,
                src_dirname
            );
        }
    }

    Ok(src_dir)
}

fn compile_protobuf<P>(dir: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let dir = dir.as_ref();
    let include_dir = dir;
    let proto_paths = {
        let example_iter = glob::glob(
            dir.join("tensorflow")
                .join("core")
                .join("example")
                .join("*.proto")
                .to_str()
                .unwrap(),
        )?
        .into_iter();
        let framework_iter = glob::glob(
            dir.join("tensorflow")
                .join("core")
                .join("framework")
                .join("*.proto")
                .to_str()
                .unwrap(),
        )?
        .into_iter();
        let util_iter = std::iter::once(Ok(dir
            .join("tensorflow")
            .join("core")
            .join("util")
            .join("event.proto")));
        example_iter
            .chain(framework_iter)
            .chain(util_iter)
            .collect::<Result<Vec<_>, _>>()?
    };

    // compile protobuf
    {
        let mut config = prost_build::Config::new();

        // conditionally use serde
        if cfg!(feature = "with-serde") {
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
        let prebuild_dest: &Path = if cfg!(feature = "with-serde") {
            &*PREBUILD_PROTOBUF_SRC_WITH_SERDE
        } else {
            &*PREBUILD_PROTOBUF_SRC_WITHOUT_SERDE
        };

        std::fs::create_dir_all(prebuild_dest.parent().unwrap())?;
        std::fs::copy(&*GENERATED_PROTOBUF_FILE, prebuild_dest)?;
    }

    Ok(())
}

fn download_tensorflow(url: &str) -> Result<PathBuf> {
    let working_dir = OUT_DIR.join("tensorflow");
    let tar_path = working_dir.join(format!("v{}.tar.gz", DEFAULT_TENSORFLOW_VERSION));

    // createw working dir
    std::fs::create_dir_all(&working_dir)?;

    // return if downloaded package exists
    if tar_path.is_file() {
        return Ok(tar_path);
    }

    // download file
    io::copy(
        &mut ureq::get(url).call().into_reader(),
        &mut BufWriter::new(File::create(&tar_path)?),
    )?;

    Ok(tar_path)
}
