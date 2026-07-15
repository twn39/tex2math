# Coverage fixtures

Regression corpus for LaTeX → MathML. Prefer adding a fixture here over a
one-off assert in `e2e_*.rs` when locking exact (or substring) output.

## Layout

```text
tests/fixtures/
  baseline/          # stable, already-supported constructs
  coverage/          # KaTeX-oriented macros (binom, pmod, …)
  p1/                # P1 coverage: choose, genfrac, substack, middle, envs, spacing
  <suite>/<name>.tex
  <suite>/<name>.mathml      # exact match (convert, no <math> wrap)
  <suite>/<name>.contains    # optional: required substrings (one per line)
  <suite>/<name>.meta        # optional: mode / flags
```

## `.tex`

- Raw math body only (no `$` / `\[`).
- Lines whose first non-space character is `%` are comments and are stripped.

## `.mathml`

Exact equality against:

```rust
convert(latex, &ConvertOptions {
    wrap_math: false,
    mathml_core: true,  // overridable via .meta
    ..Default::default()
})
```

Regenerate after intentional render changes:

```bash
cargo build --features cli
# then re-run convert for each .tex, or a small script over the tree
```

## `.contains`

One required substring per line (`#` comments allowed). Use when the full tree
is unstable but a structural token must appear (e.g. `linethickness="0"`).

## `.meta`

```text
mode=display
mathml_core=true
emit_intent=false
```

## Runner

`tests/fixtures_runner.rs` walks all `*.tex` under this directory.
