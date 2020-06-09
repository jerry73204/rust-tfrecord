mod common;

use common::*;
use rand::Rng;

#[cfg(feature = "summary")]
#[async_std::test]
async fn sync_event_writer() -> Fallible<()> {
    let mut rng = rand::thread_rng();

    let prefix = DATA_DIR
        .join("log_dir")
        .join("sync-test")
        .into_os_string()
        .into_string()
        .unwrap();
    let mut writer = EventWriterInit::from_prefix(prefix, None)?;

    for step in 0..30 {
        let value: f32 = rng.gen_range(-10.0, 10.0);
        println!("step: {}\tvalue: {}", step, value);

        writer.write_scalar("x", EventInit::with_step(step), value)?;
        thread::sleep(Duration::from_millis(1000));
    }

    Ok(())
}
