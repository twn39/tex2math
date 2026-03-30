# Tex2math

![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

`tex2math` is a blazing fast, zero-copy Rust library designed to parse LaTeX mathematical formulas and convert them into standard MathML XML. 

Instead of relying on brittle regular expressions, `tex2math` uses a robust three-step compiler architecture:
1. **Parser**: Powered by the [`winnow`](https://github.com/winnow-rs/winnow) parser combinator library for zero-allocation token extraction.
2. **AST**: Builds a strong Abstract Syntax Tree (`MathNode`) to handle LaTeX's implicit scopes and nested structures.
3. **Generator**: Emits clean, standard MathML ready to be rendered natively by modern web browsers.

## Features

- **Zero-Copy Parsing**: Extracts tokens directly from the input string slices without unnecessary heap allocations.
- **Robust AST**: Handles deep nesting and implicit scopes gracefully.
- **Visual Debugging**: Built-in trace features (via `winnow`'s debug mode) to visually print parser call trees during development.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
tex2math = { path = "path/to/tex2math" } # Update this when publishing to crates.io
winnow = "0.6"
```

## Usage Example

```rust
use tex2math::{parse_row, generate_mathml};
use winnow::Parser; // Required to use .parse_next()

fn main() {
    let latex_input = "\\frac{a}{b} 42";
    
    // The parser requires a mutable reference to the string slice (cursor)
    let mut input = latex_input;

    // 1. Parse the LaTeX string into an AST
    match parse_row.parse_next(&mut input) {
        Ok(ast) => {
            // 2. Generate MathML from the AST
            let mathml = format!("<math>{}</math>", generate_mathml(&ast));
            println!("{}", mathml);
            // Output: <math><mrow><mfrac><mi>a</mi><mi>b</mi></mfrac><mn>42</mn></mrow></math>
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
        }
    }
}
```

## Architecture

1. **`lib.rs`**: Contains the `MathNode` AST enum, all `winnow` parser combinators (`parse_fraction`, `parse_number`, etc.), and the `generate_mathml` recursive generator.
2. **`tests.rs`**: Contains standard Rust unit tests. The project strictly follows Test-Driven Development (TDD).

## Development & Testing

To run the test suite:

```bash
cargo test
```

If you need to debug a failing parser, enable the `debug` feature in `Cargo.toml` to see a colorful, interactive trace of the parser execution.

## License

This project is licensed under the MIT License.
