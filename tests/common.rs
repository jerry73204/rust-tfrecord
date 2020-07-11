pub use failure::{ensure, format_err, Fallible};
#[cfg(feature = "async_")]
pub use futures::stream::TryStreamExt;
#[cfg(feature = "serde")]
pub use prost::Message;
pub use rand::rngs::OsRng;
pub use rand_distr::Normal;
pub use std::{
    fs::File,
    io::BufWriter,
    io::Cursor,
    num::NonZeroUsize,
    path::PathBuf,
    thread,
    time::{Duration, SystemTime},
};
#[cfg(feature = "async_")]
pub use tfrecord::RecordStreamInit;
pub use tfrecord::{
    BytesReader, BytesWriter, Example, ExampleReader, ExampleWriter, Feature, RawExample,
    RawExampleReader, RawExampleWriter, RecordReaderInit, RecordWriterInit,
};
#[cfg(feature = "dataset")]
pub use tfrecord::{Dataset, DatasetInit};
#[cfg(feature = "summary")]
pub use tfrecord::{EventInit, EventWriterInit};

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

    pub static ref IMAGE_URLS: &'static [&'static str] = &[
        "https://farm3.staticflickr.com/2564/3946548112_77df49fe87_z.jpg",
        "https://farm6.staticflickr.com/5268/5797374366_ee43848f1f_z.jpg",
        "https://farm1.staticflickr.com/103/364045222_7e633c5ee5_z.jpg",
        "https://farm9.staticflickr.com/8124/8661208796_8d4b11beb3_z.jpg",
        "https://farm8.staticflickr.com/7296/9325926467_6f63b51a07_z.jpg",
        "https://farm3.staticflickr.com/2399/2210469536_c37a1bbf9a_z.jpg",
        "https://farm8.staticflickr.com/7033/6623810681_c8ffef796d_z.jpg",
        "https://farm4.staticflickr.com/3658/3574393179_088a317bca_z.jpg",
        "https://farm9.staticflickr.com/8126/8687827938_c26eb7e685_z.jpg",
        "https://farm1.staticflickr.com/72/162225436_fa7abc6a2d_z.jpg",
        "https://farm5.staticflickr.com/4037/4579942190_549048649d_z.jpg",
        "https://farm1.staticflickr.com/152/421839397_f95f1e5f12_z.jpg",
        "https://farm9.staticflickr.com/8389/8489230096_2fb2838311_z.jpg",
        "https://farm3.staticflickr.com/2123/2282016642_bf8fe494c9_z.jpg",
        "https://farm2.staticflickr.com/1366/566150996_9fac6f9b91_z.jpg",
        "https://farm1.staticflickr.com/173/419814278_76be492b37_z.jpg",
        "https://farm4.staticflickr.com/3527/3750588052_b8dd9d575b_z.jpg",
        "https://farm5.staticflickr.com/4073/5442176663_5cf23cc11a_z.jpg",
        "https://farm5.staticflickr.com/4082/4822336152_10d3e70081_z.jpg",
        "https://farm4.staticflickr.com/3663/3403988687_0de6ce12d4_z.jpg",
        "https://farm4.staticflickr.com/3226/2653462544_c01b97d003_z.jpg",
        "https://farm3.staticflickr.com/2250/1806745281_ca3986a6c8_z.jpg",
        "https://farm9.staticflickr.com/8348/8240183996_f7b0f2ddf1_z.jpg",
        "https://farm3.staticflickr.com/2018/1971396018_84991590d1_z.jpg",
        "https://farm9.staticflickr.com/8017/7155768195_d01b835c71_z.jpg",
        "https://farm4.staticflickr.com/3708/9374479963_4444ab75a0_z.jpg",
        "https://farm1.staticflickr.com/171/405321265_fb25fff175_z.jpg",
        "https://farm3.staticflickr.com/2123/2198446823_85c691081c_z.jpg",
        "https://farm9.staticflickr.com/8339/8231329597_1b9934b714_z.jpg",
        "https://farm4.staticflickr.com/3729/9437410428_5f12f85913_z.jpg",
    ];

}
