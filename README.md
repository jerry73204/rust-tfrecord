# tfrecord-rust

The crate provides the functionality to serialize and deserialize TFRecord data format from TensorFlow.

## Features

- Provide both high level `Example` type as well as low level `Vec<u8>` bytes {,de}serialization.
- Support **async/await** syntax. It's easy to work with [futures-rs](https://github.com/rust-lang/futures-rs).
- Interoperability with [serde](https://crates.io/crates/serde), [image](https://crates.io/crates/image), [ndarray](https://crates.io/crates/ndarray) and [tch](https://crates.io/crates/tch).
- TensorBoard support.

## Usage

### Use this crate in your project

Append this line to your `Cargo.toml`.

```
tfrecord = "0.3"
```

### Notice on TensorFlow updates

The crate compiles the pre-generated ProtocolBuffer code from TensorFlow. In case of TensorFlow updates or custom patches, please run the code generation manually, see [Generate ProtocolBuffer code from TensorFlow](#generate-protocolbuffer-code-from-tensorflow) section for details.

### Available Cargo features

**Module features**

- `full`: Enable all features.
- `async_`: Enable async/await feature.
- `dataset`: Enable the dataset API that can load records from multiple TFRecord files.
- `summary`: Enable the summary and event types and writters, mainly for TensorBoard.

**Third-party crate support features**

- `with-serde`: Enable support with [serde](https://crates.io/crates/serde) crate.
- `with-image`: Enable support with [image](https://crates.io/crates/image) crate.
- `with-ndarray`: Enable support with [ndarray](https://crates.io/crates/ndarray) crate.
- `with-tch`: Enable support with [tch](https://crates.io/crates/tch) crate.


## Documentation

See [docs.rs](https://docs.rs/tfrecord/) for the API.

## Example

### File reading example

This is a snipplet copied from [examples/tfrecord\_info.rs](examples/tfrecord_info.rs).

```rust
use tfrecord::{Error, ExampleReader, Feature, RecordReaderInit};

fn main() -> Result<(), Error> {
    // use init pattern to construct the tfrecord reader
    let reader: ExampleReader<_> = RecordReaderInit::default().open(&*INPUT_TFRECORD_PATH)?;

    // print header
    println!("example_no\tfeature_no\tname\ttype\tsize");

    // enumerate examples
    for (example_index, result) in reader.enumerate() {
        let example = result?;

        // enumerate features in an example
        for (feature_index, (name, feature)) in example.into_iter().enumerate() {
            print!("{}\t{}\t{}\t", example_index, feature_index, name);

            match feature {
                Feature::BytesList(list) => {
                    println!("bytes\t{}", list.len());
                }
                Feature::FloatList(list) => {
                    println!("float\t{}", list.len());
                }
                Feature::Int64List(list) => {
                    println!("int64\t{}", list.len());
                }
                Feature::None => {
                    println!("none");
                }
            }
        }
    }

    Ok(())
}
```

### Work with async/await syntax

The snipplet from [examples/tfrecord\_info\_async.rs](examples/tfrecord_info_async.rs) demonstrates the integration with [async-std](https://github.com/async-rs/async-std).

```rust
use futures::stream::TryStreamExt;
use std::{fs::File, io::BufWriter, path::PathBuf};
use tfrecord::{Error, Feature, RecordStreamInit};

pub async fn _main() -> Result<(), Error> {
    // use init pattern to construct the tfrecord stream
    let stream = RecordStreamInit::default()
        .examples_open(&*INPUT_TFRECORD_PATH)
        .await?;

    // print header
    println!("example_no\tfeature_no\tname\ttype\tsize");

    // enumerate examples
    stream
        .try_fold(0, |example_index, example| {
            async move {
                // enumerate features in an example
                for (feature_index, (name, feature)) in example.into_iter().enumerate() {
                    print!("{}\t{}\t{}\t", example_index, feature_index, name);

                    match feature {
                        Feature::BytesList(list) => {
                            println!("bytes\t{}", list.len());
                        }
                        Feature::FloatList(list) => {
                            println!("float\t{}", list.len());
                        }
                        Feature::Int64List(list) => {
                            println!("int64\t{}", list.len());
                        }
                        Feature::None => {
                            println!("none");
                        }
                    }
                }

                Ok(example_index + 1)
            }
        })
        .await?;

    Ok(())
}
```

### Work with TensorBoard

This is a simplified example of [examples/tensorboard.rs](examples/tensorboard.rs) that sends summary data to `log_dir` directory. After running the example, launch `tensorboard --logdir log_dir` to watch the outcome in TensorBoard.

```rust
use super::*;
use rand::seq::SliceRandom;
use rand_distr::{Distribution, Normal};
use std::{f32::consts::PI, thread, time::Duration};
use tfrecord::{EventInit, EventWriterInit};

pub fn _main() -> Fallible<()> {
    // show log dir
    let prefix = "log_dir/my_prefix";

    // download image files
    println!("downloading images...");
    let images = IMAGE_URLS
        .iter()
        .cloned()
        .map(|url| {
            let bytes = reqwest::blocking::get(url)?.bytes()?;
            let image = image::load_from_memory(bytes.as_ref())?;
            Ok(image)
        })
        .collect::<Fallible<Vec<_>>>()?;

    // init writer
    let mut writer = EventWriterInit::from_prefix(prefix, None)?;
    let mut rng = rand::thread_rng();

    // loop
    for step in 0..30 {
        println!("step: {}", step);

        // scalar
        {
            let value: f32 = (step as f32 * PI / 8.0).sin();
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

```

### More examples

You can visit the [examples](examples) and [tests](tests) directories to see more verbose examples.

## Generate ProtocolBuffer code from TensorFlow

The crate relies on ProtocolBuffer documents from TensorFlow. The crate ships pre-generated code from ProtocolBuffer documents by default. Most users don't need to bother with the code generation. The step is needed only in case of TensorFlow updates or your custom patch.

The build script accepts several ways to access the TensorFlow source code, controlled by the `TFRECORD_BUILD_METHOD` environment variable. The generated code will be placed under `prebuild_src` directory. See the examples below to understand the usage.

- Build from a source tarball

```sh
export TFRECORD_BUILD_METHOD="src_file:///home/myname/tensorflow-2.2.0.tar.gz"
cargo build --release --features serde,generate_protobuf_src  # with serde
cargo build --release --features generate_protobuf_src        # without serde
```

- Build from a source directory

```sh
export TFRECORD_BUILD_METHOD="src_dir:///home/myname/tensorflow-2.2.0"
cargo build --release --features serde,generate_protobuf_src  # with serde
cargo build --release --features generate_protobuf_src        # without serde
```

- Build from a URL

```sh
export TFRECORD_BUILD_METHOD="url://https://github.com/tensorflow/tensorflow/archive/v2.2.0.tar.gz"
cargo build --release --features serde,generate_protobuf_src  # with serde
cargo build --release --features generate_protobuf_src        # without serde
```

- Build from installed TensorFlow on system. The build script will search `${install_prefix}/include/tensorflow` directory for protobuf documents.

```sh
export TFRECORD_BUILD_METHOD="install_prefix:///usr"
cargo build --release --features serde,generate_protobuf_src  # with serde
cargo build --release --features generate_protobuf_src        # without serde
```

## License

MIT license. See [LICENSE](LICENSE) file for full license.
