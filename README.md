# tfrecord

\[ [API doc](https://docs.rs/tfrecord/) | [crates.io](https://crates.io/crates/tfrecord/) \]

The crate provides readers/writers for TensorFlow TFRecord data format. It has the following features.

- Provide both high level `Example` type as well as low level `Vec<u8>` bytes de/serialization.
- Support **async/await** syntax. It's easy to work with [futures-rs](https://github.com/rust-lang/futures-rs).
- Interoperability with [serde](https://crates.io/crates/serde), [image](https://crates.io/crates/image), [ndarray](https://crates.io/crates/ndarray) and [tch](https://crates.io/crates/tch).
- Support TensorBoard! ([exampe code](examples/tensorboard.rs))

## License

MIT license. See [LICENSE](LICENSE) file for full license.
