# Command dispatch & coverage

## How commands are classified

Bare names (no `\`) go through [`registry::command_spec`](../src/registry.rs):

| [`CommandSpec`](../src/registry.rs) | Source of truth | Parser role |
|-------------------------------------|-----------------|-------------|
| Table families (`FontStyle`, `Accent`, `Binom`, …) | Const tables in `registry.rs` | Fixed argument shapes in `parser/command/*` |
| `Structural` | `STRUCTURAL_CMDS` (`frac`, `sqrt`, `left`, `right`) | Outer combinators in `parser/mod.rs` |
| `Irregular` | `IRREGULAR_CMDS` | Hand combinators (`text`, `color`, `genfrac`, `tag`, …) |
| *(none)* | — | Zero-arg tables / `symbols` / unknown policy |

**Do not** re-list table-driven names in `IRREGULAR_CMDS`. Names live once in their table.

Parser layout (mirrors MathML `iter/` families):

```text
src/parser/command/
  mod.rs      # command_spec dispatch + zero-arg + irregular match
  style.rs    # font, color, accent, cancel, phantom, style switch
  scripts.rs  # overset, sideset, stretch, arrows, not, operatorname
  frac.rs     # dfrac/tfrac, binom, genfrac
  space.rs    # hskip/kern, sized delims
  misc.rs     # mod, math-class, substack, middle, tag
```

## Product boundaries

| Feature | Behavior |
|---------|----------|
| `\mathbin` / `\mathrel` / … | `MathNode::MathClass` → spaced `<mo>` (or `<mrow>`) |
| `\tag{…}` / `\notag` | **Inline** parenthesized tag (leading space); not right-aligned equation numbers |
| Unknown `\`-commands | `UnknownCommandPolicy::Identifier` (default) or `Error` → `<merror>` |
| Known environments | `matrix` / `pmatrix` / `align*` / `cases` / `array` / `smallmatrix` / … (`KNOWN_ENVIRONMENTS`) |
| Unknown environments | Still parsed as a table + `Unknown environment '…'` `<merror>` (Tolerant); Strict drops the table |
| `smallmatrix` / `substack` | Rendered with `scriptlevel="+1"` |
| `\\[dim]` row gaps | Emitted as MathML `rowspacing` when present |

## Listing known commands

```rust
use tex2math::{registered_command_names, supports_command};

assert!(supports_command("mathbf"));
let names = registered_command_names(); // registry tables only (not full symbol glyphs)
```

Registry payload tables are **sorted by key** and looked up with binary search
(`lookup` / `contains_key`). Keep them sorted when editing — unit test
`registry_tables_are_sorted_for_binary_search` enforces this.

Regenerate a dump for docs:

```bash
cargo test -q registered_command_names_is_sorted -- --nocapture 2>/dev/null || true
# or call registered_command_names() from a small bin / REPL
```

## WASM (feature `wasm`)

| JS export | Purpose |
|-----------|---------|
| `convert_to_mathml(latex, display)` | Simple convert |
| `convert_to_mathml_with_options(latex, opts)` | Full options object (camelCase) |
| `convert_batch([...])` | Batch convert; errors are structured |
| `parse_to_ast(latex)` | Owned AST JSON |
| `wasm_supports_command(name)` | Command coverage probe |

Parse/convert failures return `{ kind, message, spanStart, spanEnd }` (byte offsets).

## Adding a command

1. **Table-shaped** (one payload + fixed args): add a row to the appropriate `registry` table; ensure `command_spec` returns the family (automatic if table is wired in `command_spec`). **Prefer this path.**
2. **Irregular** shape: add name to `IRREGULAR_CMDS`, implement combinator under `parser/command/`, arm in `parse_irregular_cmd`, and **update** the snapshot in `irregular_cmds_snapshot_guardrail` (`src/tests/v2_features.rs`).
3. **Symbol only**: add to `symbols.rs` (no registry entry needed).
4. Add a fixture under `tests/fixtures/` when output must stay stable.
5. Prefer an **AST shape unit test** for the new node, not only MathML string checks.
