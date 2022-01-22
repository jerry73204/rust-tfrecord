use anyhow::Result;
use std::{
    env,
    path::{Path, PathBuf},
};

const TENSORFLOW_VERSION: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tensorflow_version"));
const DEFAULT_TENSORFLOW_URL: &str = concat!(
    "https://github.com/tensorflow/tensorflow/archive/v",
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tensorflow_version")),
    ".tar.gz"
);
const BUILD_METHOD_ENV: &str = "TFRECORD_BUILD_METHOD";

lazy_static::lazy_static! {
    static ref OUT_DIR: PathBuf = {
        PathBuf::from(env::var("OUT_DIR").unwrap())
    };
    static ref GENERATED_PROTOBUF_FILE: PathBuf = {
        OUT_DIR.join("tensorflow.rs")
    };
    static ref PROTOBUF_SRC_W_SERDE: PathBuf = {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("prebuild_src").join("tensorflow_with_serde.rs")
    };
    static ref PROTOBUF_FILE_WO_SERDE: PathBuf = {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("prebuild_src").join("tensorflow_without_serde.rs")
    };
}

fn main() -> Result<()> {
    // re-run conditions
    println!(
        "cargo:rerun-if-changed={}",
        PROTOBUF_FILE_WO_SERDE.display()
    );
    println!("cargo:rerun-if-changed={}", PROTOBUF_SRC_W_SERDE.display());
    println!("cargo:rerun-if-env-changed={}", BUILD_METHOD_ENV);

    #[cfg(feature = "generate_protobuf_src")]
    {
        let build_method = guess_build_method()?;

        match build_method {
            None => {}
            Some(BuildMethod::Url(url)) => build_by_url(&url)?,
            Some(BuildMethod::SrcDir(dir)) => build_by_src_dir(dir)?,
            Some(BuildMethod::SrcFile(file)) => build_by_src_file(file)?,
            Some(BuildMethod::InstallPrefix(prefix)) => build_by_install_prefix(prefix)?,
        }
    }

    Ok(())
}

#[cfg(feature = "generate_protobuf_src")]
use codegen::*;

#[cfg(feature = "generate_protobuf_src")]
mod codegen {
    use super::*;
    use std::env::VarError;

    use anyhow::{bail, format_err, Context, Error, Result};
    use flate2::read::GzDecoder;
    use std::{
        env,
        fs::{self, File},
        io::{self, BufReader, BufWriter},
        path::{Path, PathBuf},
    };
    use tar::Archive;

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
the environment variable "{}" must be set with the following format.

- "url://"
  Download the source from default URL "{}".

- "url://https://github.com/tensorflow/tensorflow/archive/vX.Y.Z.tar.gz"
  Download the source from specified URL.

- "src_dir:///path/to/tensorflow/dir"
  Specify unpacked TensorFlow source directory.

- "src_dir:///path/to/tensorflow/file.tar.gz"
  Specify TensorFlow source package file.

- "install_prefix:///path/to/tensorflow/prefix"
  Specify the installed TensorFlow by install prefix.
"#,
            BUILD_METHOD_ENV,
            &*DEFAULT_TENSORFLOW_URL
        )
    }

    pub fn build_by_url(url: &str) -> Result<()> {
        eprintln!("download file {}", url);
        let src_file = download_tensorflow(url)?;
        build_by_src_file(&src_file)
            .with_context(|| format!("remove {} and try again", src_file.display()))?;
        Ok(())
    }

    pub fn build_by_src_dir(src_dir: impl AsRef<Path>) -> Result<()> {
        let src_dir = src_dir.as_ref();

        // re-run if the dir changes
        println!("cargo:rerun-if-changed={}", src_dir.display());

        compile_protobuf(src_dir)?;
        Ok(())
    }

    pub fn build_by_src_file(src_file: impl AsRef<Path>) -> Result<()> {
        let src_file = src_file.as_ref();

        // re-run if the dir changes
        println!("cargo:rerun-if-changed={}", src_file.display());

        let src_dir = extract_src_file(src_file)?;
        compile_protobuf(src_dir)?;
        Ok(())
    }

    pub fn build_by_install_prefix(prefix: impl AsRef<Path>) -> Result<()> {
        compile_protobuf(prefix.as_ref().join("include").join("tensorflow"))?;
        Ok(())
    }

    pub fn extract_src_file(src_file: impl AsRef<Path>) -> Result<PathBuf> {
        let working_dir = OUT_DIR.join("tensorflow");
        let src_file = src_file.as_ref();
        let src_dirname = format!("tensorflow-{}", TENSORFLOW_VERSION);
        let src_dir = working_dir.join(&src_dirname);

        // remove previously extracted dir
        if src_dir.is_dir() {
            fs::remove_dir_all(&src_dir)?;
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

    pub fn compile_protobuf(dir: impl AsRef<Path>) -> Result<()> {
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
            )?;
            let framework_iter = glob::glob(
                dir.join("tensorflow")
                    .join("core")
                    .join("framework")
                    .join("*.proto")
                    .to_str()
                    .unwrap(),
            )?;
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

        // without serde
        {
            prost_build::compile_protos(&proto_paths, &[PathBuf::from(include_dir)])?;
            fs::create_dir_all(PROTOBUF_FILE_WO_SERDE.parent().unwrap())?;
            fs::copy(&*GENERATED_PROTOBUF_FILE, &*PROTOBUF_FILE_WO_SERDE)?;
        }

        // with serde
        {
            prost_build::Config::new()
                .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
                .compile_protos(&proto_paths, &[PathBuf::from(include_dir)])?;
            fs::create_dir_all(PROTOBUF_SRC_W_SERDE.parent().unwrap())?;
            fs::copy(&*GENERATED_PROTOBUF_FILE, &*PROTOBUF_SRC_W_SERDE)?;
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
}
