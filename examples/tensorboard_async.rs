use failure::Fallible;
use std::path::PathBuf;

lazy_static::lazy_static! {
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
    pub static ref DATA_DIR: PathBuf = {
        let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data");
        std::fs::create_dir_all(&data_dir).unwrap();
        data_dir
    };
}

#[cfg(feature = "full")]
mod async_example {
    use super::*;
    use rand::seq::SliceRandom;
    use rand_distr::{Distribution, Normal};
    use std::{f32::consts::PI, time::Duration};
    use tfrecord::{EventInit, EventWriterInit};

    pub async fn _main() -> Fallible<()> {
        // show log dir
        let log_dir = DATA_DIR.join("tensorboard_log_dir");
        let prefix = log_dir
            .join("tensorboard_example")
            .into_os_string()
            .into_string()
            .unwrap();
        println!(
            r#"Run `tensorboard --logdir '{}'` to watch the output"#,
            log_dir.display()
        );

        // download image files
        println!("downloading images...");
        let images = async_std::task::spawn_blocking(|| {
            // Because reqwest uses tokio runtime, it fails with async-std.
            // We use blocking wait instead.
            IMAGE_URLS
                .iter()
                .cloned()
                .map(|url| {
                    let bytes = reqwest::blocking::get(url)?.bytes()?;
                    let image = image::load_from_memory(bytes.as_ref())?;
                    Ok(image)
                })
                .collect::<Fallible<Vec<_>>>()
        })
        .await?;

        // init writer
        let mut writer = EventWriterInit::from_prefix_async(prefix, None).await?;
        let mut rng = rand::thread_rng();

        // loop
        for step in 0..30 {
            println!("step: {}", step);

            // scalar
            {
                let value: f32 = (step as f32 * PI / 8.0).sin();
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
}

#[cfg(feature = "full")]
#[async_std::main]
async fn main() -> Fallible<()> {
    async_example::_main().await
}

#[cfg(not(feature = "full",))]
fn main() {
    panic!(r#"please enable the "full" feature to run this example"#);
}
