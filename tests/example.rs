use failure::Fallible;
use std::path::PathBuf;
use tffile::reader::{DoCheck, NoCheck, ReaderOptions, RuntimeCheck};

#[test]
fn record_reader_test() -> Fallible<()> {
    let path = PathBuf::from("/mnt/wd/home/aeon/gqn-dataset/rooms_free_camera_with_object_rotations/train/0001-of-2034.tfrecord");

    // static checked
    ReaderOptions::new()
        .record_reader_open::<DoCheck, _>(&path)?
        .collect::<Result<Vec<_>, _>>()?;

    // static unchecked
    ReaderOptions::new()
        .record_reader_open::<NoCheck, _>(&path)?
        .collect::<Result<Vec<_>, _>>()?;

    // dynamic
    ReaderOptions::new()
        .check_integrity(true)
        .record_reader_open::<RuntimeCheck, _>(&path)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}

#[test]
fn example_reader_test() -> Fallible<()> {
    let path = PathBuf::from("/mnt/wd/home/aeon/gqn-dataset/rooms_free_camera_with_object_rotations/train/0001-of-2034.tfrecord");

    // static checked
    ReaderOptions::new()
        .example_reader_open::<DoCheck, _>(&path)?
        .collect::<Result<Vec<_>, _>>()?;

    // static unchecked
    ReaderOptions::new()
        .example_reader_open::<NoCheck, _>(&path)?
        .collect::<Result<Vec<_>, _>>()?;

    // dynamic
    ReaderOptions::new()
        .check_integrity(true)
        .example_reader_open::<RuntimeCheck, _>(&path)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}

#[test]
fn record_indexer_test() -> Fallible<()> {
    let path = PathBuf::from("/mnt/wd/home/aeon/gqn-dataset/rooms_free_camera_with_object_rotations/train/0001-of-2034.tfrecord");

    // static checked
    ReaderOptions::new()
        .record_indexer_open::<DoCheck, _>(&path)?
        .collect::<Result<Vec<_>, _>>()?;

    // static unchecked
    ReaderOptions::new()
        .record_indexer_open::<NoCheck, _>(&path)?
        .collect::<Result<Vec<_>, _>>()?;

    // dynamic
    ReaderOptions::new()
        .check_integrity(true)
        .record_indexer_open::<RuntimeCheck, _>(&path)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}
