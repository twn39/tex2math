// Lifetimes on parser combinators are intentional for readability next to `&mut &str` cursors.
#![allow(clippy::needless_lifetimes)]

//! # tex2math 2.0
//!
//! Blazing-fast LaTeX → MathML with a borrowed AST, structured errors, and a
//! one-shot [`convert`] API.
//!
//! ```rust
//! use tex2math::{convert, ConvertOptions, parse, ParseOptions};
//!
//! let mathml = convert(r"\frac{1}{2}", &ConvertOptions::default()).unwrap();
//! assert!(mathml.contains("<mfrac>"));
//!
//! let ast = parse(r"x^2", &ParseOptions::default()).unwrap();
//! let owned = ast.into_owned(); // detaches from input
//! let _ = owned;
//! ```

pub mod ast;
pub(crate) mod depth;
pub mod parser;
pub mod registry;
pub mod renderer;
pub mod sema;
pub mod symbols;

pub use ast::*;
pub use depth::{ParseCtx, ParseCtxGuard, DEFAULT_MAX_NESTING_DEPTH, MAX_NESTING_DEPTH};
pub use parser::*;
pub use registry::{command_spec, registered_command_names, supports_command, CommandSpec};
pub use renderer::*;
pub use sema::{analyze, fold_choose, fold_prescripts, fold_prime_node, fold_row};

#[cfg(test)]
mod tests;

#[cfg(feature = "wasm-bindgen")]
pub mod wasm;

use winnow::Parser;

/// Parse LaTeX into a borrowed AST (tex2math 2.0 primary entry).
///
/// The returned [`MathNode`] may borrow from `input`. Call
/// [`MathNode::into_owned`] if it must outlive `input`.
pub fn parse<'s>(input: &'s str, opts: &ParseOptions) -> Result<MathNode<'s>, ParseError> {
    // Guard restores prior TLS so nested `parse` on the same thread is safe.
    let _ctx = depth::ParseCtx::from_parse_options(opts).install();

    let mut cursor = input;
    let initial_len = cursor.len();

    match parse_math.parse_next(&mut cursor) {
        Ok(ast) => {
            if depth::take_parse_depth_exceeded() {
                return Err(ParseError::at_offset(
                    ParseErrorKind::NestingLimit,
                    initial_len.saturating_sub(cursor.len()),
                    depth::nesting_depth_error_message(),
                ));
            }

            // Semantic folding / normalize (prescripts, nested rows, …).
            let ast = sema::analyze(ast);

            let rest = cursor.trim_start();
            if !rest.is_empty() {
                match opts.trailing {
                    TrailingPolicy::Ignore => Ok(ast),
                    TrailingPolicy::Error => {
                        let preview: String = rest.chars().take(32).collect();
                        let suffix = if rest.chars().count() > 32 { "…" } else { "" };
                        let start = initial_len - rest.len();
                        Err(ParseError::new(
                            ParseErrorKind::UnexpectedTrailing,
                            start..initial_len,
                            format!(
                                "Unexpected trailing input starting with '{}{}'",
                                preview, suffix
                            ),
                        ))
                    }
                }
            } else {
                Ok(ast)
            }
        }
        Err(e) => {
            let offset = initial_len - cursor.len();
            if depth::take_parse_depth_exceeded() {
                Err(ParseError::at_offset(
                    ParseErrorKind::NestingLimit,
                    offset,
                    depth::nesting_depth_error_message(),
                ))
            } else {
                let msg = e.to_string();
                Err(ParseError::at_offset(
                    ParseErrorKind::Syntax,
                    offset,
                    if msg.is_empty() {
                        "Parse error".to_string()
                    } else {
                        msg
                    },
                ))
            }
        }
    }
}

/// Parse with default [`ParseOptions`].
#[inline]
pub fn parse_latex(input: &str) -> Result<MathNode<'_>, ParseError> {
    parse(input, &ParseOptions::default())
}

/// One-shot LaTeX → MathML conversion directly streaming into a [`MathSink`].
///
/// This avoids allocating an intermediate [`String`] and allows callers to reuse existing
/// buffers across multiple formula conversions.
pub fn convert_to(
    input: &str,
    opts: &ConvertOptions,
    out: &mut dyn MathSink,
) -> Result<(), ParseError> {
    let ast = parse(input, &opts.parse)?;

    let render_opts = RenderOptions {
        mathml_core: opts.mathml_core,
        emit_intent: opts.emit_intent,
    };

    if opts.wrap_math {
        match opts.mode {
            RenderMode::Display => {
                out.push_str(
                    "<math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"block\">",
                )
                .map_err(|e| ParseError::at_offset(ParseErrorKind::Other, 0, e.to_string()))?;
            }
            RenderMode::Inline => {
                out.push_str("<math xmlns=\"http://www.w3.org/1998/Math/MathML\">")
                    .map_err(|e| ParseError::at_offset(ParseErrorKind::Other, 0, e.to_string()))?;
            }
        }
    }

    render_mathml_to(&ast, opts.mode, &render_opts, out)
        .map_err(|e| ParseError::at_offset(ParseErrorKind::Other, 0, e.to_string()))?;

    if opts.wrap_math {
        out.push_str("</math>")
            .map_err(|e| ParseError::at_offset(ParseErrorKind::Other, 0, e.to_string()))?;
    }

    Ok(())
}

#[inline]
fn estimate_mathml_capacity(input: &str, wrap_math: bool) -> usize {
    let len = input.len();
    let bytes = input.as_bytes();
    let slash_count = memchr::memchr_iter(b'\\', bytes).count();
    let expansion_factor = if slash_count > len / 10 { 6 } else { 4 };
    let base = len.saturating_mul(expansion_factor);
    if wrap_math {
        base.saturating_add(128)
    } else {
        base.saturating_add(64)
    }
}

/// One-shot LaTeX → MathML conversion (recommended high-level API).
pub fn convert(input: &str, opts: &ConvertOptions) -> Result<String, ParseError> {
    let estimated_capacity = estimate_mathml_capacity(input, opts.wrap_math);
    let mut buf = String::with_capacity(estimated_capacity);
    convert_to(input, opts, &mut buf)?;
    Ok(buf)
}
