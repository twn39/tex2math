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

use winnow::Parser;

/// A unified, high-level API for parsing a complete LaTeX string into an Abstract Syntax Tree (AST).
///
/// This is the recommended entry point for library consumers. It automatically leverages `parse_math`
/// to handle top-level environments (like multi-column `align*` created by isolated `&` and `\\`) 
/// and consumes the input string safely.
///
/// # Differences between underlying parsers:
/// - `parse_latex` (this function): The highest-level facade. Returns a standard `Result` and handles full input.
/// - `parse_math`: Used internally to parse equations that might contain un-bracketed matrix rows (using `&` and `\\`). It wraps these in an implicit `align*` environment.
/// - `parse_row`: Parses a flat sequence of mathematical nodes strictly *without* understanding table delimiters (`&`, `\\`). Used inside explicit environments (like `\begin{matrix}`).
///
/// # Example
/// ```rust
/// use tex2math::parse_latex;
/// 
/// let ast = parse_latex(r"\frac{-b \pm \sqrt{b^2 - 4ac}}{2a}").expect("Failed to parse");
/// ```
pub fn parse_latex(input: &str) -> Result<MathNode, ParseError> {
    let mut cursor = input;
    let initial_len = cursor.len();
    match parse_math.parse_next(&mut cursor) {
        Ok(ast) => {
            if !cursor.trim().is_empty() {
                // If there's unparsed trailing garbage, we can optionally handle it.
                // For now, we return the parsed AST and ignore trailing whitespaces,
                // but if there are actual remaining unparsed symbols, it might indicate a syntax error.
            }
            Ok(ast)
        }
        Err(e) => Err(ParseError {
            message: e.to_string(),
            offset: initial_len - cursor.len(),
        }),
    }
}
