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
pub use depth::{DEFAULT_MAX_NESTING_DEPTH, MAX_NESTING_DEPTH};
pub use parser::*;
pub use registry::supports_command;
pub use renderer::*;
pub use sema::{analyze, fold_prescripts};

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
    depth::configure_parse(opts.max_depth, opts.unknown_command);

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

/// One-shot LaTeX → MathML conversion (recommended high-level API).
pub fn convert(input: &str, opts: &ConvertOptions) -> Result<String, ParseError> {
    let ast = parse(input, &opts.parse)?;
    depth::configure_render(opts.parse.max_depth);

    let render_opts = RenderOptions {
        mathml_core: opts.mathml_core,
        emit_intent: opts.emit_intent,
    };
    let inner = generate_mathml_with_options(&ast, opts.mode, &render_opts);

    if opts.wrap_math {
        let display_attr = match opts.mode {
            RenderMode::Display => " display=\"block\"",
            RenderMode::Inline => "",
        };
        Ok(format!(
            "<math xmlns=\"http://www.w3.org/1998/Math/MathML\"{}>{}</math>",
            display_attr, inner
        ))
    } else {
        Ok(inner)
    }
}
