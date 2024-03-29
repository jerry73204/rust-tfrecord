#![cfg(all(
    feature = "async",
    feature = "summary",
    feature = "with-image",
    feature = "with-tch",
    feature = "with-ndarray"
))]

mod common;

use common::*;
use rand::seq::SliceRandom;
use rand_distr::Distribution;
use tfrecord::{EventMeta, EventWriter};

#[async_std::test]
async fn async_event_writer() -> Result<()> {
    // download image files
    // ureq blocks the thread so let's wrap in spawn_blocking
    let images = async_std::task::spawn_blocking(|| {
        IMAGE_URLS
            .iter()
            .cloned()
            .map(|url| {
                println!("downloading {}", url);
                let mut bytes = vec![];
                io::copy(&mut ureq::get(url).call()?.into_reader(), &mut bytes)?;
                let image = image::load_from_memory(bytes.as_ref())?;
                Ok(image)
            })
            .collect::<Result<Vec<_>>>()
    })
    .await?;

    // init writer
    let prefix = DATA_DIR
        .join("async-event-writer-log-dir")
        .join("test")
        .into_os_string()
        .into_string()
        .unwrap();
    let mut writer = EventWriter::from_prefix_async(prefix, "", Default::default()).await?;
    let mut rng = rand::thread_rng();

    // loop
    for step in 0..30 {
        println!("step: {}", step);

        // scalar
        {
            let value: f32 = (step as f32 * std::f32::consts::PI / 8.0).sin();
            writer.write_scalar_async("scalar", step, value).await?; // with step and default wall time
        }

        // histogram
        {
            let normal = Normal::new(-20.0, 50.0).unwrap();
            let values = normal
                .sample_iter(&mut rng)
                .take(1024)
                .collect::<Vec<f32>>();
            writer
                .write_histogram_async("histogram", EventMeta::with_step(step), values) // more verbose writing
                .await?;
        }

        // image
        {
            let image = images.choose(&mut rng).unwrap();
            writer
                .write_image_async("image", (step, SystemTime::now()), image) // with step and specified wall time
                .await?;
        }

        async_std::task::sleep(Duration::from_millis(100)).await;
    }

    Ok(())
}
