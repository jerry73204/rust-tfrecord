use failure::Fallible;
use futures::stream::TryStreamExt;
use rand::Rng;
use std::path::PathBuf;
use tffile::{
    reader::{
        BytesIndexedReader, BytesReader, ExampleIndexedReader, ExampleReader, IndexedReaderInit,
        RecordReaderInit, RecordStreamInit,
    },
    Example,
};

lazy_static::lazy_static! {
    static ref INPUT_TFRECORD_PATH: PathBuf = {
        use std::{fs::File, io::BufWriter};

        let url = "https://storage.googleapis.com/download.tensorflow.org/data/fsns-20160927/testdata/fsns-00000-of-00001";
        let file_name = "input.tfrecord";

        let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data");
        std::fs::create_dir_all(&data_dir).unwrap();

        let out_path = data_dir.join(file_name);
        let mut out_file = BufWriter::new(File::create(&out_path).unwrap());
        reqwest::blocking::get(url).unwrap().copy_to(&mut out_file).unwrap();

        out_path
    };
    static ref OUTPUT_TFRECORD_PATH: PathBuf = {
        let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data");
        std::fs::create_dir_all(&data_dir).unwrap();
        data_dir.join("output.tfrecord")
    };
}

#[test]
fn blocking_reader_test() -> Fallible<()> {
    {
        let reader: BytesReader<_> = RecordReaderInit {
            check_integrity: true,
        }
        .open(&*INPUT_TFRECORD_PATH)?;
        reader.collect::<Result<Vec<Vec<u8>>, _>>()?;
    }

    {
        let reader: ExampleReader<_> = RecordReaderInit {
            check_integrity: true,
        }
        .open(&*INPUT_TFRECORD_PATH)?;
        reader.collect::<Result<Vec<Example>, _>>()?;
    }

    Ok(())
}

#[async_std::test]
async fn async_stream_test() -> Fallible<()> {
    use async_std::{fs::File, io::BufReader};

    {
        let reader = BufReader::new(File::open(&*INPUT_TFRECORD_PATH).await?);
        let stream = RecordStreamInit {
            check_integrity: true,
        }
        .bytes_from_reader(reader)
        .await?;
        stream.try_collect::<Vec<Vec<u8>>>().await?;
    }

    {
        let reader = BufReader::new(File::open(&*INPUT_TFRECORD_PATH).await?);
        let stream = RecordStreamInit {
            check_integrity: true,
        }
        .examples_from_reader(reader)
        .await?;
        stream.try_collect::<Vec<Example>>().await?;
    }

    Ok(())
}

#[test]
fn blocking_indexed_reader_test() -> Fallible<()> {
    let mut rng = rand::thread_rng();

    {
        let mut reader: BytesIndexedReader<_> = IndexedReaderInit {
            check_integrity: true,
        }
        .open(&*INPUT_TFRECORD_PATH)?;

        let num_records = reader.num_records();

        for _ in 0..(num_records * 100) {
            let index = rng.gen_range(0, num_records);
            let _: Vec<u8> = reader.get(index)?.unwrap();
        }
    }

    let mut rng = rand::thread_rng();

    {
        let mut reader: ExampleIndexedReader<_> = IndexedReaderInit {
            check_integrity: true,
        }
        .open(&*INPUT_TFRECORD_PATH)?;

        let num_records = reader.num_records();

        for _ in 0..(num_records * 100) {
            let index = rng.gen_range(0, num_records);
            let _: Example = reader.get(index)?.unwrap();
        }
    }

    Ok(())
}

#[async_std::test]
async fn async_indexed_reader_test() -> Fallible<()> {
    use async_std::{fs::File, io::BufReader};
    let mut rng = rand::thread_rng();

    {
        let rd = BufReader::new(File::open(&*INPUT_TFRECORD_PATH).await?);
        let mut reader: BytesIndexedReader<_> = IndexedReaderInit {
            check_integrity: true,
        }
        .from_async_reader(rd)
        .await?;

        let num_records = reader.num_records();

        for _ in 0..(num_records * 100) {
            let index = rng.gen_range(0, num_records);
            let _: Vec<u8> = reader.get_async(index).await?.unwrap();
        }
    }

    {
        let rd = BufReader::new(File::open(&*INPUT_TFRECORD_PATH).await?);
        let mut reader: ExampleIndexedReader<_> = IndexedReaderInit {
            check_integrity: true,
        }
        .from_async_reader(rd)
        .await?;

        let num_records = reader.num_records();

        for _ in 0..(num_records * 100) {
            let index = rng.gen_range(0, num_records);
            let _: Example = reader.get_async(index).await?.unwrap();
        }
    }

    Ok(())
}
