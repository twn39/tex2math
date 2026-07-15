# Performance notes

tex2math is designed for a **zero-copy parse hot path** and **stack-safe render**.

## Benchmarks

```bash
cargo bench --bench math_parser_bench
```

Criterion group `tex2math_pipeline` includes:

| Bench | What it measures |
|-------|------------------|
| `parse_only_nested` | Parse + sema only (no MathML string) |
| `convert_nested` | Full parse → MathML (nested continued fractions) |
| `convert_matrix` | Environments / tables |
| `convert_scripts_limits` | Scripts, large ops, limits |
| (wide expression) | Many sibling tokens |

Record numbers locally when changing registry, sema, or the iterative renderer. There is no CI regression gate yet—treat results as a **manual baseline**.

## Hot-path design

| Stage | Allocation policy |
|-------|-------------------|
| Parser | Prefer `&str` / `Cow::Borrowed` from input or `'static` tables |
| Registry | Sorted static tables + binary search (`lookup` / `contains_key`) |
| Semantics | Tree rewrites; may allocate new `Row` / boxed nodes |
| Render | Heap-iterative expand; `Frame::Lit` avoids format! for fixed tags; `MathSink` can stream without an intermediate `String` |

## Places that intentionally allocate

- `ParseError` messages and some recovery `MathNode::Error` / unknown-command `Cow::Owned`
- `emit_intent` + dynamic attributes (`format!` for colored styles, cancel mode, …)
- Outer `convert` wrapper when `wrap_math: true` builds the `<math>` envelope `String`
- WASM / `into_owned` at host boundaries

## When optimizing further

1. Prefer more `Frame::Lit` / static open tags over `Frame::Owned` in the renderer.
2. Keep new table-driven commands out of `IRREGULAR_CMDS` (fixed arity, no extra parse allocations).
3. Avoid rendering-time rewrites—do structural work in `sema::analyze` once.
4. Compare `parse_only_*` vs `convert_*` to see whether a regression is parser or emitter side.
