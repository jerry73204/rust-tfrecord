#![cfg(feature = "async")]

mod common;

use common::*;
use futures::stream::TryStreamExt as _;
use tfrecord::{
    BytesStream, BytesWriter, Example, ExampleStream, ExampleWriter, RawExample, RawExampleStream,
    RawExampleWriter,
};

#[async_std::test]
async fn stream_test() {
    // bytes
    {
        let stream = BytesStream::open(&*INPUT_TFRECORD_PATH, Default::default())
            .await
            .unwrap();
        stream.try_collect::<Vec<Vec<u8>>>().await.unwrap();
    }

    // raw examples
    {
        let stream = RawExampleStream::open(&*INPUT_TFRECORD_PATH, Default::default())
            .await
            .unwrap();
        stream.try_collect::<Vec<RawExample>>().await.unwrap();
    }

    // examples
    {
        let stream = ExampleStream::open(&*INPUT_TFRECORD_PATH, Default::default())
            .await
            .unwrap();
        stream.try_collect::<Vec<Example>>().await.unwrap();
    }
}

#[async_std::test]
async fn async_writer_test() {
    let output_path = DATA_DIR.join("async_writer_output.tfrecord");

    // bytes
    {
        let stream = BytesStream::open(&*INPUT_TFRECORD_PATH, Default::default())
            .await
            .unwrap();
        let writer = BytesWriter::create_async(&output_path).await.unwrap();

        stream
            .try_fold(writer, |mut writer, bytes| async {
                writer.send_async(bytes).await.unwrap();
                Ok(writer)
            })
            .await
            .unwrap();

        async_std::fs::remove_file(&output_path).await.unwrap();
    }

    // raw examples
    {
        let stream = RawExampleStream::open(&*INPUT_TFRECORD_PATH, Default::default())
            .await
            .unwrap();
        let writer = RawExampleWriter::create_async(&output_path).await.unwrap();

        stream
            .try_fold(writer, |mut writer, example| async {
                writer.send_async(example).await.unwrap();
                Ok(writer)
            })
            .await
            .unwrap();

        async_std::fs::remove_file(&output_path).await.unwrap();
    }

    // examples
    {
        let stream = ExampleStream::open(&*INPUT_TFRECORD_PATH, Default::default())
            .await
            .unwrap();
        let writer = ExampleWriter::create_async(&output_path).await.unwrap();

        stream
            .try_fold(writer, |mut writer, example| async {
                writer.send_async(example).await.unwrap();
                Ok(writer)
            })
            .await
            .unwrap();

        async_std::fs::remove_file(&output_path).await.unwrap();
    }
}
