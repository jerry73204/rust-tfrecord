// NOTE
// The feature gate #![cfg(feature = "async_")] does not work here.
// We use the ugly hack to switch source code depending on the "async_" feature.

#[cfg(feature = "async_")]
mod async_example {
    use futures::stream::TryStreamExt;
    use std::{
        fs::File,
        io::{self, BufWriter},
        path::PathBuf,
    };
    use tfrecord::{DatasetInit, Error, Example, Feature};

    lazy_static::lazy_static! {
        pub static ref PATH_PREFIX: String = {
            let url = "https://storage.googleapis.com/download.tensorflow.org/data/fsns-20160927/testdata/fsns-00000-of-00001";
            let file_name = "fsns-00000-of-00001";

            let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data");
            std::fs::create_dir_all(&data_dir).unwrap();

            let out_path = data_dir.join(file_name);
            io::copy(
                &mut ureq::get(url).call().into_reader(),
                &mut BufWriter::new(File::create(&out_path).unwrap()),
            ).unwrap();

            let prefix = data_dir.join("fsns-");
            prefix.into_os_string().into_string().unwrap()
        };
        pub static ref DATA_DIR: PathBuf = {
            let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data");
            std::fs::create_dir_all(&data_dir).unwrap();
            data_dir
        };
    }

    pub async fn _main() -> Result<(), Error> {
        let num_workers = num_cpus::get();
        let dataset = DatasetInit::default().from_prefix(&*PATH_PREFIX).await?;

        // print header
        println!("worker_no\texample_no\tfeature_no\tname\ttype\tsize");

        // start multiple concurrent workers to access records
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

        Ok(())
    }
}

#[cfg(feature = "async_")]
#[async_std::main]
async fn main() -> Result<(), tfrecord::Error> {
    async_example::_main().await
}

#[cfg(not(feature = "async_"))]
fn main() {
    panic!(r#"please enable the "async_" feature to run this example"#);
}
