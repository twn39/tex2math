//! Data-driven LaTeX command metadata (tex2math 2.x).
//!
//! Pure macros and simple name→payload maps live here so `parser/command` stays
//! focused on combinator wiring for irregular syntax.
//!
//! **Single source of truth:** table-driven command *names* are listed only here.
//! Tables are **sorted by key** so [`lookup`] / [`contains_key`] use binary search.
//! The parser dispatches via [`command_spec`] — never by re-enumerating strings
//! in `match` arms.

/// Font-style commands → MathML `mathvariant` values.
/// Keys are **sorted** for O(log n) [`lookup`].
pub const FONT_STYLES: &[(&str, &str)] = &[
    ("boldsymbol", "bold-italic"),
    ("mathbb", "double-struck"),
    ("mathbf", "bold"),
    ("mathcal", "script"),
    ("mathfrak", "fraktur"),
    ("mathit", "italic"),
    ("mathrm", "normal"),
    ("mathsf", "sans-serif"),
    ("mathtt", "monospace"),
    ("mathup", "normal"),
    ("mit", "italic"),
    ("rm", "normal"),
];

/// Accent commands → accent mark character.
/// Keys are **sorted** for O(log n) [`lookup`].
pub const ACCENTS: &[(&str, &str)] = &[
    ("bar", "¯"),
    ("breve", "˘"),
    ("check", "ˇ"),
    ("ddddot", "¨"),
    ("ddot", "¨"),
    ("dot", "˙"),
    ("hat", "^"),
    ("tilde", "~"),
    ("vec", "→"),
    ("widehat", "^"),
    ("widetilde", "~"),
];

/// Sized delimiter commands → CSS size.
/// Keys are **sorted** for O(log n) [`lookup`].
pub const SIZED_DELIMS: &[(&str, &str)] = &[
    ("Big", "1.8em"),
    ("Bigg", "3.0em"),
    ("Biggl", "3.0em"),
    ("Biggm", "3.0em"),
    ("Biggr", "3.0em"),
    ("Bigl", "1.8em"),
    ("Bigm", "1.8em"),
    ("Bigr", "1.8em"),
    ("big", "1.2em"),
    ("bigg", "2.4em"),
    ("biggl", "2.4em"),
    ("biggm", "2.4em"),
    ("biggr", "2.4em"),
    ("bigl", "1.2em"),
    ("bigm", "1.2em"),
    ("bigr", "1.2em"),
];

/// Cancel commands → menclose notation.
/// Keys are **sorted** for O(log n) [`lookup`].
pub const CANCEL_MODES: &[(&str, &str)] = &[
    ("bcancel", "downdiagonalstrike"),
    ("cancel", "updiagonalstrike"),
    ("xcancel", "updiagonalstrike downdiagonalstrike"),
];

/// Extensible arrows → operator glyph.
/// Keys are **sorted** for O(log n) [`lookup`].
pub const EXTENSIBLE_ARROWS: &[(&str, &str)] = &[
    ("xLeftarrow", "\u{21D0}"),
    ("xLeftrightarrow", "\u{21D4}"),
    ("xRightarrow", "\u{21D2}"),
    ("xhookleftarrow", "\u{21A9}"),
    ("xhookrightarrow", "\u{21AA}"),
    ("xleftarrow", "\u{2190}"),
    ("xleftrightarrow", "\u{2194}"),
    ("xlongequal", "="),
    ("xmapsto", "\u{21A6}"),
    ("xrightarrow", "\u{2192}"),
    ("xtwoheadleftarrow", "\u{219E}"),
    ("xtwoheadrightarrow", "\u{21A0}"),
];

/// Stretch under/over operators: (cmd, glyph, is_over).
/// Keys are **sorted** for O(log n) [`lookup_stretch`].
pub const STRETCH_OPS: &[(&str, &str, bool)] = &[
    ("overbrace", "⏞", true),
    ("overleftarrow", "\u{2190}", true),
    ("overleftrightarrow", "\u{2194}", true),
    ("overline", "¯", true),
    ("overrightarrow", "\u{2192}", true),
    ("underbrace", "⏟", false),
    ("underleftarrow", "\u{2190}", false),
    ("underleftrightarrow", "\u{2194}", false),
    ("underline", "_", false),
    ("underrightarrow", "\u{2192}", false),
];

