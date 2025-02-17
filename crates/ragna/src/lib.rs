//! A library for creating easily a GPU-native application.

mod app;
mod operations;
mod operators;
mod runner;
mod types;
mod wgsl;

pub use app::*;
pub use operators::*;
pub use types::*;

/// Transforms a Rust module to a GPU module.
pub use ragna_derive::gpu;

// TODO: add compile-time check for assignment to constants
// TODO: adapt compilation tests
