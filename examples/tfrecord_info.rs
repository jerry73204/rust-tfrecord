use once_cell::sync::Lazy;
use std::{
    fs::{self, File},
    io::{self, prelude::*, BufWriter},
    path::{Path, PathBuf},
};
use tfrecord::{Error, ExampleIter, FeatureKind};

static INPUT_TFRECORD_PATH: Lazy<PathBuf> = Lazy::new(|| {
    (move || {
        let url = include_str!("../tests/tfrecord_link.txt");
        let path = DATA_DIR.join("input.tfrecord");
        let mut writer = BufWriter::new(File::create(&path)?);
        io::copy(&mut ureq::get(url).call()?.into_reader(), &mut writer)?;
        writer.flush()?;
        anyhow::Ok(path)
    })()
    .unwrap()
});
static DATA_DIR: Lazy<&Path> = Lazy::new(|| {
    let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/test_data"));
    fs::create_dir_all(path).unwrap();
    path
});

fn main() -> Result<(), Error> {
    // use init pattern to construct the tfrecord reader
    let reader = ExampleIter::open(&*INPUT_TFRECORD_PATH, Default::default())?;

    // print header
    println!("example_no\tfeature_no\tname\ttype\tsize");

    // enumerate examples
    for (example_index, result) in reader.enumerate() {
        let example = result?;

        // enumerate features in an example
        for (feature_index, (name, feature)) in example.into_iter().enumerate() {
            print!("{}\t{}\t{}\t", example_index, feature_index, name);

            use FeatureKind as F;
            match feature.into_kinds() {
                Some(F::Bytes(list)) => {
                    println!("bytes\t{}", list.len());
                }
                Some(F::F32(list)) => {
                    println!("float\t{}", list.len());
                }
                Some(F::I64(list)) => {
                    println!("int64\t{}", list.len());
                }
                None => {
                    println!("none");
                }
            }
        }
    }

    Ok(())
}