/// Phantom kinds by command name.
/// Entries are **sorted** for O(log n) [`contains_key`].
pub const PHANTOM_KINDS: &[&str] = &["hphantom", "phantom", "vphantom"];

/// Frac style: true = displaystyle, false = textstyle.
/// Keys are **sorted** for O(log n) [`lookup_bool`].
pub const FRAC_STYLES: &[(&str, bool)] = &[("cfrac", true), ("dfrac", true), ("tfrac", false)];

/// Binomial style: `None` = default style, `Some(true)` = display, `Some(false)` = text.
/// Keys are **sorted** for O(log n) lookup.
pub const BINOM_STYLES: &[(&str, Option<bool>)] = &[
    ("binom", None),
    ("dbinom", Some(true)),
    ("tbinom", Some(false)),
];

/// Math-class wrappers (`\mathbin{...}`, …).
/// Entries are **sorted** for O(log n) [`contains_key`].
pub const MATH_CLASS_CMDS: &[&str] = &[
    "mathbin",
    "mathclose",
    "mathop",
    "mathopen",
    "mathord",
    "mathpunct",
    "mathrel",
];

/// Modular congruence helpers.
/// Entries are **sorted** for O(log n) [`contains_key`].
pub const MOD_CMDS: &[&str] = &["bmod", "mod", "pmod", "pod"];

/// Display-style switches (`\displaystyle{...}` or next atom).
/// Keys are **sorted** for O(log n) [`lookup_bool`].
pub const STYLE_SWITCH_CMDS: &[(&str, bool)] = &[("displaystyle", true), ("textstyle", false)];

/// Explicit horizontal spacing with a dimension argument (`\hskip`, `\kern`, …).
/// Entries are **sorted** for O(log n) [`contains_key`].
pub const DIM_SPACE_CMDS: &[&str] = &["hskip", "hspace", "kern", "mkern", "mskip"];

/// Zero-arg identifier aliases (Å, ø, …).
/// Keys are **sorted** for O(log n) [`lookup`].
pub const IDENT_ALIASES: &[(&str, &str)] = &[
    ("AA", "\u{00C5}"),
    ("O", "\u{00D8}"),
    ("aa", "\u{00E5}"),
    ("o", "\u{00F8}"),
];

/// Variant capital Greek → italic letter.
/// Keys are **sorted** for O(log n) [`lookup`].
pub const VAR_GREEK: &[(&str, &str)] = &[
    ("varDelta", "Δ"),
    ("varGamma", "Γ"),
    ("varLambda", "Λ"),
    ("varOmega", "Ω"),
    ("varPhi", "Φ"),
    ("varPi", "Π"),
    ("varPsi", "Ψ"),
    ("varSigma", "Σ"),
    ("varTheta", "Θ"),
    ("varUpsilon", "Υ"),
    ("varXi", "Ξ"),
];

/// Limit-with-arrow zero-arg macros (`\varinjlim`, `\varprojlim`).
/// Keys are **sorted** for O(log n) [`lookup`].
pub const VAR_LIM_CMDS: &[(&str, &str)] = &[("varinjlim", "\u{2192}"), ("varprojlim", "\u{2190}")];

/// Structural commands handled by the outer parser (not `parse_command`).
/// Entries are **sorted** for O(log n) [`contains_key`].
pub const STRUCTURAL_CMDS: &[&str] = &["frac", "left", "right", "sqrt"];

/// Known `\begin{…}` / `\end{…}` environment names (**sorted**).
///
/// Unknown names still parse as tables (recovery) but emit an adjacent
/// [`crate::MathNode::Error`] so MathML surfaces `<merror>`.
pub const KNOWN_ENVIRONMENTS: &[&str] = &[
    "Bmatrix",
    "Vmatrix",
    "align",
    "align*",
    "aligned",
    "alignedat",
    "array",
    "bmatrix",
    "cases",
    "dcases",
    "eqnarray",
    "eqnarray*",
    "equation",
    "equation*",
    "gather",
    "gather*",
    "gathered",
    "matrix",
    "multline",
    "multline*",
    "pmatrix",
    "rcases",
    "smallmatrix",
    "split",
    "substack",
    "vmatrix",
];

/// Whether `name` is a first-class environment (see [`KNOWN_ENVIRONMENTS`]).
#[inline]
pub fn is_known_environment(name: &str) -> bool {
    contains_key(KNOWN_ENVIRONMENTS, name)
}

