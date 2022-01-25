mod common;

use common::*;
use tfrecord::{Example, FeatureKind};

#[test]
fn indexer_iter_test() -> Result<()> {
    tfrecord::indexer::load_paths([&*INPUT_TFRECORD_PATH], Default::default())
        .map(|index| {
            let example: Example = index?.load()?;
            anyhow::Ok(example)
        })
        .enumerate()
        .map(|(index, example)| anyhow::Ok((index, example?)))
        .try_for_each(|args| {
            let (example_index, example) = args?;

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

            anyhow::Ok(())
        })?;
    Ok(())
}
