//! Data-driven LaTeX command metadata (tex2math 2.x).
//!
//! Pure macros and simple name→payload maps live here so `parser/command.rs`
//! stays focused on combinator wiring for irregular syntax.
//!
//! **Single source of truth:** table-driven command *names* are listed only here.
//! The parser dispatches via [`lookup`] / [`lookup_bool`] / [`lookup_stretch`] /
//! membership checks — never by re-enumerating the same strings in `match` arms.

/// Font-style commands → MathML `mathvariant` values.
pub const FONT_STYLES: &[(&str, &str)] = &[
    ("mathbf", "bold"),
    ("mathit", "italic"),
    ("mit", "italic"),
    ("mathbb", "double-struck"),
    ("mathcal", "script"),
    ("mathfrak", "fraktur"),
    ("boldsymbol", "bold-italic"),
    ("mathrm", "normal"),
    ("mathup", "normal"),
    ("rm", "normal"),
    ("mathsf", "sans-serif"),
    ("mathtt", "monospace"),
];

/// Accent commands → accent mark character.
pub const ACCENTS: &[(&str, &str)] = &[
    ("hat", "^"),
    ("widehat", "^"),
    ("vec", "→"),
    ("bar", "¯"),
    ("dot", "˙"),
    ("ddot", "¨"),
    ("ddddot", "¨"),
    ("tilde", "~"),
    ("widetilde", "~"),
    ("check", "ˇ"),
    ("breve", "˘"),
];

/// Sized delimiter commands → CSS size.
pub const SIZED_DELIMS: &[(&str, &str)] = &[
    ("big", "1.2em"),
    ("bigl", "1.2em"),
    ("bigr", "1.2em"),
    ("bigm", "1.2em"),
    ("Big", "1.8em"),
    ("Bigl", "1.8em"),
    ("Bigr", "1.8em"),
    ("Bigm", "1.8em"),
    ("bigg", "2.4em"),
    ("biggl", "2.4em"),
    ("biggr", "2.4em"),
    ("biggm", "2.4em"),
    ("Bigg", "3.0em"),
    ("Biggl", "3.0em"),
    ("Biggr", "3.0em"),
    ("Biggm", "3.0em"),
];

/// Cancel commands → menclose notation.
pub const CANCEL_MODES: &[(&str, &str)] = &[
    ("cancel", "updiagonalstrike"),
    ("bcancel", "downdiagonalstrike"),
    ("xcancel", "updiagonalstrike downdiagonalstrike"),
];

/// Extensible arrows → operator glyph.
pub const EXTENSIBLE_ARROWS: &[(&str, &str)] = &[
    ("xleftarrow", "\u{2190}"),
    ("xrightarrow", "\u{2192}"),
    ("xleftrightarrow", "\u{2194}"),
    ("xRightarrow", "\u{21D2}"),
    ("xLeftarrow", "\u{21D0}"),
    ("xLeftrightarrow", "\u{21D4}"),
    ("xmapsto", "\u{21A6}"),
    ("xlongequal", "="),
    ("xhookleftarrow", "\u{21A9}"),
    ("xhookrightarrow", "\u{21AA}"),
    ("xtwoheadleftarrow", "\u{219E}"),
    ("xtwoheadrightarrow", "\u{21A0}"),
];

/// Stretch under/over operators: (cmd, glyph, is_over).
pub const STRETCH_OPS: &[(&str, &str, bool)] = &[
    ("underbrace", "⏟", false),
    ("overbrace", "⏞", true),
    ("underline", "_", false),
    ("overline", "¯", true),
    ("overrightarrow", "\u{2192}", true),
    ("overleftarrow", "\u{2190}", true),
    ("overleftrightarrow", "\u{2194}", true),
    ("underrightarrow", "\u{2192}", false),
    ("underleftarrow", "\u{2190}", false),
    ("underleftrightarrow", "\u{2194}", false),
];

/// Phantom kinds by command name.
pub const PHANTOM_KINDS: &[&str] = &["phantom", "vphantom", "hphantom"];

/// Frac style: true = displaystyle, false = textstyle.
pub const FRAC_STYLES: &[(&str, bool)] = &[("dfrac", true), ("cfrac", true), ("tfrac", false)];

/// Zero-arg identifier aliases (Å, ø, …).
pub const IDENT_ALIASES: &[(&str, &str)] = &[
    ("AA", "\u{00C5}"),
    ("aa", "\u{00E5}"),
    ("O", "\u{00D8}"),
    ("o", "\u{00F8}"),
];

