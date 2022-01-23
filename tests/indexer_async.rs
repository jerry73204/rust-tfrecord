#![cfg(all(feature = "async"))]

mod common;

use common::*;
use tfrecord::{Example, Feature};

#[async_std::test]
async fn indexer_stream_test() -> Result<()> {
    tfrecord::indexer::load_paths_async([&*INPUT_TFRECORD_PATH], Default::default())
        .and_then(|index| async move {
            let example: Example = index.load_async().await?;
            Ok(example)
        })
        .enumerate()
        .map(|(index, example)| anyhow::Ok((index, example?)))
        .try_for_each(|(example_index, example)| async move {
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

            Ok(())
        })
        .await?;
    Ok(())
}
