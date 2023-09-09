//! Low level synchronous and asynchronous I/O functions.
//!
//! The functions are used internally to work with generic readers and writers.
//! It is not intended for common users, while we encourage using high level API.

#[cfg(feature = "async")]
pub mod r#async;
pub mod sync;