/// Irregular multi-arg commands: custom combinators in `parser/command` (not table-shaped).
///
/// Table-driven families are **not** listed here.
/// Entries are **sorted** for O(log n) [`contains_key`].
pub const IRREGULAR_CMDS: &[&str] = &[
    "atop",
    "boxed",
    "bra",
    "braket",
    "ce",
    "choose",
    "color",
    "colorbox",
    "curl",
    "dd",
    "div",
    "dv",
    "ev",
    "expval",
    "genfrac",
    "grad",
    "hline",
    "ket",
    "ketbra",
    "laplacian",
    "matrixel",
    "mel",
    "middle",
    "not",
    "notag",
    "operatorname",
    "operatorname*",
    "over",
    "overset",
    "pdv",
    "prescript",
    "pu",
    "rot",
    "sideset",
    "stackrel",
    "substack",
    "tag",
    "text",
    "textcolor",
    "underset",
    "unit",
];

/// Named math functions (`\sin`, `\lim`, …).
/// Entries are **sorted** for O(log n) [`contains_key`].
pub const MATH_FUNCTIONS: &[&str] = &[
    "Pr",
    "arccos",
    "arccosh",
    "arccot",
    "arccsc",
    "arcosh",
    "arcsec",
    "arcsin",
    "arcsinh",
    "arctan",
    "arctanh",
    "arg",
    "arsinh",
    "artanh",
    "cos",
    "cosh",
    "cot",
    "coth",
    "csc",
    "csch",
    "deg",
    "det",
    "dim",
    "exp",
    "gcd",
    "hom",
    "inf",
    "injlim",
    "ker",
    "lcm",
    "lg",
    "lim",
    "liminf",
    "limsup",
    "ln",
    "log",
    "max",
    "min",
    "plim",
    "projlim",
    "sec",
    "sech",
    "sin",
    "sinh",
    "sup",
    "tan",
    "tanh",
    "varliminf",
    "varlimsup",
];

/// Spacing macros.
/// Keys are **sorted** for O(log n) [`lookup`].
pub const SPACING_CMDS: &[(&str, &str)] = &[
    ("!", "-0.1667em"),
    (",", "0.1667em"),
    (":", "0.2222em"),
    (";", "0.2778em"),
    ("enskip", "0.5em"),
    ("enspace", "0.5em"),
    ("medspace", "0.2222em"),
    ("negthinspace", "-0.1667em"),
    ("qquad", "2em"),
    ("quad", "1em"),
    ("thickspace", "0.2778em"),
    ("thinspace", "0.1667em"),
];

/// Blackboard-bold letter macros (`\N`, `\R`, …).
/// Entries are **sorted** for O(log n) [`contains_key`].
pub const BLACKBOARD_LETTERS: &[&str] = &["C", "H", "N", "Q", "R", "Z"];

/// Unified command classification for parser dispatch (tex2math 2.x).
///
/// | Kind | Source of truth | Parser role |
/// |------|-----------------|-------------|
/// | Table families (`FontStyle`…`DimSpace`) | Const tables in this module | Run fixed argument shapes |
/// | [`CommandSpec::Structural`] | [`STRUCTURAL_CMDS`] | Rejected in `parse_command`; outer combinators own them |
/// | [`CommandSpec::Irregular`] | [`IRREGULAR_CMDS`] | Hand-written combinators |
///
/// Call [`command_spec`] once instead of re-scanning tables in the parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CommandSpec {
    FontStyle,
    FracStyle,
    Binom,
    ExtensibleArrow,
    StretchOp,
    Accent,
    Cancel,
    Phantom,
    SizedDelim,
    Mod,
    MathClass,
    StyleSwitch,
    DimSpace,
    Structural,
    Irregular,
}

