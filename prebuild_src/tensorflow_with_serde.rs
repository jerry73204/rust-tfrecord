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
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct AllocationDescription {
    /// Total number of bytes requested
    #[prost(int64, tag="1")]
    pub requested_bytes: i64,
    /// Total number of bytes allocated if known
    #[prost(int64, tag="2")]
    pub allocated_bytes: i64,
    /// Name of the allocator used
    #[prost(string, tag="3")]
    pub allocator_name: std::string::String,
    /// Identifier of the allocated buffer if known
    #[prost(int64, tag="4")]
    pub allocation_id: i64,
    /// Set if this tensor only has one remaining reference
    #[prost(bool, tag="5")]
    pub has_single_reference: bool,
    /// Address of the allocation.
    #[prost(uint64, tag="6")]
    pub ptr: u64,
}
/// Protocol buffer representing the value for an attr used to configure an Op.
/// Comment indicates the corresponding attr type.  Only the field matching the
/// attr type may be filled.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct AttrValue {
    #[prost(oneof="attr_value::Value", tags="2, 3, 4, 5, 6, 7, 8, 1, 10, 9")]
    pub value: ::std::option::Option<attr_value::Value>,
}
pub mod attr_value {
    /// LINT.IfChange
    #[derive(Clone, PartialEq, ::prost::Message)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct ListValue {
        /// "list(string)"
        #[prost(bytes, repeated, tag="2")]
        pub s: ::std::vec::Vec<std::vec::Vec<u8>>,
        /// "list(int)"
        #[prost(int64, repeated, tag="3")]
        pub i: ::std::vec::Vec<i64>,
        /// "list(float)"
        #[prost(float, repeated, tag="4")]
        pub f: ::std::vec::Vec<f32>,
        /// "list(bool)"
        #[prost(bool, repeated, tag="5")]
        pub b: ::std::vec::Vec<bool>,
        /// "list(type)"
        #[prost(enumeration="super::DataType", repeated, tag="6")]
        pub r#type: ::std::vec::Vec<i32>,
        /// "list(shape)"
        #[prost(message, repeated, tag="7")]
        pub shape: ::std::vec::Vec<super::TensorShapeProto>,
        /// "list(tensor)"
        #[prost(message, repeated, tag="8")]
        pub tensor: ::std::vec::Vec<super::TensorProto>,
        /// "list(attr)"
        #[prost(message, repeated, tag="9")]
        pub func: ::std::vec::Vec<super::NameAttrList>,
    }
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub enum Value {
        /// "string"
        #[prost(bytes, tag="2")]
        S(std::vec::Vec<u8>),
        /// "int"
        #[prost(int64, tag="3")]
        I(i64),
        /// "float"
        #[prost(float, tag="4")]
        F(f32),
        /// "bool"
        #[prost(bool, tag="5")]
        B(bool),
        /// "type"
        #[prost(enumeration="super::DataType", tag="6")]
        Type(i32),
        /// "shape"
        #[prost(message, tag="7")]
        Shape(super::TensorShapeProto),
        /// "tensor"
        #[prost(message, tag="8")]
        Tensor(super::TensorProto),
        /// any "list(...)"
        #[prost(message, tag="1")]
        List(ListValue),
        /// "func" represents a function. func.name is a function's name or
        /// a primitive op's name. func.attr.first is the name of an attr
        /// defined for that function. func.attr.second is the value for
        /// that attr in the instantiation.
        #[prost(message, tag="10")]
        Func(super::NameAttrList),
        /// This is a placeholder only used in nodes defined inside a
        /// function.  It indicates the attr value will be supplied when
        /// the function is instantiated.  For example, let us suppose a
        /// node "N" in function "FN". "N" has an attr "A" with value
        /// placeholder = "foo". When FN is instantiated with attr "foo"
        /// set to "bar", the instantiated node N's attr A will have been
        /// given the value "bar".
        #[prost(string, tag="9")]
        Placeholder(std::string::String),
    }
}
/// A list of attr names and their values. The whole list is attached
/// with a string name.  E.g., MatMul[T=float].
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct NameAttrList {
    #[prost(string, tag="1")]
    pub name: std::string::String,
    #[prost(map="string, message", tag="2")]
    pub attr: ::std::collections::HashMap<std::string::String, AttrValue>,
}
/// Used to specify and override the default API & behavior in the
/// generated code for client languages, from what you would get from
/// the OpDef alone. There will be a set of ApiDefs that are common
/// to all client languages, and another set per client language.
/// The per-client-language ApiDefs will inherit values from the
/// common ApiDefs which it can either replace or modify.
///
/// We separate the API definition from the OpDef so we can evolve the
/// API while remaining backwards compatible when interpretting old
/// graphs.  Overrides go in an "api_def.pbtxt" file with a text-format
/// ApiDefs message.
///
/// WARNING: Be *very* careful changing the API for any existing op --
/// you can change the semantics of existing code.  These changes may
/// need to wait until a major release of TensorFlow to avoid breaking
/// our compatibility promises.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ApiDef {
    /// Name of the op (in the OpDef) to specify the API for.
    #[prost(string, tag="1")]
    pub graph_op_name: std::string::String,
    /// If this op is deprecated, set deprecation message to the message
    /// that should be logged when this op is used.
    /// The message should indicate alternative op to use, if any.
    #[prost(string, tag="12")]
    pub deprecation_message: std::string::String,
    /// Major version when the op will be deleted. For e.g. set this
    /// value to 2 if op API should be removed in TensorFlow 2.0 and
    /// deprecated in versions before that.
    #[prost(int32, tag="13")]
    pub deprecation_version: i32,
    #[prost(enumeration="api_def::Visibility", tag="2")]
    pub visibility: i32,
    #[prost(message, repeated, tag="3")]
    pub endpoint: ::std::vec::Vec<api_def::Endpoint>,
    #[prost(message, repeated, tag="4")]
    pub in_arg: ::std::vec::Vec<api_def::Arg>,
    #[prost(message, repeated, tag="5")]
    pub out_arg: ::std::vec::Vec<api_def::Arg>,
    /// List of original in_arg names to specify new argument order.
    /// Length of arg_order should be either empty to keep current order
    /// or match size of in_arg.
    #[prost(string, repeated, tag="11")]
    pub arg_order: ::std::vec::Vec<std::string::String>,
    #[prost(message, repeated, tag="6")]
    pub attr: ::std::vec::Vec<api_def::Attr>,
    /// One-line human-readable description of what the Op does.
    #[prost(string, tag="7")]
    pub summary: std::string::String,
    /// Additional, longer human-readable description of what the Op does.
    #[prost(string, tag="8")]
    pub description: std::string::String,
    /// Modify an existing/inherited description by adding text to the beginning
    /// or end.
    #[prost(string, tag="9")]
    pub description_prefix: std::string::String,
    #[prost(string, tag="10")]
    pub description_suffix: std::string::String,
}
pub mod api_def {
    /// If you specify any endpoint, this will replace all of the
    /// inherited endpoints.  The first endpoint should be the
    /// "canonical" endpoint, and should not be deprecated (unless all
    /// endpoints are deprecated).
    #[derive(Clone, PartialEq, ::prost::Message)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Endpoint {
        /// Name should be either like "CamelCaseName" or
        /// "Package.CamelCaseName". Client-language-specific ApiDefs may
        /// use a snake_case convention instead of CamelCase.
        #[prost(string, tag="1")]
        pub name: std::string::String,
        /// Set if this endpoint is deprecated. If set to true, a message suggesting
        /// to use a non-deprecated endpoint instead will be printed. If all
        /// endpoints are deprecated, set deprecation_message in ApiDef instead.
        #[prost(bool, tag="3")]
        pub deprecated: bool,
        /// Major version when an endpoint will be deleted. For e.g. set this
        /// value to 2 if endpoint should be removed in TensorFlow 2.0 and
        /// deprecated in versions before that.
        #[prost(int32, tag="4")]
        pub deprecation_version: i32,
    }
    #[derive(Clone, PartialEq, ::prost::Message)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Arg {
        #[prost(string, tag="1")]
        pub name: std::string::String,
        /// Change the name used to access this arg in the API from what
        /// is used in the GraphDef.  Note that these names in `backticks`
        /// will also be replaced in the summary & description fields.
        #[prost(string, tag="2")]
        pub rename_to: std::string::String,
        /// Note: this will replace any inherited arg doc. There is no
        /// current way of modifying arg descriptions (other than replacing
        /// them entirely) as can be done with op descriptions.
        #[prost(string, tag="3")]
        pub description: std::string::String,
    }
    /// Description of the graph-construction-time configuration of this
    /// Op.  That is to say, this describes the attr fields that will
    /// be specified in the NodeDef.
    #[derive(Clone, PartialEq, ::prost::Message)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Attr {
        #[prost(string, tag="1")]
        pub name: std::string::String,
        /// Change the name used to access this attr in the API from what
        /// is used in the GraphDef.  Note that these names in `backticks`
        /// will also be replaced in the summary & description fields.
        #[prost(string, tag="2")]
        pub rename_to: std::string::String,
        /// Specify a new default value to use for this attr.  This default
        /// will be used when creating new graphs, as opposed to the
        /// default in the OpDef, which will be used when interpreting old
        /// GraphDefs.
        #[prost(message, optional, tag="3")]
        pub default_value: ::std::option::Option<super::AttrValue>,
        /// Note: this will replace any inherited attr doc, there is no current
        /// way of modifying attr descriptions as can be done with op descriptions.
        #[prost(string, tag="4")]
        pub description: std::string::String,
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub enum Visibility {
        /// Normally this is "VISIBLE" unless you are inheriting a
        /// different value from another ApiDef.
        DefaultVisibility = 0,
        /// Publicly visible in the API.
        Visible = 1,
        /// Do not include this op in the generated API. If visibility is
        /// set to 'SKIP', other fields are ignored for this op.
        Skip = 2,
        /// Hide this op by putting it into an internal namespace (or whatever
        /// is appropriate in the target language).
        Hidden = 3,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ApiDefs {
    #[prost(message, repeated, tag="1")]
    pub op: ::std::vec::Vec<ApiDef>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct CostGraphDef {
    #[prost(message, repeated, tag="1")]
    pub node: ::std::vec::Vec<cost_graph_def::Node>,
    #[prost(message, repeated, tag="2")]
    pub cost: ::std::vec::Vec<cost_graph_def::AggregatedCost>,
}
pub mod cost_graph_def {
    #[derive(Clone, PartialEq, ::prost::Message)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Node {
        /// The name of the node. Names are globally unique.
        #[prost(string, tag="1")]
        pub name: std::string::String,
        /// The device of the node. Can be empty if the node is mapped to the
        /// default partition or partitioning hasn't been run yet.
        #[prost(string, tag="2")]
        pub device: std::string::String,
        /// The id of the node. Node ids are only unique inside a partition.
        #[prost(int32, tag="3")]
        pub id: i32,
        #[prost(message, repeated, tag="4")]
        pub input_info: ::std::vec::Vec<node::InputInfo>,
        #[prost(message, repeated, tag="5")]
        pub output_info: ::std::vec::Vec<node::OutputInfo>,
        /// Temporary memory used by this node.
        #[prost(int64, tag="6")]
        pub temporary_memory_size: i64,
        /// Persistent memory used by this node.
        #[prost(int64, tag="12")]
        pub persistent_memory_size: i64,
        #[prost(int64, tag="10")]
        pub host_temp_memory_size: i64,
        #[prost(int64, tag="11")]
        pub device_temp_memory_size: i64,
        #[prost(int64, tag="16")]
        pub device_persistent_memory_size: i64,
        /// Estimate of the computational cost of this node, in microseconds.
        #[prost(int64, tag="9")]
        pub compute_cost: i64,
        /// Analytical estimate of the computational cost of this node, in
        /// microseconds.
        #[prost(int64, tag="14")]
        pub compute_time: i64,
        /// Analytical estimate of the memory access cost of this node, in
        /// microseconds.
        #[prost(int64, tag="15")]
        pub memory_time: i64,
        /// If true, the output is permanent: it can't be discarded, because this
        /// node is part of the "final output". Nodes may depend on final nodes.
        #[prost(bool, tag="7")]
        pub is_final: bool,
        /// Ids of the control inputs for this node.
        #[prost(int32, repeated, tag="8")]
        pub control_input: ::std::vec::Vec<i32>,
        /// Are the costs inaccurate?
        #[prost(bool, tag="17")]
        pub inaccurate: bool,
    }
    pub mod node {
        /// Inputs of this node. They must be executed before this node can be
        /// executed. An input is a particular output of another node, specified
        /// by the node id and the output index.
        #[derive(Clone, PartialEq, ::prost::Message)]
        #[derive(serde::Serialize, serde::Deserialize)]
        pub struct InputInfo {
            #[prost(int32, tag="1")]
            pub preceding_node: i32,
            #[prost(int32, tag="2")]
            pub preceding_port: i32,
        }
        /// Outputs of this node.
        #[derive(Clone, PartialEq, ::prost::Message)]
        #[derive(serde::Serialize, serde::Deserialize)]
        pub struct OutputInfo {
            #[prost(int64, tag="1")]
            pub size: i64,
            /// If >= 0, the output is an alias of an input. Note that an alias input
            /// may itself be an alias. The algorithm will therefore need to follow
            /// those pointers.
            #[prost(int64, tag="2")]
            pub alias_input_port: i64,
            #[prost(message, optional, tag="3")]
            pub shape: ::std::option::Option<super::super::TensorShapeProto>,
            #[prost(enumeration="super::super::DataType", tag="4")]
            pub dtype: i32,
        }
    }
    /// Total cost of this graph, typically used for balancing decisions.
    #[derive(Clone, PartialEq, ::prost::Message)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct AggregatedCost {
        /// Aggregated cost value.
        #[prost(float, tag="1")]
        pub cost: f32,
        /// Aggregated cost dimension (e.g. 'memory', 'compute', 'network').
        #[prost(string, tag="2")]
        pub dimension: std::string::String,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct InterconnectLink {
    #[prost(int32, tag="1")]
    pub device_id: i32,
    #[prost(string, tag="2")]
    pub r#type: std::string::String,
    #[prost(int32, tag="3")]
    pub strength: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct LocalLinks {
    #[prost(message, repeated, tag="1")]
    pub link: ::std::vec::Vec<InterconnectLink>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct DeviceLocality {
    /// Optional bus locality of device.  Default value of 0 means
    /// no specific locality.  Specific localities are indexed from 1.
    #[prost(int32, tag="1")]
    pub bus_id: i32,
    /// Optional NUMA locality of device.
    #[prost(int32, tag="2")]
    pub numa_node: i32,
    /// Optional local interconnect links to other devices.
    #[prost(message, optional, tag="3")]
    pub links: ::std::option::Option<LocalLinks>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct DeviceAttributes {
    /// Fully specified name of the device within a cluster.
    #[prost(string, tag="1")]
    pub name: std::string::String,
    /// String representation of device_type.
    #[prost(string, tag="2")]
    pub device_type: std::string::String,
    /// Memory capacity of device in bytes.
    #[prost(int64, tag="4")]
    pub memory_limit: i64,
    /// Platform-specific data about device that may be useful
    /// for supporting efficient data transfers.
    #[prost(message, optional, tag="5")]
    pub locality: ::std::option::Option<DeviceLocality>,
    /// A device is assigned a global unique number each time it is
    /// initialized. "incarnation" should never be 0.
    #[prost(fixed64, tag="6")]
    pub incarnation: u64,
    /// String representation of the physical device that this device maps to.
    #[prost(string, tag="7")]
    pub physical_device_desc: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct NodeDef {
    /// The name given to this operator. Used for naming inputs,
    /// logging, visualization, etc.  Unique within a single GraphDef.
    /// Must match the regexp "[A-Za-z0-9.][A-Za-z0-9_>./]*".
    #[prost(string, tag="1")]
    pub name: std::string::String,
    /// The operation name.  There may be custom parameters in attrs.
    /// Op names starting with an underscore are reserved for internal use.
    #[prost(string, tag="2")]
    pub op: std::string::String,
    /// Each input is "node:src_output" with "node" being a string name and
    /// "src_output" indicating which output tensor to use from "node". If
    /// "src_output" is 0 the ":0" suffix can be omitted.  Regular inputs
    /// may optionally be followed by control inputs that have the format
    /// "^node".
    #[prost(string, repeated, tag="3")]
    pub input: ::std::vec::Vec<std::string::String>,
    /// A (possibly partial) specification for the device on which this
    /// node should be placed.
    /// The expected syntax for this string is as follows:
    ///
    /// DEVICE_SPEC ::= PARTIAL_SPEC
    ///
    /// PARTIAL_SPEC ::= ("/" CONSTRAINT) *
    /// CONSTRAINT ::= ("job:" JOB_NAME)
    ///              | ("replica:" [1-9][0-9]*)
    ///              | ("task:" [1-9][0-9]*)
    ///              | ("device:" [A-Za-z]* ":" ([1-9][0-9]* | "*") )
    ///
    /// Valid values for this string include:
    /// * "/job:worker/replica:0/task:1/device:GPU:3"  (full specification)
    /// * "/job:worker/device:GPU:3"                   (partial specification)
    /// * ""                                    (no specification)
    ///
    /// If the constraints do not resolve to a single device (or if this
    /// field is empty or not present), the runtime will attempt to
    /// choose a device automatically.
    #[prost(string, tag="4")]
    pub device: std::string::String,
    /// Operation-specific graph-construction-time configuration.
    /// Note that this should include all attrs defined in the
    /// corresponding OpDef, including those with a value matching
    /// the default -- this allows the default to change and makes
    /// NodeDefs easier to interpret on their own.  However, if
    /// an attr with a default is not specified in this list, the
    /// default will be used.
    /// The "names" (keys) must match the regexp "[a-z][a-z0-9_]+" (and
    /// one of the names from the corresponding OpDef's attr field).
    /// The values must have a type matching the corresponding OpDef
    /// attr's type field.
    /// TODO(josh11b): Add some examples here showing best practices.
    #[prost(map="string, message", tag="5")]
    pub attr: ::std::collections::HashMap<std::string::String, AttrValue>,
    /// This stores debug information associated with the node.
    #[prost(message, optional, tag="6")]
    pub experimental_debug_info: ::std::option::Option<node_def::ExperimentalDebugInfo>,
}
pub mod node_def {
    #[derive(Clone, PartialEq, ::prost::Message)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct ExperimentalDebugInfo {
        /// Opaque string inserted into error messages created by the runtime.
        ///
        /// This is intended to store the list of names of the nodes from the
        /// original graph that this node was derived. For example if this node, say
        /// C, was result of a fusion of 2 nodes A and B, then 'original_node' would
        /// be {A, B}. This information can be used to map errors originating at the
        /// current node to some top level source code.
        #[prost(string, repeated, tag="1")]
        pub original_node_names: ::std::vec::Vec<std::string::String>,
        /// This is intended to store the list of names of the functions from the
        /// original graph that this node was derived. For example if this node, say
        /// C, was result of a fusion of node A in function FA and node B in function
        /// FB, then `original_funcs` would be {FA, FB}. If the node is in the top
        /// level graph, the `original_func` is empty. This information, with the
        /// `original_node_names` can be used to map errors originating at the
        /// current ndoe to some top level source code.
        #[prost(string, repeated, tag="2")]
        pub original_func_names: ::std::vec::Vec<std::string::String>,
    }
}
/// Defines an operation. A NodeDef in a GraphDef specifies an Op by
/// using the "op" field which should match the name of a OpDef.
/// LINT.IfChange
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct OpDef {
    /// Op names starting with an underscore are reserved for internal use.
    /// Names should be CamelCase and match the regexp "[A-Z][a-zA-Z0-9>_]*".
    #[prost(string, tag="1")]
    pub name: std::string::String,
    /// Description of the input(s).
    #[prost(message, repeated, tag="2")]
    pub input_arg: ::std::vec::Vec<op_def::ArgDef>,
    /// Description of the output(s).
    #[prost(message, repeated, tag="3")]
    pub output_arg: ::std::vec::Vec<op_def::ArgDef>,
    /// Named control outputs for this operation. Useful only for composite
    /// operations (i.e. functions) which want to name different control outputs.
    #[prost(string, repeated, tag="20")]
    pub control_output: ::std::vec::Vec<std::string::String>,
    #[prost(message, repeated, tag="4")]
    pub attr: ::std::vec::Vec<op_def::AttrDef>,
    /// Optional deprecation based on GraphDef versions.
    #[prost(message, optional, tag="8")]
    pub deprecation: ::std::option::Option<OpDeprecation>,
    /// One-line human-readable description of what the Op does.
    #[prost(string, tag="5")]
    pub summary: std::string::String,
    /// Additional, longer human-readable description of what the Op does.
    #[prost(string, tag="6")]
    pub description: std::string::String,
    // -------------------------------------------------------------------------
    // Which optimizations this operation can participate in.

    /// True if the operation is commutative ("op(a,b) == op(b,a)" for all inputs)
    #[prost(bool, tag="18")]
    pub is_commutative: bool,
    /// If is_aggregate is true, then this operation accepts N >= 2
    /// inputs and produces 1 output all of the same type.  Should be
    /// associative and commutative, and produce output with the same
    /// shape as the input.  The optimizer may replace an aggregate op
    /// taking input from multiple devices with a tree of aggregate ops
    /// that aggregate locally within each device (and possibly within
    /// groups of nearby devices) before communicating.
    /// TODO(josh11b): Implement that optimization.
    ///
    /// for things like add
    #[prost(bool, tag="16")]
    pub is_aggregate: bool,
    // Other optimizations go here, like
    //   can_alias_input, rewrite_when_output_unused, partitioning_strategy, etc.

    // -------------------------------------------------------------------------
    // Optimization constraints.

    /// Ops are marked as stateful if their behavior depends on some state beyond
    /// their input tensors (e.g. variable reading op) or if they have
    /// a side-effect (e.g. printing or asserting ops). Equivalently, stateless ops
    /// must always produce the same output for the same input and have
    /// no side-effects.
    ///
    /// By default Ops may be moved between devices.  Stateful ops should
    /// either not be moved, or should only be moved if that state can also
    /// be moved (e.g. via some sort of save / restore).
    /// Stateful ops are guaranteed to never be optimized away by Common
    /// Subexpression Elimination (CSE).
    ///
    /// for things like variables, queue
    #[prost(bool, tag="17")]
    pub is_stateful: bool,
    // -------------------------------------------------------------------------
    // Non-standard options.

    /// By default, all inputs to an Op must be initialized Tensors.  Ops
    /// that may initialize tensors for the first time should set this
    /// field to true, to allow the Op to take an uninitialized Tensor as
    /// input.
    ///
    /// for Assign, etc.
    #[prost(bool, tag="19")]
    pub allows_uninitialized_input: bool,
}
pub mod op_def {
    /// For describing inputs and outputs.
    #[derive(Clone, PartialEq, ::prost::Message)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct ArgDef {
        /// Name for the input/output.  Should match the regexp "[a-z][a-z0-9_]*".
        #[prost(string, tag="1")]
        pub name: std::string::String,
        /// Human readable description.
        #[prost(string, tag="2")]
        pub description: std::string::String,
        /// Describes the type of one or more tensors that are accepted/produced
        /// by this input/output arg.  The only legal combinations are:
        /// * For a single tensor: either the "type" field is set or the
        ///   "type_attr" field is set to the name of an attr with type "type".
        /// * For a sequence of tensors with the same type: the "number_attr"
        ///   field will be set to the name of an attr with type "int", and
        ///   either the "type" or "type_attr" field will be set as for
        ///   single tensors.
        /// * For a sequence of tensors, the "type_list_attr" field will be set
        ///   to the name of an attr with type "list(type)".
        #[prost(enumeration="super::DataType", tag="3")]
        pub r#type: i32,
        /// if specified, attr must have type "type"
        #[prost(string, tag="4")]
        pub type_attr: std::string::String,
        /// if specified, attr must have type "int"
        #[prost(string, tag="5")]
        pub number_attr: std::string::String,
        /// If specified, attr must have type "list(type)", and none of
        /// type, type_attr, and number_attr may be specified.
        #[prost(string, tag="6")]
        pub type_list_attr: std::string::String,
        /// For inputs: if true, the inputs are required to be refs.
        ///   By default, inputs can be either refs or non-refs.
        /// For outputs: if true, outputs are refs, otherwise they are not.
        #[prost(bool, tag="16")]
        pub is_ref: bool,
    }
    /// Description of the graph-construction-time configuration of this
    /// Op.  That is to say, this describes the attr fields that will
    /// be specified in the NodeDef.
    #[derive(Clone, PartialEq, ::prost::Message)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct AttrDef {
        /// A descriptive name for the argument.  May be used, e.g. by the
        /// Python client, as a keyword argument name, and so should match
        /// the regexp "[a-z][a-z0-9_]+".
        #[prost(string, tag="1")]
        pub name: std::string::String,
        /// One of the type names from attr_value.proto ("string", "list(string)",
        /// "int", etc.).
        #[prost(string, tag="2")]
        pub r#type: std::string::String,
        /// A reasonable default for this attribute if the user does not supply
        /// a value.  If not specified, the user must supply a value.
        #[prost(message, optional, tag="3")]
        pub default_value: ::std::option::Option<super::AttrValue>,
        /// Human-readable description.
        #[prost(string, tag="4")]
        pub description: std::string::String,
        // TODO(josh11b): bool is_optional?

        // --- Constraints ---
        // These constraints are only in effect if specified.  Default is no
        // constraints.

        /// For type == "int", this is a minimum value.  For "list(___)"
        /// types, this is the minimum length.
        #[prost(bool, tag="5")]
        pub has_minimum: bool,
        #[prost(int64, tag="6")]
        pub minimum: i64,
        /// The set of allowed values.  Has type that is the "list" version
        /// of the "type" field above (uses the "list" field of AttrValue).
        /// If type == "type" or "list(type)" above, then the "type" field
        /// of "allowed_values.list" has the set of allowed DataTypes.
        /// If type == "string" or "list(string)", then the "s" field of
        /// "allowed_values.list" has the set of allowed strings.
        #[prost(message, optional, tag="7")]
        pub allowed_values: ::std::option::Option<super::AttrValue>,
    }
}
/// Information about version-dependent deprecation of an op
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct OpDeprecation {
    /// First GraphDef version at which the op is disallowed.
    #[prost(int32, tag="1")]
    pub version: i32,
    /// Explanation of why it was deprecated and what to use instead.
    #[prost(string, tag="2")]
    pub explanation: std::string::String,
}
/// A collection of OpDefs
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct OpList {
    #[prost(message, repeated, tag="1")]
    pub op: ::std::vec::Vec<OpDef>,
}
/// A library is a set of named functions.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct FunctionDefLibrary {
    #[prost(message, repeated, tag="1")]
    pub function: ::std::vec::Vec<FunctionDef>,
    #[prost(message, repeated, tag="2")]
    pub gradient: ::std::vec::Vec<GradientDef>,
}
/// A function can be instantiated when the runtime can bind every attr
/// with a value. When a GraphDef has a call to a function, it must
/// have binding for every attr defined in the signature.
///
/// TODO(zhifengc):
///   * device spec, etc.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct FunctionDef {
    /// The definition of the function's name, arguments, return values,
    /// attrs etc.
    #[prost(message, optional, tag="1")]
    pub signature: ::std::option::Option<OpDef>,
    /// Attributes specific to this function definition.
    #[prost(map="string, message", tag="5")]
    pub attr: ::std::collections::HashMap<std::string::String, AttrValue>,
    #[prost(map="uint32, message", tag="7")]
    pub arg_attr: ::std::collections::HashMap<u32, function_def::ArgAttrs>,
    /// Unique IDs for each resource argument, used to track aliasing resources. If
    /// Argument A and Argument B alias each other, then
    /// resource_arg_unique_ids[A.index] == resource_arg_unique_ids[B.index].
    ///
    /// If this field is empty, none of the arguments could alias; otherwise, every
    /// resource argument should have an entry in this field.
    ///
    /// When instantiated, the unique IDs will be attached to the _Arg nodes'
    /// "_resource_arg_unique_id" attribute.
    #[prost(map="uint32, uint32", tag="8")]
    pub resource_arg_unique_id: ::std::collections::HashMap<u32, u32>,
    // In both of the following fields, there is the need to specify an
    // output that is used as either the input to another node (in
    // `node_def`) or as a return value of the function (in `ret`).
    // Unlike the NodeDefs in GraphDef, we need to be able to specify a
    // list in some cases (instead of just single outputs).  Also, we
    // need to be able to deal with lists of unknown length (so the
    // output index may not be known at function definition time).  So
    // we use the following format instead:
    // * "fun_in" where "fun_in" is the name of a function input arg in
    //   the `signature` field above.  This represents that input, whether
    //   it is a single tensor or a list.
    // * "fun_in:0" gives the first element of a function input arg (a
    //   non-list input is considered a list of length 1 for these
    //   purposes).
    // * "node:out" where "node" is the name of a node in `node_def` and
    //   "out" is the name one of its op's output arguments (the name
    //   comes from the OpDef of the node's op). This represents that
    //   node's output, whether it is a single tensor or a list.
    //   Note: We enforce that an op's output arguments are never
    //   renamed in the backwards-compatibility test.
    // * "node:out:0" gives the first element of a node output arg (a
    //   non-list output is considered a list of length 1 for these
    //   purposes).
    //
    // NOT CURRENTLY SUPPORTED (but may be in the future):
    // * "node:out:-1" gives last element in a node output list
    // * "node:out:1:" gives a list with all but the first element in a
    //   node output list
    // * "node:out::-1" gives a list with all but the last element in a
    //   node output list

    // The body of the function.  Unlike the NodeDefs in a GraphDef, attrs
    // may have values of type `placeholder` and the `input` field uses
    // the "output" format above.

    /// By convention, "op" in node_def is resolved by consulting with a
    /// user-defined library first. If not resolved, "func" is assumed to
    /// be a builtin op.
    #[prost(message, repeated, tag="3")]
    pub node_def: ::std::vec::Vec<NodeDef>,
    /// A mapping from the output arg names from `signature` to the
    /// outputs from `node_def` that should be returned by the function.
    #[prost(map="string, string", tag="4")]
    pub ret: ::std::collections::HashMap<std::string::String, std::string::String>,
    /// A mapping from control output names from `signature` to node names in
    /// `node_def` which should be control outputs of this function.
    #[prost(map="string, string", tag="6")]
    pub control_ret: ::std::collections::HashMap<std::string::String, std::string::String>,
}
pub mod function_def {
    /// Attributes for function arguments. These attributes are the same set of
    /// valid attributes as to _Arg nodes.
    #[derive(Clone, PartialEq, ::prost::Message)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct ArgAttrs {
        #[prost(map="string, message", tag="1")]
        pub attr: ::std::collections::HashMap<std::string::String, super::AttrValue>,
    }
}
/// GradientDef defines the gradient function of a function defined in
/// a function library.
///
/// A gradient function g (specified by gradient_func) for a function f
/// (specified by function_name) must follow the following:
///
/// The function 'f' must be a numerical function which takes N inputs
/// and produces M outputs. Its gradient function 'g', which is a
/// function taking N + M inputs and produces N outputs.
///
/// I.e. if we have
///    (y1, y2, ..., y_M) = f(x1, x2, ..., x_N),
/// then, g is
///    (dL/dx1, dL/dx2, ..., dL/dx_N) = g(x1, x2, ..., x_N,
///                                      dL/dy1, dL/dy2, ..., dL/dy_M),
/// where L is a scalar-value function of (x1, x2, ..., xN) (e.g., the
/// loss function). dL/dx_i is the partial derivative of L with respect
/// to x_i.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct GradientDef {
    /// The function name.
    #[prost(string, tag="1")]
    pub function_name: std::string::String,
    /// The gradient function's name.
    #[prost(string, tag="2")]
    pub gradient_func: std::string::String,
}
/// Version information for a piece of serialized data
///
/// There are different types of versions for each type of data
/// (GraphDef, etc.), but they all have the same common shape
/// described here.
///
/// Each consumer has "consumer" and "min_producer" versions (specified
/// elsewhere).  A consumer is allowed to consume this data if
///
///   producer >= min_producer
///   consumer >= min_consumer
///   consumer not in bad_consumers
///
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct VersionDef {
    /// The version of the code that produced this data.
    #[prost(int32, tag="1")]
    pub producer: i32,
    /// Any consumer below this version is not allowed to consume this data.
    #[prost(int32, tag="2")]
    pub min_consumer: i32,
    /// Specific consumer versions which are disallowed (e.g. due to bugs).
    #[prost(int32, repeated, tag="3")]
    pub bad_consumers: ::std::vec::Vec<i32>,
}
/// Represents the graph of operations
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct GraphDef {
    #[prost(message, repeated, tag="1")]
    pub node: ::std::vec::Vec<NodeDef>,
    /// Compatibility versions of the graph.  See core/public/version.h for version
    /// history.  The GraphDef version is distinct from the TensorFlow version, and
    /// each release of TensorFlow will support a range of GraphDef versions.
    #[prost(message, optional, tag="4")]
    pub versions: ::std::option::Option<VersionDef>,
    /// Deprecated single version field; use versions above instead.  Since all
    /// GraphDef changes before "versions" was introduced were forward
    /// compatible, this field is entirely ignored.
    #[prost(int32, tag="3")]
    pub version: i32,
    /// EXPERIMENTAL. DO NOT USE OR DEPEND ON THIS YET.
    ///
    /// "library" provides user-defined functions.
    ///
    /// Naming:
    ///   * library.function.name are in a flat namespace.
    ///     NOTE: We may need to change it to be hierarchical to support
    ///     different orgs. E.g.,
    ///     { "/google/nn", { ... }},
    ///     { "/google/vision", { ... }}
    ///     { "/org_foo/module_bar", { ... }}
    ///     map<string, FunctionDefLib> named_lib;
    ///   * If node[i].op is the name of one function in "library",
    ///     node[i] is deemed as a function call. Otherwise, node[i].op
    ///     must be a primitive operation supported by the runtime.
    ///
    ///
    /// Function call semantics:
    ///
    ///   * The callee may start execution as soon as some of its inputs
    ///     are ready. The caller may want to use Tuple() mechanism to
    ///     ensure all inputs are ready in the same time.
    ///
    ///   * The consumer of return values may start executing as soon as
    ///     the return values the consumer depends on are ready.  The
    ///     consumer may want to use Tuple() mechanism to ensure the
    ///     consumer does not start until all return values of the callee
    ///     function are ready.
    #[prost(message, optional, tag="2")]
    pub library: ::std::option::Option<FunctionDefLibrary>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct GraphTransferNodeInput {
    #[prost(int32, tag="1")]
    pub node_id: i32,
    #[prost(int32, tag="2")]
    pub output_port: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct GraphTransferNodeInfo {
    #[prost(string, tag="1")]
    pub name: std::string::String,
    #[prost(int32, tag="2")]
    pub node_id: i32,
    #[prost(string, tag="3")]
    pub type_name: std::string::String,
    #[prost(int32, tag="4")]
    pub soc_op_id: i32,
    #[prost(int32, tag="5")]
    pub padding_id: i32,
    #[prost(int32, tag="6")]
    pub input_count: i32,
    #[prost(int32, tag="7")]
    pub output_count: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct GraphTransferConstNodeInfo {
    #[prost(string, tag="1")]
    pub name: std::string::String,
    #[prost(int32, tag="2")]
    pub node_id: i32,
    #[prost(int64, repeated, tag="3")]
    pub shape: ::std::vec::Vec<i64>,
    #[prost(bytes, tag="4")]
    pub data: std::vec::Vec<u8>,
    #[prost(enumeration="DataType", tag="5")]
    pub dtype: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct GraphTransferNodeInputInfo {
    #[prost(int32, tag="1")]
    pub node_id: i32,
    #[prost(message, repeated, tag="2")]
    pub node_input: ::std::vec::Vec<GraphTransferNodeInput>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct GraphTransferNodeOutputInfo {
    #[prost(int32, tag="1")]
    pub node_id: i32,
    #[prost(int32, repeated, tag="2")]
    pub max_byte_size: ::std::vec::Vec<i32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct GraphTransferGraphInputNodeInfo {
    #[prost(string, tag="1")]
    pub name: std::string::String,
    #[prost(int64, repeated, tag="2")]
    pub shape: ::std::vec::Vec<i64>,
    #[prost(enumeration="DataType", tag="3")]
    pub dtype: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct GraphTransferGraphOutputNodeInfo {
    #[prost(string, tag="1")]
    pub name: std::string::String,
    #[prost(int64, repeated, tag="2")]
    pub shape: ::std::vec::Vec<i64>,
    #[prost(enumeration="DataType", tag="3")]
    pub dtype: i32,
}
/// Protocol buffer representing a handle to a tensorflow resource. Handles are
/// not valid across executions, but can be serialized back and forth from within
/// a single run.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct GraphTransferInfo {
    #[prost(message, repeated, tag="1")]
    pub node_info: ::std::vec::Vec<GraphTransferNodeInfo>,
    #[prost(message, repeated, tag="2")]
    pub const_node_info: ::std::vec::Vec<GraphTransferConstNodeInfo>,
    #[prost(message, repeated, tag="3")]
    pub node_input_info: ::std::vec::Vec<GraphTransferNodeInputInfo>,
    #[prost(message, repeated, tag="4")]
    pub node_output_info: ::std::vec::Vec<GraphTransferNodeOutputInfo>,
    /// Input Node parameters of transferred graph
    #[prost(message, repeated, tag="5")]
    pub graph_input_node_info: ::std::vec::Vec<GraphTransferGraphInputNodeInfo>,
    #[prost(message, repeated, tag="6")]
    pub graph_output_node_info: ::std::vec::Vec<GraphTransferGraphOutputNodeInfo>,
    /// Destination of graph transfer
    #[prost(enumeration="graph_transfer_info::Destination", tag="7")]
    pub destination: i32,
}
pub mod graph_transfer_info {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub enum Destination {
        Nop = 0,
        Hexagon = 1,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct KernelDef {
    /// Must match the name of an Op.
    #[prost(string, tag="1")]
    pub op: std::string::String,
    /// Type of device this kernel runs on.
    #[prost(string, tag="2")]
    pub device_type: std::string::String,
    #[prost(message, repeated, tag="3")]
    pub constraint: ::std::vec::Vec<kernel_def::AttrConstraint>,
    /// Names of the Op's input_/output_args that reside in host memory
    /// instead of device memory.
    #[prost(string, repeated, tag="4")]
    pub host_memory_arg: ::std::vec::Vec<std::string::String>,
    /// This allows experimental kernels to be registered for an op that
    /// won't be used unless the user specifies a "_kernel" attr with
    /// value matching this.
    #[prost(string, tag="5")]
    pub label: std::string::String,
    /// Prioritization of kernel amongst different devices. By default we assume
    /// priority is 0. The higher the priority the better. By default (i.e. if
    /// this is not set), we prefer GPU kernels over CPU.
    #[prost(int32, tag="6")]
    pub priority: i32,
}
pub mod kernel_def {
    #[derive(Clone, PartialEq, ::prost::Message)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct AttrConstraint {
        /// Name of an attr from the Op.
        #[prost(string, tag="1")]
        pub name: std::string::String,
        /// A list of values that this kernel supports for this attr.
        /// Like OpDef.AttrDef.allowed_values, except for kernels instead of Ops.
        #[prost(message, optional, tag="2")]
        pub allowed_values: ::std::option::Option<super::AttrValue>,
    }
}
/// A collection of KernelDefs
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct KernelList {
    #[prost(message, repeated, tag="1")]
    pub kernel: ::std::vec::Vec<KernelDef>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct TensorDescription {
    /// Data type of tensor elements
    #[prost(enumeration="DataType", tag="1")]
    pub dtype: i32,
    /// Shape of the tensor.
    #[prost(message, optional, tag="2")]
    pub shape: ::std::option::Option<TensorShapeProto>,
    /// Information about the size and allocator used for the data
    #[prost(message, optional, tag="4")]
    pub allocation_description: ::std::option::Option<AllocationDescription>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MemoryLogStep {
    /// Process-unique step id.
    #[prost(int64, tag="1")]
    pub step_id: i64,
    /// Handle describing the feeds and fetches of the step.
    #[prost(string, tag="2")]
    pub handle: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MemoryLogTensorAllocation {
    /// Process-unique step id.
    #[prost(int64, tag="1")]
    pub step_id: i64,
    /// Name of the kernel making the allocation as set in GraphDef,
    /// e.g., "affine2/weights/Assign".
    #[prost(string, tag="2")]
    pub kernel_name: std::string::String,
    /// Allocated tensor details.
    #[prost(message, optional, tag="3")]
    pub tensor: ::std::option::Option<TensorDescription>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MemoryLogTensorDeallocation {
    /// Id of the tensor buffer being deallocated, used to match to a
    /// corresponding allocation.
    #[prost(int64, tag="1")]
    pub allocation_id: i64,
    /// Name of the allocator used.
    #[prost(string, tag="2")]
    pub allocator_name: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MemoryLogTensorOutput {
    /// Process-unique step id.
    #[prost(int64, tag="1")]
    pub step_id: i64,
    /// Name of the kernel producing an output as set in GraphDef, e.g.,
    /// "affine2/weights/Assign".
    #[prost(string, tag="2")]
    pub kernel_name: std::string::String,
    /// Index of the output being set.
    #[prost(int32, tag="3")]
    pub index: i32,
    /// Output tensor details.
    #[prost(message, optional, tag="4")]
    pub tensor: ::std::option::Option<TensorDescription>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MemoryLogRawAllocation {
    /// Process-unique step id.
    #[prost(int64, tag="1")]
    pub step_id: i64,
    /// Name of the operation making the allocation.
    #[prost(string, tag="2")]
    pub operation: std::string::String,
    /// Number of bytes in the allocation.
    #[prost(int64, tag="3")]
    pub num_bytes: i64,
    /// Address of the allocation.
    #[prost(uint64, tag="4")]
    pub ptr: u64,
    /// Id of the tensor buffer being allocated, used to match to a
    /// corresponding deallocation.
    #[prost(int64, tag="5")]
    pub allocation_id: i64,
    /// Name of the allocator used.
    #[prost(string, tag="6")]
    pub allocator_name: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MemoryLogRawDeallocation {
    /// Process-unique step id.
    #[prost(int64, tag="1")]
    pub step_id: i64,
    /// Name of the operation making the deallocation.
    #[prost(string, tag="2")]
    pub operation: std::string::String,
    /// Id of the tensor buffer being deallocated, used to match to a
    /// corresponding allocation.
    #[prost(int64, tag="3")]
    pub allocation_id: i64,
    /// Name of the allocator used.
    #[prost(string, tag="4")]
    pub allocator_name: std::string::String,
    /// True if the deallocation is queued and will be performed later,
    /// e.g. for GPU lazy freeing of buffers.
    #[prost(bool, tag="5")]
    pub deferred: bool,
}
/// For serializing and restoring the state of ReaderBase, see
/// reader_base.h for details.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ReaderBaseState {
    #[prost(int64, tag="1")]
    pub work_started: i64,
    #[prost(int64, tag="2")]
    pub work_finished: i64,
    #[prost(int64, tag="3")]
    pub num_records_produced: i64,
    #[prost(bytes, tag="4")]
    pub current_work: std::vec::Vec<u8>,
}
/// Protocol buffer representing a handle to a tensorflow resource. Handles are
/// not valid across executions, but can be serialized back and forth from within
/// a single run.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct RemoteFusedGraphExecuteInfo {
    /// Definition of remote graph
    #[prost(message, optional, tag="1")]
    pub remote_graph: ::std::option::Option<GraphDef>,
    /// Remote fused graph input node name
    #[prost(string, repeated, tag="2")]
    pub graph_input_node_name: ::std::vec::Vec<std::string::String>,
    /// Remote fused graph output node name
    #[prost(string, repeated, tag="3")]
    pub graph_output_node_name: ::std::vec::Vec<std::string::String>,
    /// Executor's name
    #[prost(string, tag="4")]
    pub executor_name: std::string::String,
    /// Optional: Parameters given to the executor
    #[prost(bytes, tag="5")]
    pub serialized_executor_parameters: std::vec::Vec<u8>,
    /// Optional: Default graph input tensor shape used to allocate memory
    /// before executing op
    #[prost(message, repeated, tag="6")]
    pub default_graph_input_tensor_shape: ::std::vec::Vec<remote_fused_graph_execute_info::TensorShapeTypeProto>,
    /// Optional: Default graph input tensor shape used to allocate memory
    /// before executing op
    /// TODO(satok): Remote output tensor shape once shape information is stored
    /// in NodeDef
    #[prost(message, repeated, tag="7")]
    pub default_graph_output_tensor_shape: ::std::vec::Vec<remote_fused_graph_execute_info::TensorShapeTypeProto>,
}
pub mod remote_fused_graph_execute_info {
    #[derive(Clone, PartialEq, ::prost::Message)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct TensorShapeTypeProto {
        #[prost(enumeration="super::DataType", tag="1")]
        pub dtype: i32,
        #[prost(message, optional, tag="2")]
        pub shape: ::std::option::Option<super::TensorShapeProto>,
    }
}
/// An allocation/de-allocation operation performed by the allocator.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct AllocationRecord {
    /// The timestamp of the operation.
    #[prost(int64, tag="1")]
    pub alloc_micros: i64,
    /// Number of bytes allocated, or de-allocated if negative.
    #[prost(int64, tag="2")]
    pub alloc_bytes: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct AllocatorMemoryUsed {
    #[prost(string, tag="1")]
    pub allocator_name: std::string::String,
    /// These are per-node allocator memory stats.
    #[prost(int64, tag="2")]
    pub total_bytes: i64,
    #[prost(int64, tag="3")]
    pub peak_bytes: i64,
    /// The bytes that are not deallocated.
    #[prost(int64, tag="4")]
    pub live_bytes: i64,
    /// The allocation and deallocation timeline.
    #[prost(message, repeated, tag="6")]
    pub allocation_records: ::std::vec::Vec<AllocationRecord>,
    /// These are snapshots of the overall allocator memory stats.
    /// The number of live bytes currently allocated by the allocator.
    #[prost(int64, tag="5")]
    pub allocator_bytes_in_use: i64,
}
/// Output sizes recorded for a single execution of a graph node.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct NodeOutput {
    #[prost(int32, tag="1")]
    pub slot: i32,
    #[prost(message, optional, tag="3")]
    pub tensor_description: ::std::option::Option<TensorDescription>,
}
/// For memory tracking.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MemoryStats {
    #[prost(int64, tag="1")]
    pub temp_memory_size: i64,
    #[prost(int64, tag="3")]
    pub persistent_memory_size: i64,
    #[prost(int64, repeated, tag="5")]
    pub persistent_tensor_alloc_ids: ::std::vec::Vec<i64>,
    #[prost(int64, tag="2")]
    pub device_temp_memory_size: i64,
    #[prost(int64, tag="4")]
    pub device_persistent_memory_size: i64,
    #[prost(int64, repeated, packed="false", tag="6")]
    pub device_persistent_tensor_alloc_ids: ::std::vec::Vec<i64>,
}
/// Time/size stats recorded for a single execution of a graph node.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct NodeExecStats {
    /// TODO(tucker): Use some more compact form of node identity than
    /// the full string name.  Either all processes should agree on a
    /// global id (cost_id?) for each node, or we should use a hash of
    /// the name.
    #[prost(string, tag="1")]
    pub node_name: std::string::String,
    #[prost(int64, tag="2")]
    pub all_start_micros: i64,
    #[prost(int64, tag="3")]
    pub op_start_rel_micros: i64,
    #[prost(int64, tag="4")]
    pub op_end_rel_micros: i64,
    #[prost(int64, tag="5")]
    pub all_end_rel_micros: i64,
    #[prost(message, repeated, tag="6")]
    pub memory: ::std::vec::Vec<AllocatorMemoryUsed>,
    #[prost(message, repeated, tag="7")]
    pub output: ::std::vec::Vec<NodeOutput>,
    #[prost(string, tag="8")]
    pub timeline_label: std::string::String,
    #[prost(int64, tag="9")]
    pub scheduled_micros: i64,
    #[prost(uint32, tag="10")]
    pub thread_id: u32,
    #[prost(message, repeated, tag="11")]
    pub referenced_tensor: ::std::vec::Vec<AllocationDescription>,
    #[prost(message, optional, tag="12")]
    pub memory_stats: ::std::option::Option<MemoryStats>,
    #[prost(int64, tag="13")]
    pub all_start_nanos: i64,
    #[prost(int64, tag="14")]
    pub op_start_rel_nanos: i64,
    #[prost(int64, tag="15")]
    pub op_end_rel_nanos: i64,
    #[prost(int64, tag="16")]
    pub all_end_rel_nanos: i64,
    #[prost(int64, tag="17")]
    pub scheduled_nanos: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct DeviceStepStats {
    #[prost(string, tag="1")]
    pub device: std::string::String,
    #[prost(message, repeated, tag="2")]
    pub node_stats: ::std::vec::Vec<NodeExecStats>,
    /// Its key is thread id.
    #[prost(map="uint32, string", tag="3")]
    pub thread_names: ::std::collections::HashMap<u32, std::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct StepStats {
    #[prost(message, repeated, tag="1")]
    pub dev_stats: ::std::vec::Vec<DeviceStepStats>,
}
/// Metadata associated with a series of Summary data
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SummaryDescription {
    /// Hint on how plugins should process the data in this series.
    /// Supported values include "scalar", "histogram", "image", "audio"
    #[prost(string, tag="1")]
    pub type_hint: std::string::String,
}
/// Serialization format for histogram module in
/// core/lib/histogram/histogram.h
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct HistogramProto {
    #[prost(double, tag="1")]
    pub min: f64,
    #[prost(double, tag="2")]
    pub max: f64,
    #[prost(double, tag="3")]
    pub num: f64,
    #[prost(double, tag="4")]
    pub sum: f64,
    #[prost(double, tag="5")]
    pub sum_squares: f64,
    /// Parallel arrays encoding the bucket boundaries and the bucket values.
    /// bucket(i) is the count for the bucket i.  The range for
    /// a bucket is:
    ///   i == 0:  -DBL_MAX .. bucket_limit(0)
    ///   i != 0:  bucket_limit(i-1) .. bucket_limit(i)
    #[prost(double, repeated, tag="6")]
    pub bucket_limit: ::std::vec::Vec<f64>,
    #[prost(double, repeated, tag="7")]
    pub bucket: ::std::vec::Vec<f64>,
}
/// A SummaryMetadata encapsulates information on which plugins are able to make
/// use of a certain summary value.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SummaryMetadata {
    /// Data that associates a summary with a certain plugin.
    #[prost(message, optional, tag="1")]
    pub plugin_data: ::std::option::Option<summary_metadata::PluginData>,
    /// Display name for viewing in TensorBoard.
    #[prost(string, tag="2")]
    pub display_name: std::string::String,
    /// Longform readable description of the summary sequence. Markdown supported.
    #[prost(string, tag="3")]
    pub summary_description: std::string::String,
    /// Class of data stored in this time series. Required for compatibility with
    /// TensorBoard's generic data facilities (`DataProvider`, et al.). This value
    /// imposes constraints on the dtype and shape of the corresponding tensor
    /// values. See `DataClass` docs for details.
    #[prost(enumeration="DataClass", tag="4")]
    pub data_class: i32,
}
pub mod summary_metadata {
    #[derive(Clone, PartialEq, ::prost::Message)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct PluginData {
        /// The name of the plugin this data pertains to.
        #[prost(string, tag="1")]
        pub plugin_name: std::string::String,
        /// The content to store for the plugin. The best practice is for this to be
        /// a binary serialized protocol buffer.
        #[prost(bytes, tag="2")]
        pub content: std::vec::Vec<u8>,
    }
}
/// A Summary is a set of named values to be displayed by the
/// visualizer.
///
/// Summaries are produced regularly during training, as controlled by
/// the "summary_interval_secs" attribute of the training operation.
/// Summaries are also produced at the end of an evaluation.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Summary {
    /// Set of values for the summary.
    #[prost(message, repeated, tag="1")]
    pub value: ::std::vec::Vec<summary::Value>,
}
pub mod summary {
    #[derive(Clone, PartialEq, ::prost::Message)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Image {
        /// Dimensions of the image.
        #[prost(int32, tag="1")]
        pub height: i32,
        #[prost(int32, tag="2")]
        pub width: i32,
        /// Valid colorspace values are
        ///   1 - grayscale
        ///   2 - grayscale + alpha
        ///   3 - RGB
        ///   4 - RGBA
        ///   5 - DIGITAL_YUV
        ///   6 - BGRA
        #[prost(int32, tag="3")]
        pub colorspace: i32,
        /// Image data in encoded format.  All image formats supported by
        /// image_codec::CoderUtil can be stored here.
        #[prost(bytes, tag="4")]
        pub encoded_image_string: std::vec::Vec<u8>,
    }
    #[derive(Clone, PartialEq, ::prost::Message)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Audio {
        /// Sample rate of the audio in Hz.
        #[prost(float, tag="1")]
        pub sample_rate: f32,
        /// Number of channels of audio.
        #[prost(int64, tag="2")]
        pub num_channels: i64,
        /// Length of the audio in frames (samples per channel).
        #[prost(int64, tag="3")]
        pub length_frames: i64,
        /// Encoded audio data and its associated RFC 2045 content type (e.g.
        /// "audio/wav").
        #[prost(bytes, tag="4")]
        pub encoded_audio_string: std::vec::Vec<u8>,
        #[prost(string, tag="5")]
        pub content_type: std::string::String,
    }
    #[derive(Clone, PartialEq, ::prost::Message)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Value {
        /// This field is deprecated and will not be set.
        #[prost(string, tag="7")]
        pub node_name: std::string::String,
        /// Tag name for the data. Used by TensorBoard plugins to organize data. Tags
        /// are often organized by scope (which contains slashes to convey
        /// hierarchy). For example: foo/bar/0
        #[prost(string, tag="1")]
        pub tag: std::string::String,
        /// Contains metadata on the summary value such as which plugins may use it.
        /// Take note that many summary values may lack a metadata field. This is
        /// because the FileWriter only keeps a metadata object on the first summary
        /// value with a certain tag for each tag. TensorBoard then remembers which
        /// tags are associated with which plugins. This saves space.
        #[prost(message, optional, tag="9")]
        pub metadata: ::std::option::Option<super::SummaryMetadata>,
        /// Value associated with the tag.
        #[prost(oneof="value::Value", tags="2, 3, 4, 5, 6, 8")]
        pub value: ::std::option::Option<value::Value>,
    }
    pub mod value {
        /// Value associated with the tag.
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        #[derive(serde::Serialize, serde::Deserialize)]
        pub enum Value {
            #[prost(float, tag="2")]
            SimpleValue(f32),
            #[prost(bytes, tag="3")]
            ObsoleteOldStyleHistogram(std::vec::Vec<u8>),
            #[prost(message, tag="4")]
            Image(super::Image),
            #[prost(message, tag="5")]
            Histo(super::super::HistogramProto),
            #[prost(message, tag="6")]
            Audio(super::Audio),
            #[prost(message, tag="8")]
            Tensor(super::super::TensorProto),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum DataClass {
    /// Unknown data class, used (implicitly) for legacy data. Will not be
    /// processed by data ingestion pipelines.
    Unknown = 0,
    /// Scalar time series. Each `Value` for the corresponding tag must have
    /// `tensor` set to a rank-0 tensor of floating-point dtype, which will be
    /// converted to float64.
    Scalar = 1,
    /// Tensor time series. Each `Value` for the corresponding tag must have
    /// `tensor` set. The tensor value is arbitrary, but should be small to
    /// accommodate direct storage in database backends: an upper bound of a few
    /// kilobytes is a reasonable rule of thumb.
    Tensor = 2,
    /// Blob sequence time series. Each `Value` for the corresponding tag must
    /// have `tensor` set to a rank-1 tensor of bytestring dtype.
    BlobSequence = 3,
}
/// Can only be interpreted if you know the corresponding TensorShape.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct TensorSliceProto {
    /// Extent of the slice in all tensor dimensions.
    ///
    /// Must have one entry for each of the dimension of the tensor that this
    /// slice belongs to.  The order of sizes is the same as the order of
    /// dimensions in the TensorShape.
    #[prost(message, repeated, tag="1")]
    pub extent: ::std::vec::Vec<tensor_slice_proto::Extent>,
}
pub mod tensor_slice_proto {
    /// Extent of the slice in one dimension.
    ///
    /// Either both or no attributes must be set.  When no attribute is set
    /// means: All data in that dimension.
    #[derive(Clone, PartialEq, ::prost::Message)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Extent {
        /// Start index of the slice, starting at 0.
        #[prost(int64, tag="1")]
        pub start: i64,
        /// Length of the slice: if the length is missing or -1 we will
        /// interpret this as "everything in this dimension".  We use
        /// "oneof" to preserve information about whether the length is
        /// present without changing the serialization format from the
        /// prior proto2 version of this proto.
        #[prost(oneof="extent::HasLength", tags="2")]
        pub has_length: ::std::option::Option<extent::HasLength>,
    }
    pub mod extent {
        /// Length of the slice: if the length is missing or -1 we will
        /// interpret this as "everything in this dimension".  We use
        /// "oneof" to preserve information about whether the length is
        /// present without changing the serialization format from the
        /// prior proto2 version of this proto.
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        #[derive(serde::Serialize, serde::Deserialize)]
        pub enum HasLength {
            #[prost(int64, tag="2")]
            Length(i64),
        }
    }
}
/// Protocol buffer representing a Variable.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct VariableDef {
    /// Name of the variable tensor.
    #[prost(string, tag="1")]
    pub variable_name: std::string::String,
    /// Name of the tensor holding the variable's initial value.
    #[prost(string, tag="6")]
    pub initial_value_name: std::string::String,
    /// Name of the initializer op.
    #[prost(string, tag="2")]
    pub initializer_name: std::string::String,
    /// Name of the snapshot tensor.
    #[prost(string, tag="3")]
    pub snapshot_name: std::string::String,
    /// Support for saving variables as slices of a larger variable.
    #[prost(message, optional, tag="4")]
    pub save_slice_info_def: ::std::option::Option<SaveSliceInfoDef>,
    /// Whether to represent this as a ResourceVariable.
    #[prost(bool, tag="5")]
    pub is_resource: bool,
    /// Whether this variable should be trained.
    #[prost(bool, tag="7")]
    pub trainable: bool,
    /// Indicates when a distributed variable will be synced.
    #[prost(enumeration="VariableSynchronization", tag="8")]
    pub synchronization: i32,
    /// Indicates how a distributed variable will be aggregated.
    #[prost(enumeration="VariableAggregation", tag="9")]
    pub aggregation: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SaveSliceInfoDef {
    /// Name of the full variable of which this is a slice.
    #[prost(string, tag="1")]
    pub full_name: std::string::String,
    /// Shape of the full variable.
    #[prost(int64, repeated, tag="2")]
    pub full_shape: ::std::vec::Vec<i64>,
    /// Offset of this variable into the full variable.
    #[prost(int64, repeated, tag="3")]
    pub var_offset: ::std::vec::Vec<i64>,
    /// Shape of this variable.
    #[prost(int64, repeated, tag="4")]
    pub var_shape: ::std::vec::Vec<i64>,
}
/// Indicates when a distributed variable will be synced.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum VariableSynchronization {
    /// `AUTO`: Indicates that the synchronization will be determined by the
    /// current `DistributionStrategy` (eg. With `MirroredStrategy` this would be
    /// `ON_WRITE`).
    Auto = 0,
    /// `NONE`: Indicates that there will only be one copy of the variable, so
    /// there is no need to sync.
    None = 1,
    /// `ON_WRITE`: Indicates that the variable will be updated across devices
    /// every time it is written.
    OnWrite = 2,
    /// `ON_READ`: Indicates that the variable will be aggregated across devices
    /// when it is read (eg. when checkpointing or when evaluating an op that uses
    /// the variable).
    OnRead = 3,
}
/// Indicates how a distributed variable will be aggregated.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum VariableAggregation {
    /// `NONE`: This is the default, giving an error if you use a
    /// variable-update operation with multiple replicas.
    None = 0,
    /// `SUM`: Add the updates across replicas.
    Sum = 1,
    /// `MEAN`: Take the arithmetic mean ("average") of the updates across
    /// replicas.
    Mean = 2,
    /// `ONLY_FIRST_REPLICA`: This is for when every replica is performing the same
    /// update, but we only want to perform the update once. Used, e.g., for the
    /// global step counter.
    OnlyFirstReplica = 3,
}
/// Protocol buffer representing an event that happened during
/// the execution of a Brain model.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Event {
    /// Timestamp of the event.
    #[prost(double, tag="1")]
    pub wall_time: f64,
    /// Global step of the event.
    #[prost(int64, tag="2")]
    pub step: i64,
    #[prost(oneof="event::What", tags="3, 4, 5, 6, 7, 8, 9")]
    pub what: ::std::option::Option<event::What>,
}
pub mod event {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub enum What {
        /// An event file was started, with the specified version.
        /// This is use to identify the contents of the record IO files
        /// easily.  Current version is "brain.Event:2".  All versions
        /// start with "brain.Event:".
        #[prost(string, tag="3")]
        FileVersion(std::string::String),
        /// An encoded version of a GraphDef.
        #[prost(bytes, tag="4")]
        GraphDef(std::vec::Vec<u8>),
        /// A summary was generated.
        #[prost(message, tag="5")]
        Summary(super::Summary),
        /// The user output a log message. Not all messages are logged, only ones
        /// generated via the Python tensorboard_logging module.
        #[prost(message, tag="6")]
        LogMessage(super::LogMessage),
        /// The state of the session which can be used for restarting after crashes.
        #[prost(message, tag="7")]
        SessionLog(super::SessionLog),
        /// The metadata returned by running a session.run() call.
        #[prost(message, tag="8")]
        TaggedRunMetadata(super::TaggedRunMetadata),
        /// An encoded version of a MetaGraphDef.
        #[prost(bytes, tag="9")]
        MetaGraphDef(std::vec::Vec<u8>),
    }
}
/// Protocol buffer used for logging messages to the events file.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct LogMessage {
    #[prost(enumeration="log_message::Level", tag="1")]
    pub level: i32,
    #[prost(string, tag="2")]
    pub message: std::string::String,
}
pub mod log_message {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub enum Level {
        Unknown = 0,
        /// Note: The logging level 10 cannot be named DEBUG. Some software
        /// projects compile their C/C++ code with -DDEBUG in debug builds. So the
        /// C++ code generated from this file should not have an identifier named
        /// DEBUG.
        Debugging = 10,
        Info = 20,
        Warn = 30,
        Error = 40,
        Fatal = 50,
    }
}
/// Protocol buffer used for logging session state.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SessionLog {
    #[prost(enumeration="session_log::SessionStatus", tag="1")]
    pub status: i32,
    /// This checkpoint_path contains both the path and filename.
    #[prost(string, tag="2")]
    pub checkpoint_path: std::string::String,
    #[prost(string, tag="3")]
    pub msg: std::string::String,
}
pub mod session_log {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub enum SessionStatus {
        StatusUnspecified = 0,
        Start = 1,
        Stop = 2,
        Checkpoint = 3,
    }
}
/// For logging the metadata output for a single session.run() call.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct TaggedRunMetadata {
    /// Tag name associated with this metadata.
    #[prost(string, tag="1")]
    pub tag: std::string::String,
    /// Byte-encoded version of the `RunMetadata` proto in order to allow lazy
    /// deserialization.
    #[prost(bytes, tag="2")]
    pub run_metadata: std::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct WatchdogConfig {
    #[prost(int64, tag="1")]
    pub timeout_ms: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct RequestedExitCode {
    #[prost(int32, tag="1")]
    pub exit_code: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct WorkerHeartbeatRequest {
    #[prost(enumeration="WorkerShutdownMode", tag="1")]
    pub shutdown_mode: i32,
    #[prost(message, optional, tag="2")]
    pub watchdog_config: ::std::option::Option<WatchdogConfig>,
    #[prost(message, optional, tag="3")]
    pub exit_code: ::std::option::Option<RequestedExitCode>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct WorkerHeartbeatResponse {
    #[prost(enumeration="WorkerHealth", tag="1")]
    pub health_status: i32,
    #[prost(message, repeated, tag="2")]
    pub worker_log: ::std::vec::Vec<Event>,
    #[prost(string, tag="3")]
    pub hostname: std::string::String,
}
// Worker heartbeat messages.  Support for these operations is currently
// internal and expected to change.

/// Current health status of a worker.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum WorkerHealth {
    /// By default a worker is healthy.
    Ok = 0,
    ReceivedShutdownSignal = 1,
    InternalError = 2,
    /// Worker has been instructed to shutdown after a timeout.
    ShuttingDown = 3,
}
/// Indicates the behavior of the worker when an internal error or shutdown
/// signal is received.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum WorkerShutdownMode {
    Default = 0,
    NotConfigured = 1,
    WaitForCoordinator = 2,
    ShutdownAfterTimeout = 3,
}
