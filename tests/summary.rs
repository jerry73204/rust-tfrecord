mod common;

use common::*;
use rand::seq::SliceRandom;
use rand_distr::Distribution;

#[cfg(all(
    feature = "summary",
    feature = "with-image",
    feature = "with-tch",
    feature = "with-ndarray"
))]
#[test]
fn blocking_event_writer() -> Fallible<()> {
    // download image files
    let images = IMAGE_URLS
        .iter()
        .cloned()
        .map(|url| {
            println!("downloading {}", url);
            let bytes = reqwest::blocking::get(url)?.bytes()?;
            let image = image::load_from_memory(bytes.as_ref())?;
            Ok(image)
        })
        .collect::<Fallible<Vec<_>>>()?;

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
            writer.write_scalar("scalar", EventInit::with_step(step), value)?;
        }

        // histogram
        {
            let normal = Normal::new(-20.0, 50.0).unwrap();
            let values = normal
                .sample_iter(&mut rng)
                .take(1024)
                .collect::<Vec<f32>>();
            writer.write_histogram("histogram", EventInit::with_step(step), values)?;
        }

        // image
        {
            let image = images.choose(&mut rng).unwrap();
            writer.write_image("image", EventInit::with_step(step), image)?;
        }

        thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}

#[cfg(all(
    feature = "summary",
    feature = "with-image",
    feature = "with-tch",
    feature = "with-ndarray",
    feature = "async_"
))]
#[async_std::test]
async fn async_event_writer() -> Fallible<()> {
    // download image files
    let images = async_std::task::spawn_blocking(|| {
        // Because reqwest uses tokio runtime, it fails with async-std.
        // We use blocking wait instead.
        IMAGE_URLS
            .iter()
            .cloned()
            .map(|url| {
                println!("downloading {}", url);
                let bytes = reqwest::blocking::get(url)?.bytes()?;
                let image = image::load_from_memory(bytes.as_ref())?;
                Ok(image)
            })
            .collect::<Fallible<Vec<_>>>()
    })
    .await?;

    // init writer
    let prefix = DATA_DIR
        .join("async-event-writer-log-dir")
        .join("test")
        .into_os_string()
        .into_string()
        .unwrap();
    let mut writer = EventWriterInit::default()
        .from_prefix_async(prefix, None)
        .await?;
    let mut rng = rand::thread_rng();

    // loop
    for step in 0..30 {
        println!("step: {}", step);

        // scalar
        {
            let value: f32 = (step as f32 * std::f32::consts::PI / 8.0).sin();
            writer
                .write_scalar_async("scalar", EventInit::with_step(step), value)
                .await?;
        }

        // histogram
        {
            let normal = Normal::new(-20.0, 50.0).unwrap();
            let values = normal
                .sample_iter(&mut rng)
                .take(1024)
                .collect::<Vec<f32>>();
            writer
                .write_histogram_async("histogram", EventInit::with_step(step), values)
                .await?;
        }

        // image
        {
            let image = images.choose(&mut rng).unwrap();
            writer
                .write_image_async("image", EventInit::with_step(step), image)
                .await?;
        }

        async_std::task::sleep(Duration::from_millis(100)).await;
    }

    Ok(())
}
