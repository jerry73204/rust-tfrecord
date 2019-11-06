use failure::Fallible;
use std::path::PathBuf;

fn main() -> Fallible<()> {
    // Generate .rs files from protobuf
    let include_dir = PathBuf::from("third_party/tensorflow");
    let proto_paths = glob::glob(
        include_dir
            .join("tensorflow/core/example/*.proto")
            .to_str()
            .unwrap(),
    )?
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?;
    prost_build::compile_protos(&proto_paths, &[PathBuf::from(include_dir)])?;

    Ok(())
}
