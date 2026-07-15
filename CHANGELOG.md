# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[2.3.0]: https://github.com/twn39/tex2math/releases/tag/v2.3.0
