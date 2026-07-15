use crate::ast::*;

pub mod mathml;
pub mod sink;

pub use mathml::{generate_mathml, generate_mathml_with_options, MathMLRenderer, RenderOptions};
pub use sink::{render_mathml_to, MathSink, WriteSink};

/// An abstract backend interface for rendering a `MathNode` AST into an output format.
pub trait MathRenderer {
    /// Renders into an existing buffer (avoids intermediate allocations at the call site).
    fn render_into(
        &self,
        node: &MathNode<'_>,
        mode: RenderMode,
        buf: &mut String,
    ) -> std::fmt::Result;

    /// Convenience wrapper allocating a new `String`.
    fn render(&self, node: &MathNode<'_>, mode: RenderMode) -> String {
        let mut buf = String::with_capacity(256);
        self.render_into(node, mode, &mut buf)
            .expect("Formatting to String should never fail");
        buf
    }
}
