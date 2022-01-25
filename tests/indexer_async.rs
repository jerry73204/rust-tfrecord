#![cfg(all(feature = "async"))]

mod common;

use common::*;
use futures::stream::{StreamExt as _, TryStreamExt as _};
use tfrecord::{Example, FeatureKind};

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

                use FeatureKind as F;
                match feature.into_kinds() {
                    Some(F::Bytes(value)) => {
                        eprintln!("bytes\t{}", value.len());
                    }
                    Some(F::F32(value)) => {
                        eprintln!("float\t{}", value.len());
                    }
                    Some(F::I64(value)) => {
                        eprintln!("int64\t{}", value.len());
                    }
                    None => {
                        eprintln!("none");
                    }
                }
            }

            Ok(())
        })
        .await?;
    Ok(())
}
