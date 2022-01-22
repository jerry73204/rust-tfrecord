#![cfg(all(
    feature = "summary",
    feature = "with-image",
    feature = "with-tch",
    feature = "with-ndarray"
))]

mod common;

use common::*;
use rand::seq::SliceRandom;
use rand_distr::Distribution;
use tfrecord::{EventMeta, EventWriterInit};

#[test]
fn event_writer() -> Result<()> {
    // download image files
    let images = IMAGE_URLS
        .iter()
        .cloned()
        .map(|url| {
            println!("downloading {}", url);
            let mut bytes = vec![];
            io::copy(&mut ureq::get(url).call()?.into_reader(), &mut bytes)?;
            let image = image::load_from_memory(bytes.as_ref())?;
            Ok(image)
        })
        .collect::<Result<Vec<_>>>()?;

    // init writer
    let prefix = DATA_DIR
        .join("blocking-event-writer-log-dir")
        .join("test")
        .into_os_string()
        .into_string()
        .unwrap();
    let mut writer = EventWriterInit::default().from_prefix(prefix, None)?;
    let mut rng = rand::thread_rng();

    // loop
    for step in 0..30 {
        println!("step: {}", step);

        // scalar
        {
            let value: f32 = (step as f32 * std::f32::consts::PI / 8.0).sin();
            writer.write_scalar("scalar", step, value)?; // with step and default wall time
        }

        // histogram
        {
            let normal = Normal::new(-20.0, 50.0).unwrap();
            let values = normal
                .sample_iter(&mut rng)
                .take(1024)
                .collect::<Vec<f32>>();
            writer.write_histogram("histogram", EventMeta::with_step(step), values)?;
            // more verbose writing
        }

        // image
        {
            let image = images.choose(&mut rng).unwrap();
            writer.write_image("image", (step, SystemTime::now()), image)?; // with step and specified wall time
        }

        thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}
