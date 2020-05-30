use failure::Fallible;
#[cfg(feature = "async_")]
use futures::stream::TryStreamExt;
#[cfg(feature = "serde")]
use prost::Message;
use std::{fs::File, io::BufWriter, path::PathBuf};
#[cfg(feature = "async_")]
use tfrecord::RecordStreamInit;
use tfrecord::{
    BytesReader, BytesWriter, Example, ExampleReader, ExampleWriter, RawExample, RawExampleReader,
    RawExampleWriter, RecordReaderInit, RecordWriterInit,
};

lazy_static::lazy_static! {
    static ref INPUT_TFRECORD_PATH: PathBuf = {

        let url = "https://storage.googleapis.com/download.tensorflow.org/data/fsns-20160927/testdata/fsns-00000-of-00001";
        let file_name = "input.tfrecord";

        let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data");
        std::fs::create_dir_all(&data_dir).unwrap();

        let out_path = data_dir.join(file_name);
        let mut out_file = BufWriter::new(File::create(&out_path).unwrap());
        reqwest::blocking::get(url).unwrap().copy_to(&mut out_file).unwrap();

        out_path
    };
    static ref DATA_DIR: PathBuf = {
        let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data");
        std::fs::create_dir_all(&data_dir).unwrap();
        data_dir
    };
}

#[test]
fn blocking_reader_test() -> Fallible<()> {
    // bytes
    {
        let reader: BytesReader<_> = RecordReaderInit {
            check_integrity: true,
        }
        .open(&*INPUT_TFRECORD_PATH)?;
        reader.collect::<Result<Vec<Vec<u8>>, _>>()?;
    }

    // raw examples
    {
        let reader: RawExampleReader<_> = RecordReaderInit {
            check_integrity: true,
        }
        .open(&*INPUT_TFRECORD_PATH)?;
        reader.collect::<Result<Vec<RawExample>, _>>()?;
    }

    // examples
    {
        let reader: ExampleReader<_> = RecordReaderInit {
            check_integrity: true,
        }
        .open(&*INPUT_TFRECORD_PATH)?;
        reader.collect::<Result<Vec<Example>, _>>()?;
    }

    Ok(())
}

#[cfg(feature = "async_")]
#[async_std::test]
async fn async_stream_test() -> Fallible<()> {
    // bytes
    {
        let stream = RecordStreamInit {
            check_integrity: true,
        }
        .bytes_open(&*INPUT_TFRECORD_PATH)
        .await?;
        stream.try_collect::<Vec<Vec<u8>>>().await?;
    }

    // raw examples
    {
        let stream = RecordStreamInit {
            check_integrity: true,
        }
        .raw_examples_open(&*INPUT_TFRECORD_PATH)
        .await?;
        stream.try_collect::<Vec<RawExample>>().await?;
    }

    // examples
    {
        let stream = RecordStreamInit {
            check_integrity: true,
        }
        .examples_open(&*INPUT_TFRECORD_PATH)
        .await?;
        stream.try_collect::<Vec<Example>>().await?;
    }

    Ok(())
}

#[test]
fn blocking_writer_test() -> Fallible<()> {
    let output_path = DATA_DIR.join("blocking_writer_output.tfrecord");

    // bytes
    {
        let reader: BytesReader<_> = RecordReaderInit {
            check_integrity: true,
        }
        .open(&*INPUT_TFRECORD_PATH)?;
        let mut writer: BytesWriter<_> = RecordWriterInit::create(&output_path)?;

        for result in reader {
            let bytes = result?;
            writer.send(bytes)?;
        }

        std::fs::remove_file(&output_path)?;
    }

    // raw examples
    {
        let reader: ExampleReader<_> = RecordReaderInit {
            check_integrity: true,
        }
        .open(&*INPUT_TFRECORD_PATH)?;
        let mut writer: ExampleWriter<_> = RecordWriterInit::create(&output_path)?;

        for result in reader {
            let example = result?;
            writer.send(example)?;
        }

        std::fs::remove_file(&output_path)?;
    }

    // examples
    {
        let reader: ExampleReader<_> = RecordReaderInit {
            check_integrity: true,
        }
        .open(&*INPUT_TFRECORD_PATH)?;
        let mut writer: ExampleWriter<_> = RecordWriterInit::create(&output_path)?;

        for result in reader {
            let example = result?;
            writer.send(example)?;
        }

        std::fs::remove_file(&output_path)?;
    }

    Ok(())
}

#[cfg(feature = "async_")]
#[async_std::test]
async fn async_writer_test() -> Fallible<()> {
    let output_path = DATA_DIR.join("async_writer_output.tfrecord");

    // bytes
    {
        let stream = RecordStreamInit {
            check_integrity: true,
        }
        .bytes_open(&*INPUT_TFRECORD_PATH)
        .await?;
        let writer: BytesWriter<_> = RecordWriterInit::create_async(&output_path).await?;

        stream
            .try_fold(writer, |mut writer, bytes| {
                async {
                    writer.send_async(bytes).await?;
                    Ok(writer)
                }
            })
            .await?;

        async_std::fs::remove_file(&output_path).await?;
    }

    // raw examples
    {
        let stream = RecordStreamInit {
            check_integrity: true,
        }
        .raw_examples_open(&*INPUT_TFRECORD_PATH)
        .await?;
        let writer: RawExampleWriter<_> = RecordWriterInit::create_async(&output_path).await?;

        stream
            .try_fold(writer, |mut writer, example| {
                async {
                    writer.send_async(example).await?;
                    Ok(writer)
                }
            })
            .await?;

        async_std::fs::remove_file(&output_path).await?;
    }

    // examples
    {
        let stream = RecordStreamInit {
            check_integrity: true,
        }
        .examples_open(&*INPUT_TFRECORD_PATH)
        .await?;
        let writer: ExampleWriter<_> = RecordWriterInit::create_async(&output_path).await?;

        stream
            .try_fold(writer, |mut writer, example| {
                async {
                    writer.send_async(example).await?;
                    Ok(writer)
                }
            })
            .await?;

        async_std::fs::remove_file(&output_path).await?;
    }

    Ok(())
}

#[cfg(feature = "serde")]
#[test]
fn serde_test() -> Fallible<()> {
    {
        let reader: BytesReader<_> = RecordReaderInit {
            check_integrity: true,
        }
        .open(&*INPUT_TFRECORD_PATH)?;
        reader
            .map(|result| {
                let bytes = result?;
                let raw_example = RawExample::decode(bytes.as_slice())?;
                let example = Example::from(&raw_example);

                // assert for RawExample
                let _: RawExample = serde_json::from_str(&serde_json::to_string(&raw_example)?)?;

                // assert for Example
                let _: Example = serde_json::from_str(&serde_json::to_string(&example)?)?;

                Fallible::Ok(())
            })
            .collect::<Fallible<Vec<_>>>()?;
    }

    Ok(())
}
