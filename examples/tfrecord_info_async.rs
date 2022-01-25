#[cfg(feature = "async")]
mod async_example {
    use futures::stream::TryStreamExt;
    use std::{
        fs::File,
        io::{self, BufWriter},
        path::PathBuf,
    };
    use tfrecord::{Error, ExampleStream, FeatureKind};

    lazy_static::lazy_static! {
        pub static ref INPUT_TFRECORD_PATH: PathBuf = {
            let url = "https://storage.googleapis.com/download.tensorflow.org/data/fsns-20160927/testdata/fsns-00000-of-00001";
            let file_name = "input.tfrecord";

            let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data");
            std::fs::create_dir_all(&data_dir).unwrap();

            let out_path = data_dir.join(file_name);
            io::copy(
                &mut ureq::get(url).call().unwrap().into_reader(),
                &mut BufWriter::new(File::create(&out_path).unwrap()),
            ).unwrap();

            out_path
        };
        pub static ref DATA_DIR: PathBuf = {
            let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data");
            std::fs::create_dir_all(&data_dir).unwrap();
            data_dir
        };
    }

    pub async fn _main() -> Result<(), Error> {
        // use init pattern to construct the tfrecord stream
        let stream = ExampleStream::open(&*INPUT_TFRECORD_PATH, Default::default()).await?;

        // print header
        println!("example_no\tfeature_no\tname\ttype\tsize");

        // enumerate examples
        stream
            .try_fold(0, |example_index, example| {
                async move {
                    // enumerate features in an example
                    for (feature_index, (name, feature)) in example.into_iter().enumerate() {
                        print!("{}\t{}\t{}\t", example_index, feature_index, name);

                        use FeatureKind as F;
                        match feature.into_kinds() {
                            Some(F::Bytes(list)) => {
                                println!("bytes\t{}", list.len());
                            }
                            Some(F::F32(list)) => {
                                println!("float\t{}", list.len());
                            }
                            Some(F::I64(list)) => {
                                println!("int64\t{}", list.len());
                            }
                            None => {
                                println!("none");
                            }
                        }
                    }

                    Ok(example_index + 1)
                }
            })
            .await?;

        Ok(())
    }
}

#[cfg(feature = "async")]
#[async_std::main]
async fn main() -> Result<(), tfrecord::Error> {
    async_example::_main().await
}

#[cfg(not(feature = "async"))]
fn main() {
    panic!(r#"please enable the "async" feature to run this example"#);
}
