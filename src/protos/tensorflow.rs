/// Containers to hold repeated fundamental values.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct BytesList {
    #[prost(bytes, repeated, tag="1")]
    pub value: ::std::vec::Vec<std::vec::Vec<u8>>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct FloatList {
    #[prost(float, repeated, tag="1")]
    pub value: ::std::vec::Vec<f32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Int64List {
    #[prost(int64, repeated, tag="1")]
    pub value: ::std::vec::Vec<i64>,
}
/// Containers for non-sequential data.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Feature {
    /// Each feature can be exactly one kind.
    #[prost(oneof="feature::Kind", tags="1, 2, 3")]
    pub kind: ::std::option::Option<feature::Kind>,
}
pub mod feature {
    /// Each feature can be exactly one kind.
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub enum Kind {
        #[prost(message, tag="1")]
        BytesList(super::BytesList),
        #[prost(message, tag="2")]
        FloatList(super::FloatList),
        #[prost(message, tag="3")]
        Int64List(super::Int64List),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Features {
    /// Map from feature name to feature.
    #[prost(map="string, message", tag="1")]
    pub feature: ::std::collections::HashMap<std::string::String, Feature>,
}
/// Containers for sequential data.
///
/// A FeatureList contains lists of Features.  These may hold zero or more
/// Feature values.
///
/// FeatureLists are organized into categories by name.  The FeatureLists message
/// contains the mapping from name to FeatureList.
///
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct FeatureList {
    #[prost(message, repeated, tag="1")]
    pub feature: ::std::vec::Vec<Feature>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct FeatureLists {
    /// Map from feature name to feature list.
    #[prost(map="string, message", tag="1")]
    pub feature_list: ::std::collections::HashMap<std::string::String, FeatureList>,
}
// An Example is a mostly-normalized data format for storing data for
// training and inference.  It contains a key-value store (features); where
// each key (string) maps to a Feature message (which is oneof packed BytesList,
// FloatList, or Int64List).  This flexible and compact format allows the
// storage of large amounts of typed data, but requires that the data shape
// and use be determined by the configuration files and parsers that are used to
// read and write this format.  That is, the Example is mostly *not* a
// self-describing format.  In TensorFlow, Examples are read in row-major
// format, so any configuration that describes data with rank-2 or above
// should keep this in mind.  For example, to store an M x N matrix of Bytes,
// the BytesList must contain M*N bytes, with M rows of N contiguous values
// each.  That is, the BytesList value must store the matrix as:
//     .... row 0 .... .... row 1 .... // ...........  // ... row M-1 ....
//
// An Example for a movie recommendation application:
//   features {
//     feature {
//       key: "age"
//       value { float_list {
//         value: 29.0
//       }}
//     }
//     feature {
//       key: "movie"
//       value { bytes_list {
//         value: "The Shawshank Redemption"
//         value: "Fight Club"
//       }}
//     }
//     feature {
//       key: "movie_ratings"
//       value { float_list {
//         value: 9.0
//         value: 9.7
//       }}
//     }
//     feature {
//       key: "suggestion"
//       value { bytes_list {
//         value: "Inception"
//       }}
//     }
//     # Note that this feature exists to be used as a label in training.
//     # E.g., if training a logistic regression model to predict purchase
//     # probability in our learning tool we would set the label feature to
//     # "suggestion_purchased".
//     feature {
//       key: "suggestion_purchased"
//       value { float_list {
//         value: 1.0
//       }}
//     }
//     # Similar to "suggestion_purchased" above this feature exists to be used
//     # as a label in training.
//     # E.g., if training a linear regression model to predict purchase
//     # price in our learning tool we would set the label feature to
//     # "purchase_price".
//     feature {
//       key: "purchase_price"
//       value { float_list {
//         value: 9.99
//       }}
//     }
//  }
//
// A conformant Example data set obeys the following conventions:
//   - If a Feature K exists in one example with data type T, it must be of
//       type T in all other examples when present. It may be omitted.
//   - The number of instances of Feature K list data may vary across examples,
//       depending on the requirements of the model.
//   - If a Feature K doesn't exist in an example, a K-specific default will be
//       used, if configured.
//   - If a Feature K exists in an example but contains no items, the intent
//       is considered to be an empty tensor and no default will be used.

#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Example {
    #[prost(message, optional, tag="1")]
    pub features: ::std::option::Option<Features>,
}
// A SequenceExample is an Example representing one or more sequences, and
// some context.  The context contains features which apply to the entire
// example. The feature_lists contain a key, value map where each key is
// associated with a repeated set of Features (a FeatureList).
// A FeatureList thus represents the values of a feature identified by its key
// over time / frames.
//
// Below is a SequenceExample for a movie recommendation application recording a
// sequence of ratings by a user. The time-independent features ("locale",
// "age", "favorites") describing the user are part of the context. The sequence
// of movies the user rated are part of the feature_lists. For each movie in the
// sequence we have information on its name and actors and the user's rating.
// This information is recorded in three separate feature_list(s).
// In the example below there are only two movies. All three feature_list(s),
// namely "movie_ratings", "movie_names", and "actors" have a feature value for
// both movies. Note, that "actors" is itself a bytes_list with multiple
// strings per movie.
//
// context: {
//   feature: {
//     key  : "locale"
//     value: {
//       bytes_list: {
//         value: [ "pt_BR" ]
//       }
//     }
//   }
//   feature: {
//     key  : "age"
//     value: {
//       float_list: {
//         value: [ 19.0 ]
//       }
//     }
//   }
//   feature: {
//     key  : "favorites"
//     value: {
//       bytes_list: {
//         value: [ "Majesty Rose", "Savannah Outen", "One Direction" ]
//       }
//     }
//   }
// }
// feature_lists: {
//   feature_list: {
//     key  : "movie_ratings"
//     value: {
//       feature: {
//         float_list: {
//           value: [ 4.5 ]
//         }
//       }
//       feature: {
//         float_list: {
//           value: [ 5.0 ]
//         }
//       }
//     }
//   }
//   feature_list: {
//     key  : "movie_names"
//     value: {
//       feature: {
//         bytes_list: {
//           value: [ "The Shawshank Redemption" ]
//         }
//       }
//       feature: {
//         bytes_list: {
//           value: [ "Fight Club" ]
//         }
//       }
//     }
//   }
//   feature_list: {
//     key  : "actors"
//     value: {
//       feature: {
//         bytes_list: {
//           value: [ "Tim Robbins", "Morgan Freeman" ]
//         }
//       }
//       feature: {
//         bytes_list: {
//           value: [ "Brad Pitt", "Edward Norton", "Helena Bonham Carter" ]
//         }
//       }
//     }
//   }
// }
//
// A conformant SequenceExample data set obeys the following conventions:
//
// Context:
//   - All conformant context features K must obey the same conventions as
//     a conformant Example's features (see above).
// Feature lists:
//   - A FeatureList L may be missing in an example; it is up to the
//     parser configuration to determine if this is allowed or considered
//     an empty list (zero length).
//   - If a FeatureList L exists, it may be empty (zero length).
//   - If a FeatureList L is non-empty, all features within the FeatureList
//     must have the same data type T. Even across SequenceExamples, the type T
//     of the FeatureList identified by the same key must be the same. An entry
//     without any values may serve as an empty feature.
//   - If a FeatureList L is non-empty, it is up to the parser configuration
//     to determine if all features within the FeatureList must
//     have the same size.  The same holds for this FeatureList across multiple
//     examples.
//   - For sequence modeling, e.g.:
//        http://colah.github.io/posts/2015-08-Understanding-LSTMs/
//        https://github.com/tensorflow/nmt
//     the feature lists represent a sequence of frames.
//     In this scenario, all FeatureLists in a SequenceExample have the same
//     number of Feature messages, so that the ith element in each FeatureList
//     is part of the ith frame (or time step).
// Examples of conformant and non-conformant examples' FeatureLists:
//
// Conformant FeatureLists:
//    feature_lists: { feature_list: {
//      key: "movie_ratings"
//      value: { feature: { float_list: { value: [ 4.5 ] } }
//               feature: { float_list: { value: [ 5.0 ] } } }
//    } }
//
// Non-conformant FeatureLists (mismatched types):
//    feature_lists: { feature_list: {
//      key: "movie_ratings"
//      value: { feature: { float_list: { value: [ 4.5 ] } }
//               feature: { int64_list: { value: [ 5 ] } } }
//    } }
//
// Conditionally conformant FeatureLists, the parser configuration determines
// if the feature sizes must match:
//    feature_lists: { feature_list: {
//      key: "movie_ratings"
//      value: { feature: { float_list: { value: [ 4.5 ] } }
//               feature: { float_list: { value: [ 5.0, 6.0 ] } } }
//    } }
//
// Conformant pair of SequenceExample
//    feature_lists: { feature_list: {
//      key: "movie_ratings"
//      value: { feature: { float_list: { value: [ 4.5 ] } }
//               feature: { float_list: { value: [ 5.0 ] } } }
//    } }
// and:
//    feature_lists: { feature_list: {
//      key: "movie_ratings"
//      value: { feature: { float_list: { value: [ 4.5 ] } }
//               feature: { float_list: { value: [ 5.0 ] } }
//               feature: { float_list: { value: [ 2.0 ] } } }
//    } }
//
// Conformant pair of SequenceExample
//    feature_lists: { feature_list: {
//      key: "movie_ratings"
//      value: { feature: { float_list: { value: [ 4.5 ] } }
//               feature: { float_list: { value: [ 5.0 ] } } }
//    } }
// and:
//    feature_lists: { feature_list: {
//      key: "movie_ratings"
//      value: { }
//    } }
//
// Conditionally conformant pair of SequenceExample, the parser configuration
// determines if the second feature_lists is consistent (zero-length) or
// invalid (missing "movie_ratings"):
//    feature_lists: { feature_list: {
//      key: "movie_ratings"
//      value: { feature: { float_list: { value: [ 4.5 ] } }
//               feature: { float_list: { value: [ 5.0 ] } } }
//    } }
// and:
//    feature_lists: { }
//
// Non-conformant pair of SequenceExample (mismatched types)
//    feature_lists: { feature_list: {
//      key: "movie_ratings"
//      value: { feature: { float_list: { value: [ 4.5 ] } }
//               feature: { float_list: { value: [ 5.0 ] } } }
//    } }
// and:
//    feature_lists: { feature_list: {
//      key: "movie_ratings"
//      value: { feature: { int64_list: { value: [ 4 ] } }
//               feature: { int64_list: { value: [ 5 ] } }
//               feature: { int64_list: { value: [ 2 ] } } }
//    } }
//
// Conditionally conformant pair of SequenceExample; the parser configuration
// determines if the feature sizes must match:
//    feature_lists: { feature_list: {
//      key: "movie_ratings"
//      value: { feature: { float_list: { value: [ 4.5 ] } }
//               feature: { float_list: { value: [ 5.0 ] } } }
//    } }
// and:
//    feature_lists: { feature_list: {
//      key: "movie_ratings"
//      value: { feature: { float_list: { value: [ 4.0 ] } }
//               feature: { float_list: { value: [ 5.0, 3.0 ] } }
//    } }

#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SequenceExample {
    #[prost(message, optional, tag="1")]
    pub context: ::std::option::Option<Features>,
    #[prost(message, optional, tag="2")]
    pub feature_lists: ::std::option::Option<FeatureLists>,
}
/// Dimensions of a tensor.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct TensorShapeProto {
    /// Dimensions of the tensor, such as {"input", 30}, {"output", 40}
    /// for a 30 x 40 2D tensor.  If an entry has size -1, this
    /// corresponds to a dimension of unknown size. The names are
    /// optional.
    ///
    /// The order of entries in "dim" matters: It indicates the layout of the
    /// values in the tensor in-memory representation.
    ///
    /// The first entry in "dim" is the outermost dimension used to layout the
    /// values, the last entry is the innermost dimension.  This matches the
    /// in-memory layout of RowMajor Eigen tensors.
    ///
    /// If "dim.size()" > 0, "unknown_rank" must be false.
    #[prost(message, repeated, tag="2")]
    pub dim: ::std::vec::Vec<tensor_shape_proto::Dim>,
    /// If true, the number of dimensions in the shape is unknown.
    ///
    /// If true, "dim.size()" must be 0.
    #[prost(bool, tag="3")]
    pub unknown_rank: bool,
}
pub mod tensor_shape_proto {
    /// One dimension of the tensor.
    #[derive(Clone, PartialEq, ::prost::Message)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Dim {
        /// Size of the tensor in that dimension.
        /// This value must be >= -1, but values of -1 are reserved for "unknown"
        /// shapes (values of -1 mean "unknown" dimension).  Certain wrappers
        /// that work with TensorShapeProto may fail at runtime when deserializing
        /// a TensorShapeProto containing a dim value of -1.
        #[prost(int64, tag="1")]
        pub size: i64,
        /// Optional name of the tensor dimension.
        #[prost(string, tag="2")]
        pub name: std::string::String,
    }
}
/// (== suppress_warning documentation-presence ==)
/// LINT.IfChange
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum DataType {
    /// Not a legal value for DataType.  Used to indicate a DataType field
    /// has not been set.
    DtInvalid = 0,
    /// Data types that all computation devices are expected to be
    /// capable to support.
    DtFloat = 1,
    DtDouble = 2,
    DtInt32 = 3,
    DtUint8 = 4,
    DtInt16 = 5,
    DtInt8 = 6,
    DtString = 7,
    /// Single-precision complex
    DtComplex64 = 8,
    DtInt64 = 9,
    DtBool = 10,
    /// Quantized int8
    DtQint8 = 11,
    /// Quantized uint8
    DtQuint8 = 12,
    /// Quantized int32
    DtQint32 = 13,
    /// Float32 truncated to 16 bits.  Only for cast ops.
    DtBfloat16 = 14,
    /// Quantized int16
    DtQint16 = 15,
    /// Quantized uint16
    DtQuint16 = 16,
    DtUint16 = 17,
    /// Double-precision complex
    DtComplex128 = 18,
    DtHalf = 19,
    DtResource = 20,
    /// Arbitrary C++ data types
    DtVariant = 21,
    DtUint32 = 22,
    DtUint64 = 23,
    /// Do not use!  These are only for parameters.  Every enum above
    /// should have a corresponding value below (verified by types_test).
    DtFloatRef = 101,
    DtDoubleRef = 102,
    DtInt32Ref = 103,
    DtUint8Ref = 104,
    DtInt16Ref = 105,
    DtInt8Ref = 106,
    DtStringRef = 107,
    DtComplex64Ref = 108,
    DtInt64Ref = 109,
    DtBoolRef = 110,
    DtQint8Ref = 111,
    DtQuint8Ref = 112,
    DtQint32Ref = 113,
    DtBfloat16Ref = 114,
    DtQint16Ref = 115,
    DtQuint16Ref = 116,
    DtUint16Ref = 117,
    DtComplex128Ref = 118,
    DtHalfRef = 119,
    DtResourceRef = 120,
    DtVariantRef = 121,
    DtUint32Ref = 122,
    DtUint64Ref = 123,
}
/// Protocol buffer representing a handle to a tensorflow resource. Handles are
/// not valid across executions, but can be serialized back and forth from within
/// a single run.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ResourceHandleProto {
    /// Unique name for the device containing the resource.
    #[prost(string, tag="1")]
    pub device: std::string::String,
    /// Container in which this resource is placed.
    #[prost(string, tag="2")]
    pub container: std::string::String,
    /// Unique name of this resource.
    #[prost(string, tag="3")]
    pub name: std::string::String,
    /// Hash code for the type of the resource. Is only valid in the same device
    /// and in the same execution.
    #[prost(uint64, tag="4")]
    pub hash_code: u64,
    /// For debug-only, the name of the type pointed to by this handle, if
    /// available.
    #[prost(string, tag="5")]
    pub maybe_type_name: std::string::String,
    /// Data types and shapes for the underlying resource.
    #[prost(message, repeated, tag="6")]
    pub dtypes_and_shapes: ::std::vec::Vec<resource_handle_proto::DtypeAndShape>,
    /// A set of devices containing the resource. If empty, the resource only
    /// exists on `device`.
    #[prost(string, repeated, tag="7")]
    pub allowed_devices: ::std::vec::Vec<std::string::String>,
}
pub mod resource_handle_proto {
    /// Protocol buffer representing a pair of (data type, tensor shape).
    #[derive(Clone, PartialEq, ::prost::Message)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct DtypeAndShape {
        #[prost(enumeration="super::DataType", tag="1")]
        pub dtype: i32,
        #[prost(message, optional, tag="2")]
        pub shape: ::std::option::Option<super::TensorShapeProto>,
    }
}
/// Protocol buffer representing a tensor.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct TensorProto {
    #[prost(enumeration="DataType", tag="1")]
    pub dtype: i32,
    /// Shape of the tensor.  TODO(touts): sort out the 0-rank issues.
    #[prost(message, optional, tag="2")]
    pub tensor_shape: ::std::option::Option<TensorShapeProto>,
    // Only one of the representations below is set, one of "tensor_contents" and
    // the "xxx_val" attributes.  We are not using oneof because as oneofs cannot
    // contain repeated fields it would require another extra set of messages.

    /// Version number.
    ///
    /// In version 0, if the "repeated xxx" representations contain only one
    /// element, that element is repeated to fill the shape.  This makes it easy
    /// to represent a constant Tensor with a single value.
    #[prost(int32, tag="3")]
    pub version_number: i32,
    /// Serialized raw tensor content from either Tensor::AsProtoTensorContent or
    /// memcpy in tensorflow::grpc::EncodeTensorToByteBuffer. This representation
    /// can be used for all tensor types. The purpose of this representation is to
    /// reduce serialization overhead during RPC call by avoiding serialization of
    /// many repeated small items.
    #[prost(bytes, tag="4")]
    pub tensor_content: std::vec::Vec<u8>,
    // Type specific representations that make it easy to create tensor protos in
    // all languages.  Only the representation corresponding to "dtype" can
    // be set.  The values hold the flattened representation of the tensor in
    // row major order.

    /// DT_HALF, DT_BFLOAT16. Note that since protobuf has no int16 type, we'll
    /// have some pointless zero padding for each value here.
    #[prost(int32, repeated, tag="13")]
    pub half_val: ::std::vec::Vec<i32>,
    /// DT_FLOAT.
    #[prost(float, repeated, tag="5")]
    pub float_val: ::std::vec::Vec<f32>,
    /// DT_DOUBLE.
    #[prost(double, repeated, tag="6")]
    pub double_val: ::std::vec::Vec<f64>,
    /// DT_INT32, DT_INT16, DT_INT8, DT_UINT8.
    #[prost(int32, repeated, tag="7")]
    pub int_val: ::std::vec::Vec<i32>,
    /// DT_STRING
    #[prost(bytes, repeated, tag="8")]
    pub string_val: ::std::vec::Vec<std::vec::Vec<u8>>,
    /// DT_COMPLEX64. scomplex_val(2*i) and scomplex_val(2*i+1) are real
    /// and imaginary parts of i-th single precision complex.
    #[prost(float, repeated, tag="9")]
    pub scomplex_val: ::std::vec::Vec<f32>,
    /// DT_INT64
    #[prost(int64, repeated, tag="10")]
    pub int64_val: ::std::vec::Vec<i64>,
    /// DT_BOOL
    #[prost(bool, repeated, tag="11")]
    pub bool_val: ::std::vec::Vec<bool>,
    /// DT_COMPLEX128. dcomplex_val(2*i) and dcomplex_val(2*i+1) are real
    /// and imaginary parts of i-th double precision complex.
    #[prost(double, repeated, tag="12")]
    pub dcomplex_val: ::std::vec::Vec<f64>,
    /// DT_RESOURCE
    #[prost(message, repeated, tag="14")]
    pub resource_handle_val: ::std::vec::Vec<ResourceHandleProto>,
    /// DT_VARIANT
    #[prost(message, repeated, tag="15")]
    pub variant_val: ::std::vec::Vec<VariantTensorDataProto>,
    /// DT_UINT32
    #[prost(uint32, repeated, tag="16")]
    pub uint32_val: ::std::vec::Vec<u32>,
    /// DT_UINT64
    #[prost(uint64, repeated, tag="17")]
    pub uint64_val: ::std::vec::Vec<u64>,
}
/// Protocol buffer representing the serialization format of DT_VARIANT tensors.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct VariantTensorDataProto {
    /// Name of the type of objects being serialized.
    #[prost(string, tag="1")]
    pub type_name: std::string::String,
    /// Portions of the object that are not Tensors.
    #[prost(bytes, tag="2")]
    pub metadata: std::vec::Vec<u8>,
    /// Tensors contained within objects being serialized.
    #[prost(message, repeated, tag="3")]
    pub tensors: ::std::vec::Vec<TensorProto>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct VarLenFeatureProto {
    #[prost(enumeration="DataType", tag="1")]
    pub dtype: i32,
    #[prost(string, tag="2")]
    pub values_output_tensor_name: std::string::String,
    #[prost(string, tag="3")]
    pub indices_output_tensor_name: std::string::String,
    #[prost(string, tag="4")]
    pub shapes_output_tensor_name: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct FixedLenFeatureProto {
    #[prost(enumeration="DataType", tag="1")]
    pub dtype: i32,
    #[prost(message, optional, tag="2")]
    pub shape: ::std::option::Option<TensorShapeProto>,
    #[prost(message, optional, tag="3")]
    pub default_value: ::std::option::Option<TensorProto>,
    #[prost(string, tag="4")]
    pub values_output_tensor_name: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct FeatureConfiguration {
    #[prost(oneof="feature_configuration::Config", tags="1, 2")]
    pub config: ::std::option::Option<feature_configuration::Config>,
}
pub mod feature_configuration {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub enum Config {
        #[prost(message, tag="1")]
        FixedLenFeature(super::FixedLenFeatureProto),
        #[prost(message, tag="2")]
        VarLenFeature(super::VarLenFeatureProto),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ExampleParserConfiguration {
    #[prost(map="string, message", tag="1")]
    pub feature_map: ::std::collections::HashMap<std::string::String, FeatureConfiguration>,
}
