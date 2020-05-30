use std::{fs::File, io::BufWriter, path::PathBuf};

lazy_static::lazy_static! {
    pub static ref INPUT_TFRECORD_PATH: PathBuf = {
        let url = "https://storage.googleapis.com/download.tensorflow.org/data/fsns-20160927/testdata/fsns-00000-of-00001";
        let file_name = "input.tfrecord";

        let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data");
        std::fs::create_dir_all(&data_dir).unwrap();

        let out_path = data_dir.join(file_name);
        let mut out_file = BufWriter::new(File::create(&out_path).unwrap());
        reqwest::blocking::get(url).unwrap().copy_to(&mut out_file).unwrap();

        out_path
    };
    pub static ref DATA_DIR: PathBuf = {
        let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data");
        std::fs::create_dir_all(&data_dir).unwrap();
        data_dir
    };
}
