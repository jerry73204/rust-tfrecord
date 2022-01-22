//! ProtocolBuffer types compiled from TensorFlow.
//!
//! The types are provided by ProtocolBuffer documents from TensorFlow repository.
//! They are used internally for {,de}serialization.

mod ext;

#[cfg(feature = "with-serde")]
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/prebuild_src/tensorflow_with_serde.rs"
));

#[cfg(not(feature = "with-serde"))]
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/prebuild_src/tensorflow_without_serde.rs"
));
