mod common;

use common::*;
use tfrecord::{Example, Feature};

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

            anyhow::Ok(())
        })?;
    Ok(())
}
