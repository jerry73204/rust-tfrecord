use anyhow::Result;

const BUILD_METHOD_ENV: &str = "TFRECORD_BUILD_METHOD";

fn main() -> Result<()> {
    // re-run conditions
    println!("cargo:rerun-if-env-changed={}", BUILD_METHOD_ENV);

    #[cfg(feature = "generate_protobuf_src")]
    {
        use tfrecord_codegen::{
            build_by_install_prefix, build_by_src_dir, build_by_src_file, build_by_url,
            guess_build_method, BuildMethod,
        };

        let build_method = guess_build_method()?;

        let prebuild_src_dir = env!("CARGO_MANIFEST_DIR");

        match build_method {
            None => {}
            Some(BuildMethod::Url(url)) => build_by_url(&url, prebuild_src_dir)?,
            Some(BuildMethod::SrcDir(dir)) => build_by_src_dir(dir, prebuild_src_dir)?,
            Some(BuildMethod::SrcFile(file)) => build_by_src_file(file, prebuild_src_dir)?,
            Some(BuildMethod::InstallPrefix(prefix)) => {
                build_by_install_prefix(prefix, prebuild_src_dir)?
            }
        }
    }

    Ok(())
}
