# Migrating to tex2math 2.x

tex2math 2.0+ is a **breaking** line focused on a borrowed AST, structured
diagnostics, and a one-shot conversion API aligned with MathML Core usage.

## Quick map

| 1.x | 2.x |
|-----|-----|
| `parse_latex(input) -> Result<MathNode, ParseError>` | Same name, but `MathNode<'_>` **borrows** `input` |
| `MathNode` owned `String` leaves | `MathNode<'s>` with `Cow<'s, str>` leaves |
| `ParseError { message, offset }` | `ParseError { kind, span, message }` (+ `offset()` helper) |
| `generate_mathml` / `MathMLRenderer` | Still available; prefer `convert` |
| *(none)* | `parse`, `convert`, `ParseOptions`, `ConvertOptions` |
| Silent trailing garbage | Default: `TrailingPolicy::Error` |

## Recommended call sites

```rust
use tex2math::{convert, ConvertOptions};

let xml = convert(r"\frac{1}{2}", &ConvertOptions::default())?;
```

```rust
use tex2math::{parse, ParseOptions};

let input = r"x^2";
let ast = parse(input, &ParseOptions::default())?;
// Keep `input` alive while using `ast`, or:
let owned = ast.into_owned();
```

## Options

### Parse (`ParseOptions`)

| Field | Default | Meaning |
|-------|---------|---------|
| `max_depth` | `64` | Nesting cap (stack-safe parse) |
| `trailing` | `Error` | Trailing junk → error, or `Ignore` |
| `recovery` | `Tolerant` | Prefer salvage / `MathNode::Error` |
| `unknown_command` | `Identifier` | Unknown `\cmd` → `Identifier("\\cmd")`; set `Error` for `<merror>` |

### Convert (`ConvertOptions`)

| Field | Default | Meaning |
|-------|---------|---------|
| `parse` | (see above) | Nested parse options |
| `mode` | `Inline` | `Display` sets `display="block"` + large-op limits |
| `wrap_math` | `true` | Wrap in `<math xmlns=...>` |
| `mathml_core` | `true` on `convert` | Core-friendly emission (see coverage table) |
| `emit_intent` | `false` | Experimental MathML 4 `intent` (see coverage table) |

## CLI

```bash
tex2math --display '\sum_{i=1}^n i'
tex2math --max-depth 128 --allow-trailing '...'
tex2math --no-wrap 'x+y'
tex2math --unknown-error '\notacommand{x}'   # unknown cmds → merror
```

## WASM

`convert_to_mathml` / `convert_batch` go through the 2.x `convert` path.
`parse_to_ast` returns an **owned** serializable tree (`into_owned`).

## Version timeline

### 2.0

- Borrowed AST (`Cow` leaves), structured `ParseError`, `parse` / `convert`.
- `TrailingPolicy`, nesting depth guards.

### 2.1

- **Semantic pass** (`sema::analyze`): prescript fold + recursive normalize after parse.
- **Command registry** (`registry`): fonts, accents, sized delims, cancel, arrows, stretch ops; query via `supports_command`.
- **`mathml_core`**: when true (default on `convert`), `\cancel`/`\boxed` avoid non-Core `<menclose>`.
- **`emit_intent`**: experimental MathML 4 intents on selected constructs.

### 2.2

- **`MathSink`**: stream MathML into any `fmt::Write` buffer (`render_mathml_to`, `WriteSink`).
- **Heap-iterative renderer**: deep ASTs no longer grow the OS call stack during MathML emission.
- Renderer expand split by AST family (`tokens` / `structure` / `style` / `scripts` / `environment`).

### Current hardening (post-2.2)

- **Registry single-source**: table-driven command *names* live only in `registry`; `parse_command` dispatches via lookups.
- **`UnknownCommandPolicy`**: configurable unknown-control-sequence behavior.
- **Coverage docs**: [`RENDER_OPTIONS.md`](./RENDER_OPTIONS.md) for `mathml_core` / `emit_intent` matrices.
- **Benches**: criterion paths for parse-only vs full `convert`.
- **Fixture corpus**: `tests/fixtures/` + `fixtures_runner` for exact/substring MathML locks.
- **Extra KaTeX-oriented macros**: `\binom`/`\dbinom`/`\tbinom`, `\pmod`/`\bmod`/`\mod`/`\pod`, `\stackrel`, `\mathbin`… class wrappers, more inverse trig/hyperbolic names (`\arccot`, `\lcm`, …); AST gains `MathNode::Binom`.
- **P1 coverage**: infix `\choose` (folds to `Binom`), `\genfrac`, `\substack`, `\middle`, `\displaystyle`/`\textstyle`, `\hskip`/`\kern`/`\mkern`/`\hspace`/`\mskip`, `\tag`/`\notag`; environment align for `aligned`/`split`/`gathered`/`smallmatrix`/`substack`.

## See also

- [`RENDER_OPTIONS.md`](./RENDER_OPTIONS.md) — `mathml_core` and `emit_intent` coverage
- Crate README — high-level 2.x usage
