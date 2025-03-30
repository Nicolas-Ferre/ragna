//! A library for creating easily a GPU-native application.

mod app;
mod context;
mod glob;
mod instructions;
mod operations;
mod operators;
mod runner;
mod testing;
mod types;
mod wgsl;

pub use app::*;
pub use context::*;
pub use glob::*;
pub use instructions::*;
pub use operators::*;
pub use testing::*;
pub use types::array::*;
pub use types::primitive::*;
pub use types::range::*;
pub use types::vectors::*;
pub use types::*;

/// Transforms a Rust module to a GPU module.
pub use ragna_derive::gpu;
