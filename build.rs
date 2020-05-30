use failure::Fallible;
use std::path::PathBuf;

lazy_static::lazy_static! {
    static ref CARGO_MANIFEST_DIR: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
}

fn main() -> Fallible<()> {
    // Generate .rs files from protobuf
    let include_dir = PathBuf::from("tensorflow");
    let proto_paths = glob::glob(
        include_dir
            .join("tensorflow/core/example/*.proto")
            .to_str()
            .unwrap(),
    )?
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?;

    let out_dir = CARGO_MANIFEST_DIR.join("src").join("protos");

    let mut config = prost_build::Config::new();
    config.out_dir(&out_dir);
    if cfg!(feature = "serde") {
        config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");
    }
    config.compile_protos(&proto_paths, &[PathBuf::from(include_dir)])?;

    Ok(())
}
