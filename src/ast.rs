//! Abstract syntax tree and diagnostic types for tex2math 2.0.
//!
//! Leaf text is stored as [`Cow<'s, str>`] so parsers can borrow from the input
//! (or from `'static` symbol tables) and only allocate when necessary.
//! Call [`MathNode::into_owned`] to detach an AST from the input buffer.

use std::borrow::Cow;
use std::ops::Range;

// ---------------------------------------------------------------------------
// Diagnostics
// ---------------------------------------------------------------------------

/// Classifies a parse/convert failure for programmatic handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum ParseErrorKind {
    /// Nesting exceeded [`crate::ParseOptions::max_depth`].
    NestingLimit,
    /// Non-whitespace input remained after a successful partial parse.
    UnexpectedTrailing,
    /// Generic syntax / combinator failure.
    Syntax,
    /// Serialization or host-boundary failure (e.g. WASM).
    Other,
}

/// Structured parse error with span and stable kind.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParseError {
    pub kind: ParseErrorKind,
    /// Byte range into the original input (`start..end`). Empty span uses `start == end == offset`.
    pub span: Range<usize>,
    pub message: String,
}

impl ParseError {
    pub fn new(kind: ParseErrorKind, span: Range<usize>, message: impl Into<String>) -> Self {
        Self {
            kind,
            span,
            message: message.into(),
        }
    }

    pub fn at_offset(kind: ParseErrorKind, offset: usize, message: impl Into<String>) -> Self {
        Self::new(kind, offset..offset, message)
    }

    /// Backward-compatible alias for the span start.
    #[inline]
    pub fn offset(&self) -> usize {
        self.span.start
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Parse error at byte offset {} ({:?}): {}",
            self.span.start, self.kind, self.message
        )
    }
}

impl std::error::Error for ParseError {}

// ---------------------------------------------------------------------------
// Options
// ---------------------------------------------------------------------------

/// How to treat unparsed trailing input after a successful parse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TrailingPolicy {
    /// Report [`ParseErrorKind::UnexpectedTrailing`] (default, safe).
    #[default]
    Error,
    /// Ignore trailing non-whitespace (legacy 1.x silent behavior).
    Ignore,
}

/// Error recovery aggressiveness during parse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RecoveryMode {
    /// Prefer `MathNode::Error` / salvage over hard failure where possible.
    #[default]
    Tolerant,
    /// Fail faster on structural errors (still recovers some brace issues).
    Strict,
}

/// How to treat a `\`-command that is not in any symbol/registry table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UnknownCommandPolicy {
    /// Emit `MathNode::Identifier("\\cmd")` so the glyph still appears (default).
    #[default]
    Identifier,
    /// Emit `MathNode::Error` (renders as red `<merror>` in MathML).
    Error,
}

/// Options controlling LaTeX parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParseOptions {
    /// Maximum parse nesting depth (stack-safe cap). Default: 64.
    pub max_depth: u32,
    pub trailing: TrailingPolicy,
    pub recovery: RecoveryMode,
    /// Behavior for unknown control sequences (see [`UnknownCommandPolicy`]).
    pub unknown_command: UnknownCommandPolicy,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            max_depth: crate::depth::DEFAULT_MAX_NESTING_DEPTH,
            trailing: TrailingPolicy::Error,
            recovery: RecoveryMode::Tolerant,
            unknown_command: UnknownCommandPolicy::Identifier,
        }
    }
}

/// Options for the one-shot LaTeX → MathML conversion.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConvertOptions {
    pub parse: ParseOptions,
    pub mode: RenderMode,
    /// Wrap output in `<math xmlns=...>` root.
    pub wrap_math: bool,
    /// Prefer MathML Core-friendly constructs (default true).
    pub mathml_core: bool,
    /// Emit experimental `intent` attributes where known (default false).
    pub emit_intent: bool,
}

impl Default for ConvertOptions {
    fn default() -> Self {
        Self {
            parse: ParseOptions::default(),
            mode: RenderMode::Inline,
            wrap_math: true,
            mathml_core: true,
            emit_intent: false,
        }
    }
}

impl ConvertOptions {
    pub fn display() -> Self {
        Self {
            mode: RenderMode::Display,
            ..Default::default()
        }
    }
}

// ---------------------------------------------------------------------------
// Formatting enums
// ---------------------------------------------------------------------------

