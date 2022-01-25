mod common;

use common::*;
use tfrecord::{BytesIter, BytesWriter, Example, ExampleIter, ExampleWriter};

#[test]
fn reader_test() {
    // bytes
    {
        let reader = BytesIter::open(&*INPUT_TFRECORD_PATH, Default::default()).unwrap();
        reader.collect::<Result<Vec<Vec<u8>>, _>>().unwrap();
    }

    // examples
    {
        let reader = ExampleIter::open(&*INPUT_TFRECORD_PATH, Default::default()).unwrap();
        reader.collect::<Result<Vec<Example>, _>>().unwrap();
    }
}

#[test]
fn writer_test() {
    let output_path = DATA_DIR.join("blocking_writer_output.tfrecord");

    // bytes
    {
        let reader = BytesIter::open(&*INPUT_TFRECORD_PATH, Default::default()).unwrap();
        let mut writer = BytesWriter::create(&output_path).unwrap();

        for result in reader {
            let bytes = result.unwrap();
            writer.send(bytes).unwrap();
        }

        std::fs::remove_file(&output_path).unwrap();
    }

    // examples
    {
        let reader = ExampleIter::open(&*INPUT_TFRECORD_PATH, Default::default()).unwrap();
        let mut writer = ExampleWriter::create(&output_path).unwrap();

        for result in reader {
            let example = result.unwrap();
            writer.send(example).unwrap();
        }

        std::fs::remove_file(&output_path).unwrap();
    }
}

#[cfg(feature = "serde")]
#[test]
fn serde_test() {
    use prost::Message as _;

    BytesIter::open(&*INPUT_TFRECORD_PATH, Default::default())
        .unwrap()
        .try_for_each(|result| -> Result<_> {
            let bytes = result.unwrap();
            let example = Example::decode(bytes.as_slice()).unwrap();
            let _: Example =
                serde_json::from_str(&serde_json::to_string(&example).unwrap()).unwrap();

            Ok(())
        })
        .unwrap();
}
