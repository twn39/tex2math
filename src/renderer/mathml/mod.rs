use crate::ast::*;
use crate::renderer::sink::MathSink;
use crate::renderer::MathRenderer;
use std::fmt;

mod basic;
mod iter;

/// Options controlling MathML emission (tex2math 2.0+).
///
/// Defaults: `mathml_core = false`, `emit_intent = false` so `generate_mathml`
/// keeps historical tags (e.g. `<menclose>`). Use `ConvertOptions { mathml_core: true, .. }`
/// for Core-friendly output via [`crate::convert`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RenderOptions {
    /// Prefer MathML Core-friendly output (omit non-Core elements like `menclose`).
    pub mathml_core: bool,
    /// Emit MathML 4 `intent` attributes where known (experimental).
    pub emit_intent: bool,
}

/// The standard MathML rendering backend (heap-iterative emission).
pub struct MathMLRenderer {
    pub options: RenderOptions,
}

impl MathMLRenderer {
    pub fn new() -> Self {
        Self {
            options: RenderOptions::default(),
        }
    }

    pub fn with_options(options: RenderOptions) -> Self {
        Self { options }
    }

    /// Stream MathML into any [`MathSink`] (stack-safe for deep trees).
    pub fn render_to_sink(
        &self,
        node: &MathNode<'_>,
        mode: RenderMode,
        sink: &mut dyn MathSink,
    ) -> fmt::Result {
        let mut ctx = RenderCtx {
            out: sink,
            mode,
            options: self.options,
        };
        self.render_node_iter(node, &mut ctx)
    }
}

impl Default for MathMLRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl MathRenderer for MathMLRenderer {
    fn render_into(&self, node: &MathNode<'_>, mode: RenderMode, buf: &mut String) -> fmt::Result {
        self.render_to_sink(node, mode, buf)
    }
}

/// Shared rendering context for the MathML backend.
pub(super) struct RenderCtx<'a> {
    pub out: &'a mut dyn MathSink,
    pub mode: RenderMode,
    pub options: RenderOptions,
}

/// Generate MathML with default render options.
pub fn generate_mathml(node: &MathNode<'_>, mode: RenderMode) -> String {
    MathMLRenderer::new().render(node, mode)
}

/// Generate MathML with explicit [`RenderOptions`].
pub fn generate_mathml_with_options(
    node: &MathNode<'_>,
    mode: RenderMode,
    options: &RenderOptions,
) -> String {
    MathMLRenderer::with_options(*options).render(node, mode)
}