/// The rendering mode for the mathematical formula.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RenderMode {
    Inline,
    Display,
}

/// Limit placement for operators (`\limits` / `\nolimits` / default).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LimitBehavior {
    Default,
    Limits,
    NoLimits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PhantomKind {
    /// `\phantom`: invisible, full width/height/depth.
    Invisible,
    /// `\vphantom`: height/depth only.
    Vertical,
    /// `\hphantom`: width only.
    Horizontal,
}

// ---------------------------------------------------------------------------
// AST
// ---------------------------------------------------------------------------

/// Abstract syntax tree node for a mathematical expression (tex2math 2.0).
///
/// Lifetime `'s` is the borrow of the source string (and/or `'static` tables).
/// Use [`MathNode::into_owned`] when the tree must outlive the input.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum MathNode<'s> {
    Number(Cow<'s, str>),
    Identifier(Cow<'s, str>),
    Operator(Cow<'s, str>),
    Fraction(Box<MathNode<'s>>, Box<MathNode<'s>>),
    /// Binomial coefficient `\binom{n}{k}` → parenthesized fraction with zero rule.
    Binom(Box<MathNode<'s>>, Box<MathNode<'s>>),
    /// Infix `\choose` marker; folded into [`MathNode::Binom`] by the semantic pass.
    ChooseMarker,
    /// Stretchy delimiter from `\middle` (inside `\left`…`\right`).
    Middle(Cow<'s, str>),
    Scripts {
        base: Box<MathNode<'s>>,
        sub: Option<Box<MathNode<'s>>>,
        sup: Option<Box<MathNode<'s>>>,
        pre_sub: Option<Box<MathNode<'s>>>,
        pre_sup: Option<Box<MathNode<'s>>>,
        behavior: LimitBehavior,
    },
    Row(Vec<MathNode<'s>>),
    Sqrt(Box<MathNode<'s>>),
    Root {
        index: Box<MathNode<'s>>,
        content: Box<MathNode<'s>>,
    },
    Fenced {
        open: Cow<'s, str>,
        content: Box<MathNode<'s>>,
        close: Cow<'s, str>,
    },
    Environment {
        name: Cow<'s, str>,
        format: Option<Cow<'s, str>>,
        rows: Vec<(Vec<MathNode<'s>>, Option<Cow<'s, str>>)>,
    },
    Text(Cow<'s, str>),
    Style {
        variant: Cow<'s, str>,
        content: Box<MathNode<'s>>,
    },
    Accent {
        mark: Cow<'s, str>,
        content: Box<MathNode<'s>>,
    },
    Function(Cow<'s, str>),
    OperatorName(Box<MathNode<'s>>),
    SizedDelimiter {
        size: Cow<'s, str>,
        delim: Cow<'s, str>,
    },
    Space(Cow<'s, str>),
    Color {
        color: Cow<'s, str>,
        content: Box<MathNode<'s>>,
    },
    ColorBox {
        bg_color: Cow<'s, str>,
        content: Box<MathNode<'s>>,
    },
    Boxed(Box<MathNode<'s>>),
    Phantom {
        kind: PhantomKind,
        content: Box<MathNode<'s>>,
    },
    Cancel {
        mode: Cow<'s, str>,
        content: Box<MathNode<'s>>,
    },
    StretchOp {
        op: Cow<'s, str>,
        is_over: bool,
        content: Box<MathNode<'s>>,
    },
    StyledMath {
        displaystyle: bool,
        content: Box<MathNode<'s>>,
    },
    Error(Cow<'s, str>),
}

/// Fully owned AST (`'static` cows).
pub type OwnedMathNode = MathNode<'static>;

impl<'s> MathNode<'s> {
    #[inline]
    pub fn is_large_op(&self) -> bool {
        match self {
            MathNode::Operator(op) => crate::symbols::is_large_op_symbol(op.as_ref()),
            MathNode::Function(f) => crate::symbols::is_large_math_function(f.as_ref()),
            MathNode::StretchOp { .. } => true,
            _ => false,
        }
    }

    /// Deep-copy into an owned tree that no longer borrows the input.
    pub fn into_owned(self) -> MathNode<'static> {
        use MathNode::*;
        match self {
            Number(s) => Number(Cow::Owned(s.into_owned())),
            Identifier(s) => Identifier(Cow::Owned(s.into_owned())),
            Operator(s) => Operator(Cow::Owned(s.into_owned())),
            Fraction(a, b) => Fraction(Box::new(a.into_owned()), Box::new(b.into_owned())),
            Binom(a, b) => Binom(Box::new(a.into_owned()), Box::new(b.into_owned())),
            ChooseMarker => ChooseMarker,
            Middle(s) => Middle(Cow::Owned(s.into_owned())),
            Scripts {
                base,
                sub,
                sup,
                pre_sub,
                pre_sup,
                behavior,
            } => Scripts {
                base: Box::new(base.into_owned()),
                sub: sub.map(|n| Box::new(n.into_owned())),
                sup: sup.map(|n| Box::new(n.into_owned())),
                pre_sub: pre_sub.map(|n| Box::new(n.into_owned())),
                pre_sup: pre_sup.map(|n| Box::new(n.into_owned())),
                behavior,
            },
            Row(nodes) => Row(nodes.into_iter().map(MathNode::into_owned).collect()),
            Sqrt(c) => Sqrt(Box::new(c.into_owned())),
            Root { index, content } => Root {
                index: Box::new(index.into_owned()),
                content: Box::new(content.into_owned()),
            },
            Fenced {
                open,
                content,
                close,
            } => Fenced {
                open: Cow::Owned(open.into_owned()),
                content: Box::new(content.into_owned()),
                close: Cow::Owned(close.into_owned()),
            },
            Environment { name, format, rows } => Environment {
                name: Cow::Owned(name.into_owned()),
                format: format.map(|f| Cow::Owned(f.into_owned())),
                rows: rows
                    .into_iter()
                    .map(|(cells, sp)| {
                        (
                            cells.into_iter().map(MathNode::into_owned).collect(),
                            sp.map(|s| Cow::Owned(s.into_owned())),
                        )
                    })
                    .collect(),
            },
            Text(s) => Text(Cow::Owned(s.into_owned())),
            Style { variant, content } => Style {
                variant: Cow::Owned(variant.into_owned()),
                content: Box::new(content.into_owned()),
            },
            Accent { mark, content } => Accent {
                mark: Cow::Owned(mark.into_owned()),
                content: Box::new(content.into_owned()),
            },
            Function(s) => Function(Cow::Owned(s.into_owned())),
            OperatorName(c) => OperatorName(Box::new(c.into_owned())),
            SizedDelimiter { size, delim } => SizedDelimiter {
                size: Cow::Owned(size.into_owned()),
                delim: Cow::Owned(delim.into_owned()),
            },
            Space(s) => Space(Cow::Owned(s.into_owned())),
            Color { color, content } => Color {
                color: Cow::Owned(color.into_owned()),
                content: Box::new(content.into_owned()),
            },
            ColorBox { bg_color, content } => ColorBox {
                bg_color: Cow::Owned(bg_color.into_owned()),
                content: Box::new(content.into_owned()),
            },
            Boxed(c) => Boxed(Box::new(c.into_owned())),
            Phantom { kind, content } => Phantom {
                kind,
                content: Box::new(content.into_owned()),
            },
            Cancel { mode, content } => Cancel {
                mode: Cow::Owned(mode.into_owned()),
                content: Box::new(content.into_owned()),
            },
            StretchOp {
                op,
                is_over,
                content,
            } => StretchOp {
                op: Cow::Owned(op.into_owned()),
                is_over,
                content: Box::new(content.into_owned()),
            },
            StyledMath {
                displaystyle,
                content,
            } => StyledMath {
                displaystyle,
                content: Box::new(content.into_owned()),
            },
            Error(s) => Error(Cow::Owned(s.into_owned())),
        }
    }
}

/// Borrow a string slice as a cow (zero alloc when from input/static).
#[inline]
pub fn cow_borrowed<'s>(s: &'s str) -> Cow<'s, str> {
    Cow::Borrowed(s)
}

/// Own a string as a cow.
#[inline]
pub fn cow_owned<'s>(s: String) -> Cow<'s, str> {
    Cow::Owned(s)
}

/// Lift a `'static` str into any `'s` cow.
#[inline]
pub fn cow_static<'s>(s: &'static str) -> Cow<'s, str> {
    Cow::Borrowed(s)
}
