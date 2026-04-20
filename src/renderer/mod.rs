use crate::ast::*;

pub mod mathml;
pub use mathml::{MathMLRenderer, generate_mathml};

/// An abstract backend interface for rendering a `MathNode` abstract syntax tree into an output format.
///
/// All backend renderers must implement this trait.
pub trait MathRenderer {
    /// Renders the `MathNode` into the provided string buffer to avoid unnecessary memory allocations.
    fn render_into(&self, node: &MathNode, mode: RenderMode, buf: &mut String) -> std::fmt::Result;

    /// Renders the `MathNode` into a new `String`.
    /// This provides backward compatibility and ease of use.
    fn render(&self, node: &MathNode, mode: RenderMode) -> String {
        let mut buf = String::with_capacity(256);
        self.render_into(node, mode, &mut buf).expect("Formatting to String should never fail");
        buf
    }
}
