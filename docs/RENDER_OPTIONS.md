# Render option coverage: `mathml_core` & `emit_intent`

These flags live on [`ConvertOptions`](https://docs.rs/tex2math) / [`RenderOptions`](https://docs.rs/tex2math).

Defaults:

| API | `mathml_core` | `emit_intent` |
|-----|---------------|---------------|
| `convert` / `ConvertOptions::default()` | **true** | false |
| `generate_mathml` / `RenderOptions::default()` | false | false |

## `mathml_core`

When **true**, prefer MathML Core–friendly constructs (avoid non-Core elements).

| Construct | Core (`true`) | Non-core (`false`) |
|-----------|---------------|--------------------|
| `\cancel` / `\bcancel` / `\xcancel` | `<mrow>` (optionally with intent) wrapping content | `<menclose notation="…">` |
| `\boxed{…}` | `<mrow>` (optionally `intent="boxed"`) | `<menclose notation="box">` |
| Everything else | Unchanged | Unchanged |

**Not yet differentiated:** stretch ops, phantoms, `mpadded`, environment tables, fonts, accents — these already use Core-friendly tags or are accepted widely.

## `emit_intent` (experimental MathML 4)

When **true**, emit `intent="…"` on selected presentation elements.

| AST / construct | Intent value | Element |
|-----------------|--------------|---------|
| Fraction (`\frac`, …) | `fraction` | `<mfrac>` |
| Sqrt | `square-root` | `<msqrt>` |
| Root (`\sqrt[n]`) | `root` | `<mroot>` |
| Row / group | `group` | `<mrow>` |
| Fenced (`\left…\right`) | `fenced` | outer `<mrow>` |
| Boxed (core path) | `boxed` | `<mrow>` |
| Cancel (core path) | `cancel:{mode}` | `<mrow>` |
| Accent (`\hat`, …) | `accent` | `<mover accent="true">` |
| Scripts / limits | `scripts` | `<msub>` / `<msup>` / `<munder>` / … / `<mmultiscripts>` |
| Environment table | `table` | `<mtable>` |

**Not covered yet:** `Color`, `ColorBox`, `Style` / font variants, `Phantom`, `StretchOp`, `OperatorName`, bare tokens (`mi`/`mo`/`mn`), `SizedDelimiter`, `Space`, `Function`, `Text`, `Error`.

Intent values are stable for currently covered nodes; new nodes may gain intents in minor releases without changing existing strings.

## Testing

See `src/tests/v2_features.rs` for regression checks on cancel/boxed core paths and intent emission for fraction, root, fenced, scripts, accent, and tables.

For broader LaTeX → MathML coverage (including newly added macros such as `\binom`, `\pmod`, `\stackrel`), see the fixture corpus under [`tests/fixtures/`](../tests/fixtures/README.md) and the runner `tests/fixtures_runner.rs`.