/// Variant capital Greek → italic letter.
pub const VAR_GREEK: &[(&str, &str)] = &[
    ("varGamma", "Γ"),
    ("varDelta", "Δ"),
    ("varTheta", "Θ"),
    ("varLambda", "Λ"),
    ("varXi", "Ξ"),
    ("varPi", "Π"),
    ("varSigma", "Σ"),
    ("varUpsilon", "Υ"),
    ("varPhi", "Φ"),
    ("varPsi", "Ψ"),
    ("varOmega", "Ω"),
];

/// Limit-with-arrow zero-arg macros (`\varinjlim`, `\varprojlim`).
pub const VAR_LIM_CMDS: &[(&str, &str)] = &[("varinjlim", "\u{2192}"), ("varprojlim", "\u{2190}")];

/// Structural commands handled by the outer parser (not `parse_command`).
pub const STRUCTURAL_CMDS: &[&str] = &["frac", "sqrt", "left", "right"];

/// Irregular multi-arg commands wired by hand in `parse_command` (not table-shaped).
pub const IRREGULAR_CMDS: &[&str] = &[
    "text",
    "color",
    "textcolor",
    "colorbox",
    "boxed",
    "overset",
    "underset",
    "sideset",
    "operatorname",
    "operatorname*",
    "not",
];

#[inline]
pub fn lookup<'a>(table: &'a [(&str, &str)], key: &str) -> Option<&'a str> {
    table
        .iter()
        .find_map(|&(k, v)| if k == key { Some(v) } else { None })
}

#[inline]
pub fn lookup_bool(table: &[(&str, bool)], key: &str) -> Option<bool> {
    table
        .iter()
        .find_map(|&(k, v)| if k == key { Some(v) } else { None })
}

#[inline]
pub fn lookup_stretch(key: &str) -> Option<(&'static str, bool)> {
    STRETCH_OPS
        .iter()
        .find_map(|&(k, glyph, over)| if k == key { Some((glyph, over)) } else { None })
}

/// Whether `name` is a known unary/binary style command handled by the registry tables.
pub fn is_registered_style_cmd(cmd: &str) -> bool {
    lookup(FONT_STYLES, cmd).is_some()
        || lookup(ACCENTS, cmd).is_some()
        || lookup(SIZED_DELIMS, cmd).is_some()
        || lookup(CANCEL_MODES, cmd).is_some()
        || lookup(EXTENSIBLE_ARROWS, cmd).is_some()
        || lookup_stretch(cmd).is_some()
        || PHANTOM_KINDS.contains(&cmd)
        || lookup_bool(FRAC_STYLES, cmd).is_some()
}

/// Named math functions (`\sin`, `\lim`, …).
pub const MATH_FUNCTIONS: &[&str] = &[
    "sin", "cos", "tan", "csc", "sec", "cot", "arcsin", "arccos", "arctan", "sinh", "cosh", "tanh",
    "exp", "log", "ln", "lg", "lim", "limsup", "liminf", "max", "min", "sup", "inf", "det", "arg",
    "dim", "deg", "ker", "hom", "Pr", "gcd", "injlim", "projlim",
];

/// Spacing macros.
pub const SPACING_CMDS: &[(&str, &str)] = &[
    ("quad", "1em"),
    ("qquad", "2em"),
    ("enspace", "0.5em"),
    ("enskip", "0.5em"),
    (",", "0.1667em"),
    ("thinspace", "0.1667em"),
    (":", "0.2222em"),
    ("medspace", "0.2222em"),
    (";", "0.2778em"),
    ("thickspace", "0.2778em"),
    ("!", "-0.1667em"),
    ("negthinspace", "-0.1667em"),
];

pub const BLACKBOARD_LETTERS: &[&str] = &["N", "R", "Z", "C", "Q", "H"];

/// True if the bare command name (without leading `\`) is known to the converter.
pub fn supports_command(cmd: &str) -> bool {
    let cmd = cmd.strip_prefix('\\').unwrap_or(cmd);
    is_registered_style_cmd(cmd)
        || MATH_FUNCTIONS.contains(&cmd)
        || lookup(SPACING_CMDS, cmd).is_some()
        || BLACKBOARD_LETTERS.contains(&cmd)
        || lookup(IDENT_ALIASES, cmd).is_some()
        || lookup(VAR_GREEK, cmd).is_some()
        || lookup(VAR_LIM_CMDS, cmd).is_some()
        || STRUCTURAL_CMDS.contains(&cmd)
        || IRREGULAR_CMDS.contains(&cmd)
        || crate::symbols::lookup_symbol(cmd).is_some()
}
