# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.4.1] — 2026-08-26

### Added

- **Online Visual Fixtures Gallery**:
  - Automated deployment of the visual fixtures gallery to GitHub Pages on every release/documentation update ([https://twn39.github.io/tex2math/gallery.html](https://twn39.github.io/tex2math/gallery.html)).
  - Added repository and Rust API documentation links in the visual gallery header.
  - Added gallery badge and online documentation links in `README.md` and `tests/fixtures/README.md`.

## [2.4.0] — 2026-08-25

### Added

- **New LaTeX Commands & Symbols**:
  - `\prescript{sup}{sub}{base}` command for nuclear isotope and tensor pre-superscript/subscript notation (emits `<mmultiscripts><mprescripts/></mmultiscripts>`).
  - `\hline` horizontal divider lines in `array` and `matrix` environments (emits `<mtable rowlines="...">`).
  - `\coloncolonequals`, `\coloncoloneq`, `\coloncoloneqq` double colon equal operator symbols (`∷=`, U+2A74).
- **CLI Visual Options**:
  - `--html` flag to generate a standalone HTML document with MathML preview and syntax badge.
  - `--preview` (`-p`) flag to generate HTML to a temporary file and automatically launch the default browser.
- **Visual Fixture Gallery**:
  - Added interactive browser gallery generator `examples/generate_gallery.rs` with suite filtering, search, and native MathML render cards.
  - Expanded test fixture corpus to **209 real-world formulas** (covering advanced physics, calculus, linear algebra, delimiters, environments, and showcase proofs).

### Changed & Performance

- **Zero-Copy & Parser Optimizations**:
  - Upgraded parser combinators to `winnow = "1.0"`.
  - Replaced heap recursion stack with `SmallVec<[Frame; 32]>` for 0-heap allocations in hot-path rendering.
  - Optimized streaming output pre-allocation (+14.5% benchmark speedup on limit scripts, +10.7% on wide sums).
- **CSS Improvements**:
  - Added `white-space: pre` on `<mtext>` in HTML previews to preserve leading/trailing whitespace in `\text{...}`.
  - Fixed Flexbox center overflow scroll clipping for ultra-wide formulas.

## [2.3.0] — 2026-07-15

### Added

- **`emit_intent` coverage** for previously missing constructs:
  - bare tokens: `number` / `identifier` / `operator`
  - `Text` → `text`, `SizedDelimiter` → `sized-delimiter`, `Middle` → `middle`
  - `StretchOp` → `stretch-over` / `stretch-under`
  - `StyledMath` → `displaystyle` / `textstyle`
- **CLI flags**: `--no-mathml-core`, `--emit-intent` (library options were already available; WASM `convert_to_mathml_with_options` already exposed `mathmlCore` / `emitIntent`).
- **Irregular command snapshot test** (`irregular_cmds_snapshot_guardrail`) so new multi-arg macros prefer registry tables over `IRREGULAR_CMDS` growth.
- **`docs/PERFORMANCE.md`**: how to run criterion benches and hot-path notes.
- **`CHANGELOG.md`** and crate version alignment with the 2.x feature timeline.

### Changed

- Crate version **2.0.0 → 2.3.0** to match documented post-2.0 capabilities (sema, registry, MathSink, intent matrix, fixtures).
- `sema` module docs: fixed **pass order** (primes → prescripts → choose → normalize).
- Intent / Core coverage tables in `docs/RENDER_OPTIONS.md` updated.

### Documentation

- README CLI table, migration notes, and render-option matrices refreshed for 2.3.

## [2.2.0] — (historical, unreleased as separate crates.io tags)

Documented in-tree as the MathSink + heap-iterative renderer split era:

- `MathSink` / `render_mathml_to`
- Heap-iterative MathML expand by AST family (`tokens` / `structure` / `style` / `scripts` / `environment`)

## [2.1.0] — (historical)

- Semantic pass (`sema::analyze`)
- Command registry single-source + `supports_command`
- `mathml_core` / experimental `emit_intent` (partial matrix)

## [2.0.0] — 2025

### Breaking

- Borrowed AST: `MathNode<'s>` with `Cow<'s, str>` leaves
- Structured `ParseError { kind, span, message }`
- Primary APIs: `parse` / `convert` + options
- Default trailing policy: error on junk

See [docs/MIGRATION-2.0.md](docs/MIGRATION-2.0.md).

[2.4.1]: https://github.com/twn39/tex2math/releases/tag/v2.4.1
[2.4.0]: https://github.com/twn39/tex2math/releases/tag/v2.4.0
[2.3.0]: https://github.com/twn39/tex2math/releases/tag/v2.3.0
