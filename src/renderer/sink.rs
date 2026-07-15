//! Streaming output sink for renderers (tex2math 2.2).
//!
//! Backends write MathML (or future formats) through [`MathSink`] instead of
//! always allocating an intermediate `String`. [`String`] implements the trait
//! via [`std::fmt::Write`].

use std::fmt::{self, Write};

/// Destination for renderer output.
///
/// Implementors must be able to accept UTF-8 fragments in order. The default
/// methods mirror common buffer operations used by the MathML backend.
pub trait MathSink: Write {
    /// Append a string slice (default: [`Write::write_str`]).
    #[inline]
    fn push_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s)
    }

    /// Append a single character.
    #[inline]
    fn push_char(&mut self, c: char) -> fmt::Result {
        self.write_char(c)
    }
}

impl MathSink for String {}

/// Adapter over any [`fmt::Write`] (e.g. `Vec<u8>` via wrappers, file buffers).
pub struct WriteSink<W: Write> {
    pub inner: W,
}

impl<W: Write> WriteSink<W> {
    pub fn new(inner: W) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> W {
        self.inner
    }
}

impl<W: Write> Write for WriteSink<W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.inner.write_str(s)
    }
}

impl<W: Write> MathSink for WriteSink<W> {}

/// Render MathML into an arbitrary [`MathSink`].
pub fn render_mathml_to(
    node: &crate::ast::MathNode<'_>,
    mode: crate::ast::RenderMode,
    options: &super::mathml::RenderOptions,
    sink: &mut dyn MathSink,
) -> fmt::Result {
    super::mathml::MathMLRenderer::with_options(*options).render_to_sink(node, mode, sink)
}
