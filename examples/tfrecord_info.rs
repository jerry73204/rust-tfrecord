mod common;

use common::INPUT_TFRECORD_PATH;
use tffile::{EasyExampleReader, EasyFeature, Error, RecordReaderInit};

fn main() -> Result<(), Error> {
    // use init pattern to construct the tfrecord reader
    let reader: EasyExampleReader<_> = RecordReaderInit {
        check_integrity: true,
    }
    .open(&*INPUT_TFRECORD_PATH)?;

    // print header
    println!("example_no\tfeature_no\tname\ttype\tsize");

    // enumerate examples
    for (example_index, result) in reader.enumerate() {
        let example = result?;

        // enumerate features in an example
        for (feature_index, (name, feature)) in example.into_iter().enumerate() {
            print!("{}\t{}\t{}\t", example_index, feature_index, name);

            match feature {
                EasyFeature::BytesList(list) => {
                    println!("bytes\t{}", list.len());
                }
                EasyFeature::FloatList(list) => {
                    println!("float\t{}", list.len());
                }
                EasyFeature::Int64List(list) => {
                    println!("int64\t{}", list.len());
                }
                EasyFeature::None => {
                    println!("none");
                }
            }
        }
    }

    Ok(())
}
