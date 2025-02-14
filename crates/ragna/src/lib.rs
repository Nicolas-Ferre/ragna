//! A library for creating easily a GPU-native application.

mod app;
mod operations;
mod operators;
mod runner;
mod types;
mod values;
mod wgsl;

pub use app::*;
pub use operators::*;
pub use types::*;
pub use values::*;

/// Transforms a Rust module to a GPU module.
pub use ragna_derive::gpu;
