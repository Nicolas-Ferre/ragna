//! A library for creating easily a GPU-native application.

mod app;
mod instructions;
mod operations;
mod operators;
mod runner;
mod types;
mod wgsl;

pub use app::*;
pub use instructions::*;
pub use operators::*;
pub use types::primitive::*;
pub use types::range::*;
pub use types::*;

/// Transforms a Rust module to a GPU module.
pub use ragna_derive::gpu;

pub use once_cell::sync::Lazy as Glob;
