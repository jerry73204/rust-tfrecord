mod common;

use common::*;
use rand::Rng;

#[cfg(all(feature = "async", feature = "dataset"))]
#[async_std::test]
async fn dataset_stream_test() -> Result<()> {
    let num_workers = num_cpus::get();

    for max_open_files in 0..=(num_cpus::get()) {
        let dataset = DatasetInit {
            max_open_files: NonZeroUsize::new(max_open_files), // None if zero
            ..Default::default()
        }
        .from_paths(&[&*INPUT_TFRECORD_PATH])
        .await?;

        // print header
        println!("worker_no\texample_no\tfeature_no\tname\ttype\tsize");

        let futures_iter = (0..num_workers)
            .map(|worker_index| {
                let dataset = dataset.clone();

                async move {
                    dataset
                        .stream::<Example>()
                        .try_fold(0, |example_index, example| {
                            async move {
                                // enumerate features in an example
                                for (feature_index, (name, feature)) in
                                    example.into_iter().enumerate()
                                {
                                    print!(
                                        "{}\t{}\t{}\t{}\t",
                                        worker_index, example_index, feature_index, name
                                    );

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

                                Ok(example_index + 1)
                            }
                        })
                        .await?;

                    Result::<_, Error>::Ok(())
                }
            })
            .map(async_std::task::spawn);

        futures::future::try_join_all(futures_iter).await?;
    }
    Ok(())
}

#[cfg(all(feature = "async", feature = "dataset"))]
#[async_std::test]
async fn dataset_random_access_test() -> Result<()> {
    let num_workers = num_cpus::get();

    for max_open_files in 0..=(num_cpus::get()) {
        let dataset = DatasetInit {
            max_open_files: NonZeroUsize::new(max_open_files), // None if zero
            ..Default::default()
        }
        .from_paths(&[&*INPUT_TFRECORD_PATH])
        .await?;

        // print header
        println!("worker_no\tround\texample_no\tfeature_no\tname\ttype\tsize");

        let futures_iter = (0..num_workers)
            .map(|worker_index| {
                let mut dataset = dataset.clone();

                async move {
                    let mut rng = OsRng::default();
                    let num_records = dataset.num_records();
                    ensure!(
                        dataset.get::<Example>(num_records).await? == None,
                        "unexpected Some(_)"
                    );

                    for round in 0..(num_records * 10) {
                        let example_index = rng.gen_range(0..num_records);
                        let example = dataset
                            .get::<Example>(example_index)
                            .await?
                            .ok_or_else(|| format_err!("unexpected None"))?;

                        // enumerate features in an example
                        for (feature_index, (name, feature)) in example.into_iter().enumerate() {
                            print!(
                                "{}\t{}\t{}\t{}\t{}\t",
                                worker_index, round, example_index, feature_index, name
                            );

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
                    }

                    Ok(())
                }
            })
            .map(async_std::task::spawn);

        futures::future::try_join_all(futures_iter).await?;
    }
    Ok(())
}
