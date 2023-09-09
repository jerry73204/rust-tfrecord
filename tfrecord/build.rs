use anyhow::Result;
use std::env;

const PROTOBUF_FILE_W_SERDE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/prebuild_src/tensorflow_with_serde.rs",
);
const PROTOBUF_FILE_WO_SERDE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/prebuild_src/tensorflow_without_serde.rs",
);
const BUILD_METHOD_ENV: &str = "TFRECORD_BUILD_METHOD";

fn main() -> Result<()> {
    // re-run conditions
    println!("cargo:rerun-if-changed={}", PROTOBUF_FILE_WO_SERDE);
    println!("cargo:rerun-if-changed={}", PROTOBUF_FILE_W_SERDE);
    println!("cargo:rerun-if-env-changed={}", BUILD_METHOD_ENV);

    #[cfg(feature = "generate_protobuf_src")]
    {
        use tfrecord_codegen::{
            build_by_install_prefix, build_by_src_dir, build_by_src_file, build_by_url,
            guess_build_method, BuildMethod,
        };

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
