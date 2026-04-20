use crate::{generate_mathml, parse_row, RenderMode};
use wasm_bindgen::prelude::*;
use winnow::Parser as WinnowParser;

/// Convert a LaTeX string to MathML for use in JavaScript via WebAssembly.
#[wasm_bindgen]
pub fn convert_to_mathml(latex_input: &str, display_mode: bool) -> Result<String, String> {
    let mode = if display_mode {
        RenderMode::Display
    } else {
        RenderMode::Inline
    };

    let mut cursor = latex_input;

    match parse_row.parse_next(&mut cursor) {
        Ok(ast) => {
            let mathml = generate_mathml(&ast, mode);
            let display_attr = if display_mode {
                " display=\"block\""
            } else {
                ""
            };
            
            Ok(format!(
                "<math xmlns=\"http://www.w3.org/1998/Math/MathML\"{}>{}</math>",
                display_attr, mathml
            ))
        }
        Err(e) => Err(format!("Parse error: {}", e)),
    }
}
