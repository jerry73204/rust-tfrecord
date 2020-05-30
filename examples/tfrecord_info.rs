use std::{fs::File, io::BufWriter, path::PathBuf};
use tfrecord::{Error, ExampleReader, Feature, RecordReaderInit};

lazy_static::lazy_static! {
    pub static ref INPUT_TFRECORD_PATH: PathBuf = {
        let url = "https://storage.googleapis.com/download.tensorflow.org/data/fsns-20160927/testdata/fsns-00000-of-00001";
        let file_name = "input.tfrecord";

        let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data");
        std::fs::create_dir_all(&data_dir).unwrap();

        let out_path = data_dir.join(file_name);
        let mut out_file = BufWriter::new(File::create(&out_path).unwrap());
        reqwest::blocking::get(url).unwrap().copy_to(&mut out_file).unwrap();

        out_path
    };
    pub static ref DATA_DIR: PathBuf = {
        let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data");
        std::fs::create_dir_all(&data_dir).unwrap();
        data_dir
    };
}

fn main() -> Result<(), Error> {
    // use init pattern to construct the tfrecord reader
    let reader: ExampleReader<_> = RecordReaderInit::default().open(&*INPUT_TFRECORD_PATH)?;

    // print header
    println!("example_no\tfeature_no\tname\ttype\tsize");

    // enumerate examples
    for (example_index, result) in reader.enumerate() {
        let example = result?;

        // enumerate features in an example
        for (feature_index, (name, feature)) in example.into_iter().enumerate() {
            print!("{}\t{}\t{}\t", example_index, feature_index, name);

            match feature {
                Feature::BytesList(list) => {
                    println!("bytes\t{}", list.len());
                }
                Feature::FloatList(list) => {
                    println!("float\t{}", list.len());
                }
                Feature::Int64List(list) => {
                    println!("int64\t{}", list.len());
                }
                Feature::None => {
                    println!("none");
                }
            }
        }
    }

    Ok(())
}
