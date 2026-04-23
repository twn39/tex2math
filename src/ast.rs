// ==========================================
// 1. AST (抽象语法树) 定义
// ==========================================

/// Error type returned when parsing fails. Contains the error message and byte offset.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ParseError {
    pub message: String,
    pub offset: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error at byte offset {}: {}", self.offset, self.message)
    }
}

impl std::error::Error for ParseError {}

/// The rendering mode for the mathematical formula.
///
/// `Inline` mode is used for math within text (`$...$`), often leading to smaller fonts and different operator limits.
/// `Display` mode is used for standalone equations (`$$...$$`), often with limits displayed above and below operators.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum RenderMode {
    Inline,
    Display,
}

/// Specifies the rendering behavior of limits (subscripts and superscripts) for operators.
///
/// This determines whether limits are placed to the side of the operator (like `\nolimits`) or
/// directly above and below (like `\limits`), or following the default rules for the operator.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum LimitBehavior {
    Default,
    Limits,   // 强制 \limits (总是生成 munderover)
    NoLimits, // 强制 \nolimits (总是生成 msubsup)
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum PhantomKind {
    /// \phantom: 不可见，但占据完整的原始宽度、高度和深度
    Invisible,
    /// \vphantom: 不可见，保留高度和深度，但将宽度压缩为 0
    Vertical,
    /// \hphantom: 不可见，保留宽度，但将高度和深度压缩为 0
    Horizontal,
}

/// The Abstract Syntax Tree (AST) node representing a mathematical structure parsed from LaTeX.
///
/// This enum is the core representation of all mathematical elements, including numbers, identifiers,
/// operators, fractions, scripts, roots, matrices, and various styling configurations.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum MathNode {
    Number(String),
    Identifier(String),
    Operator(String),
    Fraction(Box<MathNode>, Box<MathNode>),

    // 我们将所有上下标和上下界合并成一个更智能、更通用的统一节点
    // 之前我们分为 SubSup 和 UnderOver，现在我们在生成时动态决定它们！
    Scripts {
        base: Box<MathNode>,
        sub: Option<Box<MathNode>>,
        sup: Option<Box<MathNode>>,
        // 新增：用于张量和前置角标
        pre_sub: Option<Box<MathNode>>,
        pre_sup: Option<Box<MathNode>>,
        behavior: LimitBehavior,
    },

    Row(Vec<MathNode>),
    Sqrt(Box<MathNode>),
    Root {
        index: Box<MathNode>,
        content: Box<MathNode>,
    },
    Fenced {
        open: String,
        content: Box<MathNode>,
        close: String,
    },
    Environment {
        name: String,
        format: Option<String>,
        rows: Vec<(Vec<MathNode>, Option<String>)>,
    },
    Text(String),
    Style {
        variant: String,
        content: Box<MathNode>,
    },
    Accent {
        mark: String,
        content: Box<MathNode>,
    },
    Function(String),
    OperatorName(Box<MathNode>), // For \operatorname{...} allowing complex content
    SizedDelimiter {
        size: String,
        delim: String,
    },
    Space(String),

    // == 新增：高级文本处理与颜色系统 ==
    Color {
        color: String,
        content: Box<MathNode>,
    },
    ColorBox {
        bg_color: String,
        content: Box<MathNode>,
    },
    Boxed(Box<MathNode>), // 边框

    // == 新增：隐形占位符与约分划线 ==
    Phantom {
        kind: PhantomKind,
        content: Box<MathNode>,
    },
    Cancel {
        mode: String, // 对应 notation 的属性值
        content: Box<MathNode>,
    },

    // == 新增：可拉伸跨度修饰符 ==
    StretchOp {
        op: String,
        is_over: bool,
        content: Box<MathNode>,
    },

    // == 新增：displaystyle 切换，用于 \dfrac, \tfrac 等 ==
    StyledMath {
        displaystyle: bool,
        content: Box<MathNode>,
    },

    Error(String),
}

impl MathNode {
    pub fn is_large_op(&self) -> bool {
        match self {
            MathNode::Operator(op) => crate::symbols::is_large_op_symbol(op),
            MathNode::Function(f) => crate::symbols::is_large_math_function(f),
            MathNode::StretchOp { .. } => true,
            _ => false,
        }
    }
}
