# Tex2math 🚀

[![CI](https://github.com/twn39/tex2math/actions/workflows/ci.yml/badge.svg)](https://github.com/twn39/tex2math/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/tex2math.svg)](https://crates.io/crates/tex2math)
[![Docs](https://img.shields.io/badge/docs-GitHub_Pages-blue)](https://twn39.github.io/tex2math/)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

> A blazing fast, zero-copy Rust library and CLI tool designed to parse LaTeX mathematical formulas and compile them into perfectly structured MathML XML.

Inspired by the robust AST designs of **texmath** and the rich symbol dictionaries of **KaTeX**, `tex2math` is built to be a lightweight, JavaScript-free alternative for generating native MathML directly on the backend (or via WebAssembly).

---

## 💡 Why `tex2math`?

Most lightweight Markdown parsers use fragile regular expressions to handle math, which fails catastrophically on nested structures like `\sqrt{\frac{a}{b}}`. On the other hand, full-blown engines like KaTeX require shipping a massive JavaScript runtime.

`tex2math` sits in the sweet spot:
* **True Parsing**: It builds a complete Abstract Syntax Tree (AST) using parser combinators.
* **Native MathML**: Outputs standard MathML that modern browsers (Chrome 109+, Safari, Firefox) render natively at 60fps without *any* client-side scripts.
* **Zero Overhead**: Written in pure Rust. Memory allocations are kept to an absolute minimum.

## ✨ Key Features

- ⚡️ **Blazing Fast & Zero-Copy**: Built on top of the [`winnow`](https://github.com/winnow-rs/winnow) parser combinator library, the lexer reads directly from string slices.
- 🎓 **Academic-Grade Typography**: 
  - Smart rendering of limits based on contexts (*Inline* vs *Display*).
  - Multi-dimensional matrix alignments (`\begin{align}`).
  - Dynamically stretchy braces (`\left( \right]`).
- 📚 **Huge Symbol Dictionary**: Natively supports over **450+** mathematical symbols, Greek letters, relational operators, and logic arrows—extracted directly from KaTeX source specs.
- 🛡️ **Tolerant Parsing & Error Recovery**: Designed to be indestructible. A missing `}` or unclosed environment will NOT crash the engine, but gracefully emit a red `<merror>` block while salvaging the rest of the formula.
- 🎨 **Advanced Ecosystem Support**: Out-of-the-box support for text modes (`\text{...}`), inline styling (`\mathbf`, `\color`), math accents (`\vec`, `\overbrace`), and complex tensor prescripts (`{}_a^b X`).

---

## 🛠️ Installation

### As a Library (Crate)
Add this to your `Cargo.toml`. 
*Note: The CLI dependencies are fully optional, keeping the library footprint incredibly small (~30KB).*

```toml
[dependencies]
tex2math = "1.0"
winnow = "0.6"
```

### As a CLI Tool
You can install the CLI tool directly using Cargo:

```bash
cargo install tex2math --features cli
```

---

## 🚀 Usage

### 💻 Using the CLI
The CLI is simple and easily integrated into bash pipelines (e.g., passing output directly to Pandoc or HTML generators).

```bash
# Direct argument evaluation
tex2math "\frac{-b \pm \sqrt{b^2 - 4ac}}{2a}"

# Display mode evaluation (forces \sum limits to render top/bottom)
tex2math --display "\sum_{i=1}^n x_i"

# Pipe via stdin
echo "\int_0^\infty f(x) \, dx" | tex2math
```

### 🦀 Using the Library

```rust
use tex2math::{parse_row, MathMLRenderer, MathRenderer, RenderMode};
use winnow::Parser; 

fn main() {
    let latex_input = "\\underbrace{a+b}_{=X} \\times \\mathbf{R}";
    
    // The parser requires a mutable reference to the string slice (cursor)
    let mut input = latex_input;

    // 1. Parse the LaTeX string into an AST
    match parse_row.parse_next(&mut input) {
        Ok(ast) => {
            // 2. Initialize the backend renderer
            let renderer = MathMLRenderer::new();
            
            // 3. Generate MathML (Choose Inline or Display mode)
            let mathml = renderer.render(&ast, RenderMode::Display);
            println!("<math>{}</math>", mathml);
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
        }
    }
}
```

---

## 🏗️ Architecture

1. **Parser Combinators (`winnow`)**: Transforms string slices into a highly structured `MathNode` enum.
2. **AST Folding Pass**: Post-processes the AST to handle complex semantic overrides, such as converting empty-based scripts into true MathML `<mmultiscripts>` (tensor prescripts).
3. **Pluggable Renderers**: The abstract `MathRenderer` trait decouples AST generation from final presentation mapping, allowing future expansions (e.g., Typst or HTML/CSS backends).

## 🧪 Development & Testing

This project strictly follows **Test-Driven Development (TDD)**. It includes over 50 rigorous regression tests inspired by `KaTeX` and `texmath` to ensure edge cases (e.g., nested roots, empty environments, infinite recursions) are perfectly handled.

```bash
# Run tests without verbose trace outputs
cargo test --no-default-features
```

If you need to debug a failing parser, enable the `debug-trace` feature in `Cargo.toml` to see a colorful, interactive trace of the parser execution tree.

---

## 📄 License

This project is licensed under the **MIT License**.
