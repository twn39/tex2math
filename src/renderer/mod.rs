use crate::ast::*;

pub mod mathml;
pub use mathml::{generate_mathml, MathMLRenderer};

/// An abstract backend interface for rendering a `MathNode` abstract syntax tree into an output format.
///
/// All backend renderers must implement this trait.
pub trait MathRenderer {
    fn render(&self, node: &MathNode, mode: RenderMode) -> String;
}
