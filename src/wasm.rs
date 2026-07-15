use crate::{convert, parse, ConvertOptions, ParseError, ParseErrorKind, ParseOptions, RenderMode};
use wasm_bindgen::prelude::*;

fn err_to_js(err: ParseError) -> JsValue {
    #[cfg(feature = "serde")]
    {
        serde_wasm_bindgen::to_value(&err).unwrap_or_else(|_| JsValue::from_str(&err.message))
    }
    #[cfg(not(feature = "serde"))]
    {
        JsValue::from_str(&err.message)
    }
}

/// Convert a LaTeX string to MathML for use in JavaScript via WebAssembly.
#[wasm_bindgen]
pub fn convert_to_mathml(latex_input: &str, display_mode: bool) -> Result<String, JsValue> {
    let opts = ConvertOptions {
        mode: if display_mode {
            RenderMode::Display
        } else {
            RenderMode::Inline
        },
        wrap_math: true,
        ..Default::default()
    };
    convert(latex_input, &opts).map_err(err_to_js)
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
    // Serialize an owned tree so it does not borrow the input buffer.
    let ast = parse(latex_input, &ParseOptions::default())
        .map_err(err_to_js)?
        .into_owned();
    serde_wasm_bindgen::to_value(&ast)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize AST: {}", e)))
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
        let opts = ConvertOptions {
            mode: if input.display {
                RenderMode::Display
            } else {
                RenderMode::Inline
            },
            wrap_math: true,
            ..Default::default()
        };
        match convert(&input.latex, &opts) {
            Ok(xml) => results.push(WasmBatchResult {
                success: true,
                result: Some(xml),
                error: None,
            }),
            Err(e) => results.push(WasmBatchResult {
                success: false,
                result: None,
                error: Some(e),
            }),
        }
    }

    serde_wasm_bindgen::to_value(&results)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize batch results: {}", e)))
}

// Silence unused-import when serde is off for kind (used in docs).
#[allow(dead_code)]
fn _kind_export() -> ParseErrorKind {
    ParseErrorKind::Other
}
