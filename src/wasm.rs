//! WebAssembly / JavaScript bindings (feature `wasm`).
//!
//! Errors are always structured objects with `kind`, `message`, `spanStart`,
//! and `spanEnd` (byte offsets) so host code can highlight source ranges.

use crate::{
    convert, parse, supports_command, ConvertOptions, ParseError, ParseErrorKind, ParseOptions,
    RenderMode, TrailingPolicy, UnknownCommandPolicy,
};
use wasm_bindgen::prelude::*;

/// Stable JSON-shaped error for JS consumers.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct WasmParseError {
    kind: &'static str,
    message: String,
    span_start: usize,
    span_end: usize,
}

impl From<ParseError> for WasmParseError {
    fn from(err: ParseError) -> Self {
        Self {
            kind: match err.kind {
                ParseErrorKind::NestingLimit => "nesting_limit",
                ParseErrorKind::UnexpectedTrailing => "unexpected_trailing",
                ParseErrorKind::Syntax => "syntax",
                ParseErrorKind::Other => "other",
            },
            message: err.message,
            span_start: err.span.start,
            span_end: err.span.end,
        }
    }
}

fn err_to_js(err: ParseError) -> JsValue {
    let w = WasmParseError::from(err);
    #[cfg(feature = "serde")]
    {
        serde_wasm_bindgen::to_value(&w).unwrap_or_else(|_| JsValue::from_str(&w.message))
    }
    #[cfg(not(feature = "serde"))]
    {
        JsValue::from_str(&format!(
            "{} at {}..{}: {}",
            w.kind, w.span_start, w.span_end, w.message
        ))
    }
}

/// Optional convert flags from JS (`convert_to_mathml_with_options`).
#[cfg(feature = "serde")]
#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WasmConvertOptions {
    /// Display mode (`display="block"`). Default false.
    pub display: Option<bool>,
    /// Wrap in `<math xmlns=…>`. Default true.
    pub wrap_math: Option<bool>,
    /// MathML Core–friendly emission. Default true.
    pub mathml_core: Option<bool>,
    /// Emit experimental `intent` attributes. Default false.
    pub emit_intent: Option<bool>,
    /// Max parse nesting depth. Default 64.
    pub max_depth: Option<u32>,
    /// Unknown `\`-commands → error nodes. Default false (identifier fallback).
    pub unknown_error: Option<bool>,
    /// Ignore trailing junk after a successful parse. Default false.
    pub allow_trailing: Option<bool>,
}

#[cfg(feature = "serde")]
impl WasmConvertOptions {
    fn into_convert_options(self) -> ConvertOptions {
        let mut opts = ConvertOptions::default();
        if self.display.unwrap_or(false) {
            opts.mode = RenderMode::Display;
        }
        if let Some(w) = self.wrap_math {
            opts.wrap_math = w;
        }
        if let Some(c) = self.mathml_core {
            opts.mathml_core = c;
        }
        if let Some(i) = self.emit_intent {
            opts.emit_intent = i;
        }
        if let Some(d) = self.max_depth {
            opts.parse.max_depth = d;
        }
        if self.unknown_error.unwrap_or(false) {
            opts.parse.unknown_command = UnknownCommandPolicy::Error;
        }
        if self.allow_trailing.unwrap_or(false) {
            opts.parse.trailing = TrailingPolicy::Ignore;
        }
        opts
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

/// Convert with a JS options object (all fields optional).
///
/// ```js
/// convert_to_mathml_with_options("\\frac{1}{2}", {
///   display: true,
///   wrapMath: true,
///   mathmlCore: true,
///   emitIntent: false,
///   maxDepth: 64,
///   unknownError: false,
///   allowTrailing: false,
/// });
/// ```
#[cfg(feature = "serde")]
#[wasm_bindgen]
pub fn convert_to_mathml_with_options(
    latex_input: &str,
    options_js: &JsValue,
) -> Result<String, JsValue> {
    let wopts: WasmConvertOptions = if options_js.is_null() || options_js.is_undefined() {
        WasmConvertOptions::default()
    } else {
        serde_wasm_bindgen::from_value(options_js.clone())
            .map_err(|e| JsValue::from_str(&format!("Invalid convert options: {e}")))?
    };
    convert(latex_input, &wopts.into_convert_options()).map_err(err_to_js)
}

/// Whether a bare or `\cmd` name is known to the converter.
#[wasm_bindgen]
pub fn wasm_supports_command(cmd: &str) -> bool {
    supports_command(cmd)
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
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize AST: {e}")))
}

#[cfg(feature = "serde")]
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WasmBatchResult {
    pub success: bool,
    pub result: Option<String>,
    pub error: Option<WasmParseError>,
}

#[cfg(feature = "serde")]
#[wasm_bindgen]
pub fn convert_batch(inputs_js: &JsValue) -> Result<JsValue, JsValue> {
    let inputs: Vec<WasmBatchInput> = serde_wasm_bindgen::from_value(inputs_js.clone())
        .map_err(|e| JsValue::from_str(&format!("Invalid batch input: {e}")))?;

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
                error: Some(WasmParseError::from(e)),
            }),
        }
    }

    serde_wasm_bindgen::to_value(&results)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize batch results: {e}")))
}

// Keep kind in the type graph for docs / non-serde builds.
#[allow(dead_code)]
fn _kind_export() -> ParseErrorKind {
    ParseErrorKind::Other
}
