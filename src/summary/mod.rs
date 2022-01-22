#![cfg(feature = "summary")]

//! Types of summaries and events and writers for TensorBoard.
//!
//! The [EventWriter] writes the file that can be recognized by TensorBoard.
//! See the document of [EventWriter] to understand the usage.

mod event;
mod writer;

pub use event::*;
pub use writer::*;
