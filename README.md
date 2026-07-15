<div align="center">

# Tex2math 🚀

[![CI](https://github.com/twn39/tex2math/actions/workflows/ci.yml/badge.svg)](https://github.com/twn39/tex2math/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/tex2math.svg)](https://crates.io/crates/tex2math)
[![Docs](https://img.shields.io/badge/docs-GitHub_Pages-blue)](https://twn39.github.io/tex2math/)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)
![License](https://img.shields.io/badge/license-LGPL--3.0-blue.svg)

> A blazing fast Rust library and CLI tool that parses LaTeX math and emits structured MathML — zero JS, backend- or WASM-friendly.

</div>

Inspired by the AST designs of **texmath** and the symbol dictionaries of **KaTeX**, `tex2math` is a lightweight alternative for generating native MathML without shipping a browser math engine.

---

## 💡 Why `tex2math`?

Most lightweight Markdown parsers use fragile regular expressions for math and fail on nested structures like `\sqrt{\frac{a}{b}}`. Full engines like KaTeX require a large JavaScript runtime.

`tex2math` sits in the middle:

* **True parsing** — full AST via [`winnow`](https://github.com/winnow-rs/winnow) combinators.
* **Native MathML** — browsers (Chrome 109+, Safari, Firefox) render MathML without client scripts.
* **Low overhead** — pure Rust, slice-oriented parse, borrowed AST leaves, nesting-depth guards, heap-iterative render.

---

## ✨ Key Features

- ⚡️ **Slice-based parsing** — walks input as `&str` slices; AST leaves are `Cow<'s, str>` (borrow input or static tables). Call `into_owned()` when the tree must outlive the source.
- 🎓 **Academic-grade layout** — display vs inline limits, multi-column `align` / `array`, stretchy `\left`…`\right`.
- 📚 **Large symbol table** — 450+ symbols, Greek, relations, arrows (KaTeX-oriented).
- 🛡️ **Tolerant recovery** — missing `}` / unclosed environments emit `<merror>` and salvage the rest when possible.
- 🎨 **Styles & accents** — `\text`, `\mathbf`, `\color`, accents, stretch ops, cancel, tensor-style prescripts via a post-parse semantic pass.
- 🔌 **Pluggable emission** — `MathRenderer` + `MathSink` (stream into any `fmt::Write`); MathML is the default backend.
- 🧩 **Data-driven commands** — fonts, accents, arrows, stretch ops live in `registry` (single source of names).

---

## 🛠️ Installation

### Library

```toml
[dependencies]
tex2math = "2"
```

### CLI

```bash
cargo install tex2math --features cli
```

---

## 🚀 Usage

### CLI

```bash
tex2math "\frac{-b \pm \sqrt{b^2 - 4ac}}{2a}"
tex2math --display "\sum_{i=1}^n x_i"
echo "\int_0^\infty f(x) \, dx" | tex2math
tex2math --no-wrap --unknown-error '\foo{x}'
```

| Flag | Effect |
|------|--------|
| `--display` | Display mode (`display="block"`, large-op limits) |
| `--no-wrap` | Omit outer `<math>` wrapper |
| `--max-depth N` | Parse nesting cap (default 64) |
| `--allow-trailing` | Ignore trailing junk after a successful parse |
| `--unknown-error` | Unknown `\cmd` → `<merror>` instead of identifier fallback |

### Library (2.x)

```rust
use tex2math::{convert, parse, ConvertOptions, ParseOptions, UnknownCommandPolicy};

fn main() {
    // One-shot conversion (recommended)
    let mathml = convert(
        r"\underbrace{a+b}_{=X} \times \mathbf{R}",
        &ConvertOptions::display(),
    )
    .expect("convert");
    println!("{mathml}");

    // Borrowed AST (zero-copy leaves where possible)
    let input = r"x^2 + y^2";
    let ast = parse(input, &ParseOptions::default()).unwrap();
    let _owned = ast.into_owned(); // detach from `input`

    // Unknown commands as errors (renders <merror>)
    let opts = ConvertOptions {
        parse: ParseOptions {
            unknown_command: UnknownCommandPolicy::Error,
            ..Default::default()
        },
        wrap_math: false,
        ..Default::default()
    };
    let _ = convert(r"\notreal{x}", &opts);
}
```

`ConvertOptions` defaults: wrap in `<math>`, **MathML Core–friendly** emission (`mathml_core: true`), no experimental `intent` attributes. See [docs/RENDER_OPTIONS.md](docs/RENDER_OPTIONS.md).

---

## 🏗️ Architecture

```text
LaTeX &str → parser (winnow) → MathNode AST → sema folds → MathML renderer → String / MathSink
                  ↑                    ↑
            registry/symbols      DepthGuard / ParseError
```

1. **Parser** — combinators in `parser/` build a syntactic AST.
2. **Registry** — command name → payload tables (single source for table-driven cmds).
3. **Semantic pass** — prescripts, row normalize (`sema::analyze`).
4. **Heap-iterative MathML** — expand by AST family (`tokens` / `structure` / `style` / `scripts` / `environment`).

Migration notes: [docs/MIGRATION-2.0.md](docs/MIGRATION-2.0.md).

---

## 🧪 Development & Testing

TDD with unit tests under `src/tests/` and integration tests under `tests/`.

```bash
cargo test
cargo test --all-features
cargo bench --bench math_parser_bench   # criterion (parse / convert / deep trees)
```

Enable `debug-trace` for a colorful winnow parse trace when debugging.

---

## 📄 License

LGPL-3.0-only — see crate metadata.
