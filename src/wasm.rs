use crate::{generate_mathml, parse_math, RenderMode, ParseError};
use wasm_bindgen::prelude::*;
use winnow::Parser as WinnowParser;


/// Convert a LaTeX string to MathML for use in JavaScript via WebAssembly.
#[wasm_bindgen]
pub fn convert_to_mathml(latex_input: &str, display_mode: bool) -> Result<String, JsValue> {
    let mode = if display_mode {
        RenderMode::Display
    } else {
        RenderMode::Inline
    };

    let mut cursor = latex_input;
    let initial_len = cursor.len();

    match parse_math.parse_next(&mut cursor) {
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
        Err(e) => {
            #[cfg(feature = "serde")]
            {
                let err_obj = ParseError {
                    message: format!("Parse error: {}", e),
                    offset: initial_len - cursor.len(),
                };
                Err(serde_wasm_bindgen::to_value(&err_obj).unwrap_or_else(|_| JsValue::from_str(&err_obj.message)))
            }
            #[cfg(not(feature = "serde"))]
            {
                Err(JsValue::from_str(&format!("Parse error: {}", e)))
            }
        }
    }
}

#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
pub struct WasmBatchInput {
    pub latex: String,
    pub display: bool,
}

#[cfg(feature = "serde")]
#[wasm_bindgen]
pub fn parse_to_ast(latex_input: &str) -> Result<JsValue, JsValue> {
    let mut cursor = latex_input;
    let initial_len = cursor.len();

    match parse_math.parse_next(&mut cursor) {
        Ok(ast) => {
            Ok(serde_wasm_bindgen::to_value(&ast).unwrap())
        }
        Err(e) => {
            let err_obj = ParseError {
                message: format!("Parse error: {}", e),
                offset: initial_len - cursor.len(),
            };
            Err(serde_wasm_bindgen::to_value(&err_obj).unwrap_or_else(|_| JsValue::from_str(&err_obj.message)))
        }
    }
}

#[cfg(feature = "serde")]
#[derive(serde::Serialize)]
pub struct WasmBatchResult {
    pub success: bool,
    pub result: Option<String>,
    pub error: Option<ParseError>,
}

#[cfg(feature = "serde")]
#[wasm_bindgen]
pub fn convert_batch(inputs_js: &JsValue) -> Result<JsValue, JsValue> {
    let inputs: Vec<WasmBatchInput> = serde_wasm_bindgen::from_value(inputs_js.clone())
        .map_err(|e| JsValue::from_str(&format!("Invalid batch input: {}", e)))?;

    let mut results = Vec::with_capacity(inputs.len());

    for input in inputs {
        let mode = if input.display { RenderMode::Display } else { RenderMode::Inline };
        let mut cursor = input.latex.as_str();
        let initial_len = cursor.len();

        match parse_math.parse_next(&mut cursor) {
            Ok(ast) => {
                let mathml = generate_mathml(&ast, mode);
                let display_attr = if input.display { " display=\"block\"" } else { "" };
                let xml = format!(
                    "<math xmlns=\"http://www.w3.org/1998/Math/MathML\"{}>{}</math>",
                    display_attr, mathml
                );
                results.push(WasmBatchResult {
                    success: true,
                    result: Some(xml),
                    error: None,
                });
            }
            Err(e) => {
                results.push(WasmBatchResult {
                    success: false,
                    result: None,
                    error: Some(ParseError {
                        message: format!("Parse error: {}", e),
                        offset: initial_len - cursor.len(),
                    }),
                });
            }
        }
    }

    Ok(serde_wasm_bindgen::to_value(&results).unwrap())
}

