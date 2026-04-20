#![allow(clippy::needless_lifetimes)]
#![allow(clippy::new_without_default)]
#![allow(clippy::redundant_pattern_matching)]

pub mod ast;
pub mod parser;
pub mod renderer;
pub mod symbols;

pub use ast::*;
pub use parser::*;
pub use renderer::*;

#[cfg(test)]
mod tests;

#[cfg(feature = "wasm-bindgen")]
pub mod wasm;
