use anyhow::{ensure, Result};
use flate2::read::GzDecoder;
use itertools::izip;
use packed_struct::prelude::*;
use packed_struct_codegen::PackedStruct;
use std::io::{prelude::*, Cursor};
use tfrecord::{Example, ExampleWriter, Feature, RecordWriterInit};

const IMAGES_URL: &'static str = "http://yann.lecun.com/exdb/mnist/train-images-idx3-ubyte.gz";
const LABELS_URL: &'static str = "http://yann.lecun.com/exdb/mnist/train-labels-idx1-ubyte.gz";
const OUTPUT_FILE: &'static str = "mnist.tfrecord";

fn main() -> Result<()> {
    // download and decode data set
    let (images, labels) = mnist_loader::load_mnist()?;

    // writer to tfrecord file
    let mut writer: ExampleWriter<_> = RecordWriterInit::create(OUTPUT_FILE)?;

    for (image, label) in izip!(images, labels) {
        // build example
        let image_feature = Feature::FloatList(
            image
                .into_iter()
                .map(|pixel| pixel as f32)
                .collect::<Vec<_>>(),
        );
        let label_feature = Feature::Int64List(vec![label as i64]);

        let example = vec![
            ("image".into(), image_feature),
            ("label".into(), label_feature),
        ]
        .into_iter()
        .collect::<Example>();

        // append to file
        writer.send(example)?;
    }

    // finalize
    println!("tfrecord file is written to {}", OUTPUT_FILE);

    Ok(())
}

mod mnist_loader {
    use super::*;

    #[derive(PackedStruct)]
    #[packed_struct(endian = "msb")]
    pub struct LabelHeader {
        pub magic: u32,
        pub num_labels: u32,
    }

    #[derive(PackedStruct)]
    #[packed_struct(endian = "msb")]
    pub struct ImageHeader {
        pub magic: u32,
        pub num_images: u32,
        pub num_rows: u32,
        pub num_cols: u32,
    }

    pub fn load_mnist() -> Result<(Vec<Vec<u8>>, Vec<u8>)> {
        let images_bytes = download_url(IMAGES_URL)?;
        let labels_bytes = download_url(LABELS_URL)?;

        let images = parse_images_bytes(&images_bytes)?;
        let labels = parse_labels_bytes(&labels_bytes)?;
        ensure!(images.len() == labels.len(), "the data set is corrupted");

        Ok((images, labels))
    }

    fn parse_images_bytes(bytes: &[u8]) -> Result<Vec<Vec<u8>>> {
        // decode header
        let ImageHeader {
            magic,
            num_images,
            num_rows,
            num_cols,
        } = ImageHeader::unpack_from_slice(&bytes[0..ImageHeader::packed_bytes()])?;
        ensure!(magic == 0x00000803, "the data set is corrupted");
        ensure!(
            bytes.len()
                == ImageHeader::packed_bytes() + (num_images * num_rows * num_cols) as usize,
            "the data set is corrupted"
        );

        // decode images
        let images = (0..num_images)
            .scan(ImageHeader::packed_bytes(), |offset, _| {
                let size = (num_rows * num_cols) as usize;
                let begin_offset = *offset;
                let end_offset = *offset + size;
                *offset = end_offset;
                Some((begin_offset, end_offset))
            })
            .map(|(begin_offset, end_offset)| Vec::from(&bytes[begin_offset..end_offset]))
            .collect::<Vec<_>>();

        Ok(images)
    }

    fn parse_labels_bytes(bytes: &[u8]) -> Result<Vec<u8>> {
        // decode header
        let LabelHeader { magic, num_labels } =
            LabelHeader::unpack_from_slice(&bytes[0..LabelHeader::packed_bytes()])?;
        ensure!(magic == 0x00000801, "the data set is corrupted");
        ensure!(
            bytes.len() == LabelHeader::packed_bytes() + num_labels as usize,
            "the data set is corrupted"
        );

        // decode labels
        let begin_offset = LabelHeader::packed_bytes();
        let end_offset = begin_offset + num_labels as usize;
        let labels = Vec::from(&bytes[begin_offset..end_offset]);

        Ok(labels)
    }

    fn download_url(url: &str) -> Result<Vec<u8>> {
        println!("downloading {}", url);
        let bytes = reqwest::blocking::get(url)?.bytes()?;
        let cursor = Cursor::new(bytes.as_ref());
        let mut decoder = GzDecoder::new(cursor);

        let mut buf = vec![];
        decoder.read_to_end(&mut buf)?;
        Ok(buf)
    }
}
