mod common;

use common::*;

#[test]
fn blocking_reader_test() -> Result<()> {
    // bytes
    {
        let reader: BytesReader<_> = RecordReaderInit::default().open(&*INPUT_TFRECORD_PATH)?;
        reader.collect::<Result<Vec<Vec<u8>>, _>>()?;
    }

    // raw examples
    {
        let reader: RawExampleReader<_> =
            RecordReaderInit::default().open(&*INPUT_TFRECORD_PATH)?;
        reader.collect::<Result<Vec<RawExample>, _>>()?;
    }

    // examples
    {
        let reader: ExampleReader<_> = RecordReaderInit::default().open(&*INPUT_TFRECORD_PATH)?;
        reader.collect::<Result<Vec<Example>, _>>()?;
    }

    Ok(())
}

#[cfg(feature = "async")]
#[async_std::test]
async fn async_stream_test() -> Result<()> {
    // bytes
    {
        let stream = RecordStreamInit::default()
            .bytes_open(&*INPUT_TFRECORD_PATH)
            .await?;
        stream.try_collect::<Vec<Vec<u8>>>().await?;
    }

    // raw examples
    {
        let stream = RecordStreamInit::default()
            .raw_examples_open(&*INPUT_TFRECORD_PATH)
            .await?;
        stream.try_collect::<Vec<RawExample>>().await?;
    }

    // examples
    {
        let stream = RecordStreamInit::default()
            .examples_open(&*INPUT_TFRECORD_PATH)
            .await?;
        stream.try_collect::<Vec<Example>>().await?;
    }

    Ok(())
}

#[test]
fn blocking_writer_test() -> Result<()> {
    let output_path = DATA_DIR.join("blocking_writer_output.tfrecord");

    // bytes
    {
        let reader: BytesReader<_> = RecordReaderInit::default().open(&*INPUT_TFRECORD_PATH)?;
        let mut writer: BytesWriter<_> = RecordWriterInit::create(&output_path)?;

        for result in reader {
            let bytes = result?;
            writer.send(bytes)?;
        }

        std::fs::remove_file(&output_path)?;
    }

    // raw examples
    {
        let reader: RawExampleReader<_> =
            RecordReaderInit::default().open(&*INPUT_TFRECORD_PATH)?;
        let mut writer: RawExampleWriter<_> = RecordWriterInit::create(&output_path)?;

        for result in reader {
            let raw_example = result?;
            writer.send(raw_example)?;
        }

        std::fs::remove_file(&output_path)?;
    }

    // examples
    {
        let reader: ExampleReader<_> = RecordReaderInit::default().open(&*INPUT_TFRECORD_PATH)?;
        let mut writer: ExampleWriter<_> = RecordWriterInit::create(&output_path)?;

        for result in reader {
            let example = result?;
            writer.send(example)?;
        }

        std::fs::remove_file(&output_path)?;
    }

    Ok(())
}

#[cfg(feature = "async")]
#[async_std::test]
async fn async_writer_test() -> Result<()> {
    let output_path = DATA_DIR.join("async_writer_output.tfrecord");

    // bytes
    {
        let stream = RecordStreamInit::default()
            .bytes_open(&*INPUT_TFRECORD_PATH)
            .await?;
        let writer: BytesWriter<_> = RecordWriterInit::create_async(&output_path).await?;

        stream
            .try_fold(writer, |mut writer, bytes| async {
                writer.send_async(bytes).await?;
                Ok(writer)
            })
            .await?;

        async_std::fs::remove_file(&output_path).await?;
    }

    // raw examples
    {
        let stream = RecordStreamInit::default()
            .raw_examples_open(&*INPUT_TFRECORD_PATH)
            .await?;
        let writer: RawExampleWriter<_> = RecordWriterInit::create_async(&output_path).await?;

        stream
            .try_fold(writer, |mut writer, example| async {
                writer.send_async(example).await?;
                Ok(writer)
            })
            .await?;

        async_std::fs::remove_file(&output_path).await?;
    }

    // examples
    {
        let stream = RecordStreamInit::default()
            .examples_open(&*INPUT_TFRECORD_PATH)
            .await?;
        let writer: ExampleWriter<_> = RecordWriterInit::create_async(&output_path).await?;

        stream
            .try_fold(writer, |mut writer, example| async {
                writer.send_async(example).await?;
                Ok(writer)
            })
            .await?;

        async_std::fs::remove_file(&output_path).await?;
    }

    Ok(())
}

#[cfg(feature = "serde")]
#[test]
fn serde_test() -> Result<()> {
    {
        let reader: BytesReader<_> = RecordReaderInit::default().open(&*INPUT_TFRECORD_PATH)?;
        reader
            .map(|result| {
                let bytes = result?;
                let raw_example = RawExample::decode(bytes.as_slice())?;
                let example = Example::from(&raw_example);

                // assert for RawExample
                let _: RawExample = serde_json::from_str(&serde_json::to_string(&raw_example)?)?;

                // assert for Example
                let _: Example = serde_json::from_str(&serde_json::to_string(&example)?)?;

                Result::Ok(())
            })
            .collect::<Result<Vec<_>>>()?;
    }

    Ok(())
}