/// Classify a bare command name (no leading `\`) for dispatch.
///
/// Uses a direct `match` (compiled to a length-switched jump table by LLVM)
/// instead of scanning 15 tables sequentially. Parity with the `pub const`
/// metadata tables is enforced by unit tests.
///
/// Returns `None` when the name is not a registry command (caller may try
/// [`crate::symbols::lookup_symbol`] or unknown-command policy).
#[inline]
pub fn command_spec(cmd: &str) -> Option<CommandSpec> {
    Some(match cmd {
        // STRUCTURAL_CMDS
        "frac" | "left" | "right" | "sqrt" => CommandSpec::Structural,
        // FONT_STYLES
        "boldsymbol" | "mathbb" | "mathbf" | "mathcal" | "mathfrak" | "mathit" | "mathrm"
        | "mathsf" | "mathtt" | "mathup" | "mit" | "rm" => CommandSpec::FontStyle,
        // FRAC_STYLES
        "cfrac" | "dfrac" | "tfrac" => CommandSpec::FracStyle,
        // BINOM_STYLES
        "binom" | "dbinom" | "tbinom" => CommandSpec::Binom,
        // EXTENSIBLE_ARROWS
        "xLeftarrow" | "xLeftrightarrow" | "xRightarrow" | "xhookleftarrow" | "xhookrightarrow"
        | "xleftarrow" | "xleftrightarrow" | "xlongequal" | "xmapsto" | "xrightarrow"
        | "xtwoheadleftarrow" | "xtwoheadrightarrow" => CommandSpec::ExtensibleArrow,
        // STRETCH_OPS
        "overbrace"
        | "overleftarrow"
        | "overleftrightarrow"
        | "overline"
        | "overrightarrow"
        | "underbrace"
        | "underleftarrow"
        | "underleftrightarrow"
        | "underline"
        | "underrightarrow" => CommandSpec::StretchOp,
        // ACCENTS
        "bar" | "breve" | "check" | "ddddot" | "ddot" | "dot" | "hat" | "tilde" | "vec"
        | "widehat" | "widetilde" => CommandSpec::Accent,
        // CANCEL_MODES
        "bcancel" | "cancel" | "xcancel" => CommandSpec::Cancel,
        // PHANTOM_KINDS
        "hphantom" | "phantom" | "vphantom" => CommandSpec::Phantom,
        // SIZED_DELIMS
        "Big" | "Bigg" | "Biggl" | "Biggm" | "Biggr" | "Bigl" | "Bigm" | "Bigr" | "big"
        | "bigg" | "biggl" | "biggm" | "biggr" | "bigl" | "bigm" | "bigr" => {
            CommandSpec::SizedDelim
        }
        // MOD_CMDS
        "bmod" | "mod" | "pmod" | "pod" => CommandSpec::Mod,
        // MATH_CLASS_CMDS
        "mathbin" | "mathclose" | "mathop" | "mathopen" | "mathord" | "mathpunct" | "mathrel" => {
            CommandSpec::MathClass
        }
        // STYLE_SWITCH_CMDS
        "displaystyle" | "textstyle" => CommandSpec::StyleSwitch,
        // DIM_SPACE_CMDS
        "hskip" | "hspace" | "kern" | "mkern" | "mskip" => CommandSpec::DimSpace,
        // IRREGULAR_CMDS
        "atop" | "boxed" | "bra" | "braket" | "ce" | "choose" | "color" | "colorbox" | "curl"
        | "dd" | "div" | "dv" | "ev" | "expval" | "genfrac" | "grad" | "hline" | "ket"
        | "ketbra" | "laplacian" | "matrixel" | "mel" | "middle" | "not" | "notag"
        | "operatorname" | "operatorname*" | "over" | "overset" | "pdv" | "prescript" | "pu"
        | "rot" | "sideset" | "stackrel" | "substack" | "tag" | "text" | "textcolor"
        | "underset" | "unit" => CommandSpec::Irregular,
        _ => return None,
    })
}

/// Binary-search a sorted `(&str, &str)` table.
#[inline]
pub fn lookup<'a>(table: &'a [(&str, &str)], key: &str) -> Option<&'a str> {
    table
        .binary_search_by_key(&key, |&(k, _)| k)
        .ok()
        .map(|i| table[i].1)
}

/// Binary-search a sorted `(&str, bool)` table.
#[inline]
pub fn lookup_bool(table: &[(&str, bool)], key: &str) -> Option<bool> {
    table
        .binary_search_by_key(&key, |&(k, _)| k)
        .ok()
        .map(|i| table[i].1)
}

/// Binary-search a sorted `(&str, Option<bool>)` table.
#[inline]
pub fn lookup_opt_bool(table: &[(&str, Option<bool>)], key: &str) -> Option<Option<bool>> {
    table
        .binary_search_by_key(&key, |&(k, _)| k)
        .ok()
        .map(|i| table[i].1)
}

/// Binary-search membership in a sorted `&str` slice.
#[inline]
pub fn contains_key(table: &[&str], key: &str) -> bool {
    table.binary_search(&key).is_ok()
}

