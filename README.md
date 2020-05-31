# tfrecord-rust

The crate provides the functionality to serialize and deserialize TFRecord data format from TensorFlow.

## Features

- Provide both high level `EasyExample` type as well as low level `Vec<u8>` bytes {,de}serialization.
- Support **async/await** syntax. It's easy to work with [futures-rs](https://github.com/rust-lang/futures-rs).
- Interoperability with [serde](https://github.com/serde-rs/serde).

## Usage

Append this line to your `Cargo.toml`.

```
tfrecord = "0.1.1"
```

The crate provides several cargo features that you can conditionally compile modules.

- `serde`: Enable interoperability with [serde](https://github.com/serde-rs/serde) to serialize and deserialize example types.
- `async_`: Enable async/await feature.
- `dataset`: Enable the dataset API that can load records from multiple TFRecord files.
- `full`: Enable all features above.

By default, the crate compiles the pre-built ProtocolBuffer code in the repository. If you would like to re-run the code generation, see [Generate ProtocolBuffer code from TensorFlow](#generate-protocolbuffer-code-from-tensorflow) section.

## Documentation

See [docs.rs](https://docs.rs/tfrecord/) for the API.

## Example

### File reading example

This is a snipplet copied from [examples/tfrecord\_info.rs](examples/tfrecord_info.rs).

```rust
use tfrecord::{EasyExampleReader, EasyFeature, Error, RecordReaderInit};

fn main() -> Result<(), Error> {
    // use init pattern to construct the tfrecord reader
    let reader: EasyExampleReader<_> = RecordReaderInit {
        check_integrity: true,
    }
    .open(&*INPUT_TFRECORD_PATH)?;

    // print header
    println!("example_no\tfeature_no\tname\ttype\tsize");

    // enumerate examples
    for (example_index, result) in reader.enumerate() {
        let example = result?;

        // enumerate features in an example
        for (feature_index, (name, feature)) in example.into_iter().enumerate() {
            print!("{}\t{}\t{}\t", example_index, feature_index, name);

            match feature {
                EasyFeature::BytesList(list) => {
                    println!("bytes\t{}", list.len());
                }
                EasyFeature::FloatList(list) => {
                    println!("float\t{}", list.len());
                }
                EasyFeature::Int64List(list) => {
                    println!("int64\t{}", list.len());
                }
                EasyFeature::None => {
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
use tfrecord::{EasyFeature, Error, RecordStreamInit};

#[async_std::main]
async fn main() -> Result<(), Error> {
    // use init pattern to construct the tfrecord stream
    let stream = RecordStreamInit {
        check_integrity: true,
    }
    .easy_examples_open(&*INPUT_TFRECORD_PATH)
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
                        EasyFeature::BytesList(list) => {
                            println!("bytes\t{}", list.len());
                        }
                        EasyFeature::FloatList(list) => {
                            println!("float\t{}", list.len());
                        }
                        EasyFeature::Int64List(list) => {
                            println!("int64\t{}", list.len());
                        }
                        EasyFeature::None => {
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

### More examples

Also, we suggest visiting the [test code](tests) for more detailed usage.


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
