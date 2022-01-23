mod common;

use common::*;
use prost::Message as _;
use std::{fs::File, io::BufWriter};
use tfrecord::{BytesIter, Example, ExampleIter, RawExample, RawExampleIter, RecordWriter};

#[test]
fn reader_test() {
    // bytes
    {
        let reader = BytesIter::open(&*INPUT_TFRECORD_PATH, Default::default()).unwrap();
        reader.collect::<Result<Vec<Vec<u8>>, _>>().unwrap();
    }

    // raw examples
    {
        let reader = RawExampleIter::open(&*INPUT_TFRECORD_PATH, Default::default()).unwrap();
        reader.collect::<Result<Vec<RawExample>, _>>().unwrap();
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
        let mut writer: RecordWriter<Vec<u8>, BufWriter<File>> =
            RecordWriter::create(&output_path).unwrap();

        for result in reader {
            let bytes = result.unwrap();
            writer.send(bytes).unwrap();
        }

        std::fs::remove_file(&output_path).unwrap();
    }

    // raw examples
    {
        let reader = RawExampleIter::open(&*INPUT_TFRECORD_PATH, Default::default()).unwrap();
        let mut writer: RecordWriter<RawExample, BufWriter<File>> =
            RecordWriter::create(&output_path).unwrap();

        for result in reader {
            let raw_example = result.unwrap();
            writer.send(raw_example).unwrap();
        }

        std::fs::remove_file(&output_path).unwrap();
    }

    // examples
    {
        let reader = ExampleIter::open(&*INPUT_TFRECORD_PATH, Default::default()).unwrap();
        let mut writer: RecordWriter<
            std::collections::HashMap<String, tfrecord::Feature>,
            BufWriter<File>,
        > = RecordWriter::create(&output_path).unwrap();

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
    {
        use tfrecord::Example;

        let reader = BytesIter::open(&*INPUT_TFRECORD_PATH, Default::default()).unwrap();
        reader
            .map(|result| {
                let bytes = result.unwrap();
                let raw_example = RawExample::decode(bytes.as_slice()).unwrap();
                let example = Example::from(raw_example.clone());

                // assert for RawExample
                let _: RawExample =
                    serde_json::from_str(&serde_json::to_string(&raw_example).unwrap()).unwrap();

                // assert for Example
                let _: Example =
                    serde_json::from_str(&serde_json::to_string(&example).unwrap()).unwrap();

                Result::Ok(())
            })
            .collect::<Result<Vec<_>>>()
            .unwrap();
    }
}
