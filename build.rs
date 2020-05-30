use failure::Fallible;
use std::path::PathBuf;

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

    let mut config = prost_build::Config::new();
    if cfg!(feature = "serde") {
        config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");
    }
    config.compile_protos(&proto_paths, &[PathBuf::from(include_dir)])?;

    Ok(())
}