/// Binary-search [`STRETCH_OPS`].
#[inline]
pub fn lookup_stretch(key: &str) -> Option<(&'static str, bool)> {
    STRETCH_OPS
        .binary_search_by_key(&key, |&(k, _, _)| k)
        .ok()
        .map(|i| {
            let (_, glyph, over) = STRETCH_OPS[i];
            (glyph, over)
        })
}

/// Whether `name` is a known table-driven / style family command.
#[inline]
pub fn is_registered_style_cmd(cmd: &str) -> bool {
    matches!(
        command_spec(cmd),
        Some(
            CommandSpec::FontStyle
                | CommandSpec::FracStyle
                | CommandSpec::Binom
                | CommandSpec::ExtensibleArrow
                | CommandSpec::StretchOp
                | CommandSpec::Accent
                | CommandSpec::Cancel
                | CommandSpec::Phantom
                | CommandSpec::SizedDelim
                | CommandSpec::Mod
                | CommandSpec::MathClass
                | CommandSpec::StyleSwitch
                | CommandSpec::DimSpace
        )
    )
}

/// Map `\mathbin` … `\mathpunct` names to [`crate::ast::MathClass`].
#[inline]
pub fn math_class_of(cmd: &str) -> Option<crate::ast::MathClass> {
    use crate::ast::MathClass;
    match cmd {
        "mathbin" => Some(MathClass::Bin),
        "mathrel" => Some(MathClass::Rel),
        "mathop" => Some(MathClass::Op),
        "mathord" => Some(MathClass::Ord),
        "mathopen" => Some(MathClass::Open),
        "mathclose" => Some(MathClass::Close),
        "mathpunct" => Some(MathClass::Punct),
        _ => None,
    }
}

/// True if the bare command name (without leading `\`) is known to the converter.
pub fn supports_command(cmd: &str) -> bool {
    let cmd = cmd.strip_prefix('\\').unwrap_or(cmd);
    command_spec(cmd).is_some()
        || contains_key(MATH_FUNCTIONS, cmd)
        || lookup(SPACING_CMDS, cmd).is_some()
        || contains_key(BLACKBOARD_LETTERS, cmd)
        || lookup(IDENT_ALIASES, cmd).is_some()
        || lookup(VAR_GREEK, cmd).is_some()
        || lookup(VAR_LIM_CMDS, cmd).is_some()
        || crate::symbols::lookup_symbol(cmd).is_some()
}

/// Alphabetically sorted unique bare command names known to the registry tables
/// (style families, functions, spacing, irregular, structural). Symbol-table-only
/// glyphs from [`crate::symbols`] are not included — use [`supports_command`].
pub fn registered_command_names() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = Vec::with_capacity(256);
    let mut push = |s: &'static str| names.push(s);
    for &(k, _) in FONT_STYLES {
        push(k);
    }
    for &(k, _) in ACCENTS {
        push(k);
    }
    for &(k, _) in SIZED_DELIMS {
        push(k);
    }
    for &(k, _) in CANCEL_MODES {
        push(k);
    }
    for &(k, _) in EXTENSIBLE_ARROWS {
        push(k);
    }
    for &(k, _, _) in STRETCH_OPS {
        push(k);
    }
    for &k in PHANTOM_KINDS {
        push(k);
    }
    for &(k, _) in FRAC_STYLES {
        push(k);
    }
    for &(k, _) in BINOM_STYLES {
        push(k);
    }
    for &k in MATH_CLASS_CMDS {
        push(k);
    }
    for &k in MOD_CMDS {
        push(k);
    }
    for &(k, _) in STYLE_SWITCH_CMDS {
        push(k);
    }
    for &k in DIM_SPACE_CMDS {
        push(k);
    }
    for &k in STRUCTURAL_CMDS {
        push(k);
    }
    for &k in KNOWN_ENVIRONMENTS {
        push(k);
    }
    for &k in IRREGULAR_CMDS {
        push(k);
    }
    for &k in MATH_FUNCTIONS {
        push(k);
    }
    for &(k, _) in SPACING_CMDS {
        push(k);
    }
    for &k in BLACKBOARD_LETTERS {
        push(k);
    }
    for &(k, _) in IDENT_ALIASES {
        push(k);
    }
    for &(k, _) in VAR_GREEK {
        push(k);
    }
    for &(k, _) in VAR_LIM_CMDS {
        push(k);
    }
    names.sort_unstable();
    names.dedup();
    names
}
