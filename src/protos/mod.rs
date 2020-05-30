//! ProtocolBuffer types compiled from TensorFlow.
//!
//! The types are provided by ProtocolBuffer documents from TensorFlow repository.
//! They are used internally for {,de}serialization.

include!(concat!(env!("OUT_DIR"), "/tensorflow.rs"));
