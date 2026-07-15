# Render option coverage: `mathml_core` & `emit_intent`

These flags live on [`ConvertOptions`](https://docs.rs/tex2math) / [`RenderOptions`](https://docs.rs/tex2math).

Defaults:

| API | `mathml_core` | `emit_intent` |
|-----|---------------|---------------|
| `convert` / `ConvertOptions::default()` | **true** | false |
| `generate_mathml` / `RenderOptions::default()` | false | false |
| CLI | true (use `--no-mathml-core` to disable) | false (`--emit-intent`) |
| WASM `convert_to_mathml_with_options` | true unless `mathmlCore: false` | `emitIntent` optional |

## `mathml_core`

When **true**, prefer MathML Core–friendly constructs (avoid non-Core elements).

| Construct | Core (`true`) | Non-core (`false`) |
|-----------|---------------|--------------------|
| `\cancel` / `\bcancel` / `\xcancel` | `<mrow>` (optionally with intent) wrapping content | `<menclose notation="…">` |
| `\boxed{…}` | `<mrow>` (optionally `intent="boxed"`) | `<menclose notation="box">` |
| Everything else | Unchanged | Unchanged |

Stretch ops, phantoms, `mpadded`, environment tables, fonts, and accents already use Core-friendly tags (or tags widely accepted); they are **not** branched on this flag.

## `emit_intent` (experimental MathML 4)

When **true**, emit `intent="…"` on presentation elements.

### Structural & scripts

| AST / construct | Intent value | Element |
|-----------------|--------------|---------|
| Fraction (`\frac`, …) | `fraction` | `<mfrac>` |
| Binom | `binomial` | outer `<mrow>` / inner `<mfrac>` |
| Sqrt | `square-root` | `<msqrt>` |
| Root (`\sqrt[n]`) | `root` | `<mroot>` |
| Row / group | `group` | `<mrow>` |
| Fenced (`\left…\right`) | `fenced` | outer `<mrow>` |
| Boxed (core path) | `boxed` | `<mrow>` |
| Cancel (core path) | `cancel:{mode}` | `<mrow>` |
| Accent (`\hat`, …) | `accent` | `<mover accent="true">` |
| Scripts / limits | `scripts` | `<msub>` / `<msup>` / `<munder>` / … / `<mmultiscripts>` |
| Environment table | `table` | `<mtable>` |

### Style & tokens (2.3+)

| AST / construct | Intent value | Element |
|-----------------|--------------|---------|
| Color | `color` | `<mstyle>` |
| ColorBox | `colorbox` | `<mstyle>` |
| Style / font | `style` | `<mstyle>` |
| StyledMath | `displaystyle` / `textstyle` | `<mstyle>` |
| Phantom / v/h | `phantom` / `vphantom` / `hphantom` | `<mphantom>` / `<mpadded>` |
| StretchOp over/under | `stretch-over` / `stretch-under` | `<mover>` / `<munder>` |
| OperatorName | `operator-name` | `<mi>` / `<mrow>` |
| MathClass | `math-class:{bin\|rel\|…}` | `<mo>` / `<mrow>` |
| Function | `function` | `<mi>` |
| Space | `space` | `<mspace>` |
| Error | `error` | `<merror>` |
| Number | `number` | `<mn>` |
| Identifier | `identifier` | `<mi>` |
| Operator | `operator` | `<mo>` |
| Text | `text` | `<mtext>` |
| SizedDelimiter | `sized-delimiter` | `<mo>` |
| Middle | `middle` | `<mo>` |

Intent values for covered nodes are stable; new nodes may gain intents in minor releases without changing existing strings.

## Testing

See `src/tests/v2_features.rs` (`emit_intent_coverage_matrix` and related tests).

For broader LaTeX → MathML coverage, see the fixture corpus under [`tests/fixtures/`](../tests/fixtures/README.md) and the runner `tests/fixtures_runner.rs`.
