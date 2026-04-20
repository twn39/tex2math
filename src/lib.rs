#![allow(clippy::needless_lifetimes)]
#![allow(clippy::new_without_default)]
#![allow(clippy::redundant_pattern_matching)]
use winnow::ascii::{alpha1, digit1, multispace0 as space0};
use winnow::combinator::{alt, delimited, opt, preceded, repeat, separated, trace};
use winnow::prelude::*;
use winnow::token::{literal, one_of, take_till, take_until};

mod symbols;

// ==========================================
// 1. AST (抽象语法树) 定义
// ==========================================

/// The rendering mode for the mathematical formula.
///
/// `Inline` mode is used for math within text (`$...$`), often leading to smaller fonts and different operator limits.
/// `Display` mode is used for standalone equations (`$$...$$`), often with limits displayed above and below operators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderMode {
    Inline,
    Display,
}

/// Specifies the rendering behavior of limits (subscripts and superscripts) for operators.
///
/// This determines whether limits are placed to the side of the operator (like `\nolimits`) or
/// directly above and below (like `\limits`), or following the default rules for the operator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LimitBehavior {
    Default,
    Limits,   // 强制 \limits (总是生成 munderover)
    NoLimits, // 强制 \nolimits (总是生成 msubsup)
}

#[derive(Debug, Clone, PartialEq)]
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
        is_large_op: bool,
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
    NewLine,
}
// 2. Winnow 解析器 (Parser)
// ==========================================

/// Parses a numeric literal (e.g., `123`, `3.14`) into a `MathNode::Number`.
pub fn parse_number<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace(
        "parse_number",
        // 支持整数和小数，如 42、3.14。使用 take() 捕获整个匹配区间作为字符串。
        (digit1, opt(('.', digit1)))
            .take()
            .map(|s: &str| MathNode::Number(s.to_string())),
    )
    .parse_next(input)
}

/// Parses a single alphabetic character as an identifier into a `MathNode::Identifier`.
pub fn parse_ident<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace(
        "parse_ident",
        winnow::token::one_of(|c: char| c.is_ascii_alphabetic())
            .map(|c: char| MathNode::Identifier(c.to_string())),
    )
    .parse_next(input)
}

/// Parses common mathematical operators (e.g., `+`, `-`, `=`, `!`) into a `MathNode::Operator`.
pub fn parse_operator<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace(
        "parse_operator",
        one_of([
            '+', '-', '=', '<', '>', '(', ')', '[', ']', '|', ',', '/', '*', '.', ':',
        ])
        .map(|c: char| MathNode::Operator(c.to_string())),
    )
    .parse_next(input)
}

/// Parses LaTeX fractions like `\frac{num}{den}`, `\dfrac{num}{den}`, and `\tfrac{num}{den}`.
pub fn parse_fraction<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace("parse_fraction", |input: &mut &'s str| {
        let _ = literal("\\frac").parse_next(input)?;

        // 辅助函数：解析一个 { ... } 块。如果没有右括号，不报错，而是吸收剩余所有字符并报错。
        let mut parse_block = |inp: &mut &'s str| -> ModalResult<MathNode> {
            let _ = preceded(space0, literal("{")).parse_next(inp)?;
            let content = parse_row.parse_next(inp)?;

            if opt(preceded(space0, literal("}")))
                .parse_next(inp)?
                .is_some()
            {
                Ok(content)
            } else {
                let remaining = winnow::token::rest.parse_next(inp)?;
                Ok(MathNode::Row(vec![
                    content,
                    MathNode::Error(format!("Missing '}}' in fraction, found: '{}'", remaining)),
                ]))
            }
        };

        // 第一个块：分子 (若匹配不到左括号，直接失败，因为这不符合 \frac 的特征)
        let num = parse_block.parse_next(input)?;

        // 第二个块：分母 (如果连左括号都没有，那说明整个格式残缺，我们将分母作为一个空的错误)
        let den = if let Ok(d) = parse_block.parse_next(input) {
            d
        } else {
            MathNode::Error("Missing denominator block".to_string())
        };

        Ok(MathNode::Fraction(Box::new(num), Box::new(den)))
    })
    .parse_next(input)
}

/// Parses a grouped expression enclosed in curly braces (`{...}`) into a `MathNode::Row` or a single node.
pub fn parse_group<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace("parse_group", |input: &mut &'s str| {
        // 匹配左大括号
        let _ = preceded(space0, literal("{")).parse_next(input)?;

        // 尝试正常解析一行内容
        let content = parse_row.parse_next(input)?;

        // 尝试匹配右大括号
        if opt(preceded(space0, literal("}")))
            .parse_next(input)?
            .is_some()
        {
            Ok(content)
        } else {
            // == 错误恢复 ==
            // 如果没找到右大括号，说明公式残缺。我们把剩下的内容视为错误节点。
            // 但为了让外层能继续渲染已正确解析的部分，我们返回一个包含了 Error 的 Row。
            let remaining = winnow::token::rest.parse_next(input)?;
            Ok(MathNode::Row(vec![
                content,
                MathNode::Error(format!("Missing '}}', found: '{}'", remaining)),
            ]))
        }
    })
    .parse_next(input)
}

// == 新增：解析 \sqrt 和 \sqrt[3] ==
/// Parses a square root or nth root like `\sqrt{x}` or `\sqrt[n]{x}`.
pub fn parse_sqrt<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace("parse_sqrt", |input: &mut &'s str| {
        let _ = literal("\\sqrt").parse_next(input)?;

        // 提取 [ 和 ] 之间的纯字符串（不让内部的 parse_row 贪婪吃掉外面的 ]）
        let index_str_opt =
            opt(delimited((space0, '['), take_till(0.., |c| c == ']'), ']')).parse_next(input)?;

        let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;

        if let Some(mut idx_str) = index_str_opt {
            // 对提取出来的字符串进行 AST 解析
            let index_node = parse_row.parse_next(&mut idx_str)?;
            Ok(MathNode::Root {
                index: Box::new(index_node),
                content: Box::new(content),
            })
        } else {
            Ok(MathNode::Sqrt(Box::new(content)))
        }
    })
    .parse_next(input)
}

// 解析 \left / \right 后跟随的定界符
// 支持单字符（(, ), [, ], |, .）和命令符号（\langle, \lfloor, \lceil, \lVert 等）
fn parse_fence_delim<'s>(input: &mut &'s str) -> ModalResult<String> {
    preceded(
        space0,
        alt((
            // 隐形定界符
            literal(".").map(|_: &str| ".".to_string()),
            // 转义的单字符定界符：\{, \}, \| 等
            preceded('\\', one_of(['{', '}', '|', '[', ']'])).map(|c: char| c.to_string()),
            // 命令式定界符：\langle, \rangle, \lfloor, \lceil, \lVert 等
            preceded('\\', alpha1).map(|cmd: &str| {
                match cmd {
                    "langle" | "lang" => "\u{27E8}", // ⟨
                    "rangle" | "rang" => "\u{27E9}", // ⟩
                    "lfloor" => "\u{230A}",          // ⌊
                    "rfloor" => "\u{230B}",          // ⌋
                    "lceil" => "\u{2308}",           // ⌈
                    "rceil" => "\u{2309}",           // ⌉
                    "lbrace" => "{",
                    "rbrace" => "}",
                    "lbrack" => "[",
                    "rbrack" => "]",
                    "vert" | "lvert" | "rvert" => "|",
                    "Vert" | "lVert" | "rVert" => "∥", // ∥
                    "uparrow" => "\u{2191}",           // ↑
                    "downarrow" => "\u{2193}",         // ↓
                    "Uparrow" => "\u{21D1}",           // ⇑
                    "Downarrow" => "\u{21D3}",         // ⇓
                    "updownarrow" => "\u{2195}",       // ↕
                    "Updownarrow" => "\u{21D5}",       // ⇕
                    _ => cmd,
                }
                .to_string()
            }),
            // 单字符定界符
            one_of(['(', ')', '[', ']', '{', '}', '|', '<', '>']).map(|c: char| {
                match c {
                    '<' => "\u{27E8}".to_string(), // ⟨
                    '>' => "\u{27E9}".to_string(), // ⟩
                    _ => c.to_string(),
                }
            }),
        )),
    )
    .parse_next(input)
}

// == 新增：解析 \left 和 \right ==
/// Parses dynamically sized fences like `\left( ... \right)`.
pub fn parse_left_right<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace("parse_left_right", |input: &mut &'s str| {
        let _ = literal("\\left").parse_next(input)?;
        let open = parse_fence_delim.parse_next(input)?;
        let content = delimited(space0, parse_row, space0).parse_next(input)?;
        // Fix 3: graceful recovery when \right is missing
        // (e.g. caused by a mis-nested \begin/\end truncating the environment body).
        // Instead of backtracking and leaving \left as an unresolvable stuck atom,
        // we emit a Fenced node with an implicit empty close delimiter '.'.
        let close = if literal::<&str, &str, winnow::error::ContextError>("\\right")
            .parse_next(input)
            .is_ok()
        {
            parse_fence_delim.parse_next(input)?
        } else {
            ".".to_string() // implicit empty close, same as \left. \right.
        };
        Ok(MathNode::Fenced {
            open,
            content: Box::new(content),
            close,
        })
    })
    .parse_next(input)
}
// == 新增：命令符号字典映射 ==
// 参考了 KaTeX 和 texmath 的底层字典，将 LaTeX 命令映射为等价的 Unicode 字符
/// Parses a general LaTeX command starting with a backslash (e.g., `\alpha`, `\int`, `\mathbf{x}`).

// --- Sub-parsers for commands ---

fn parse_text_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    let inner_text = delimited((space0, '{'), take_until(0.., "}"), '}').parse_next(input)?;
    Ok(MathNode::Text(inner_text.to_string()))
}

fn parse_color_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode> {
    match cmd {
        "textcolor" => {
            let color = delimited((space0, '{'), take_until(0.., "}"), '}').parse_next(input)?;
            let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
            Ok(MathNode::Color {
                color: color.to_string(),
                content: Box::new(content),
            })
        }
        "colorbox" => {
            let bg_color = delimited((space0, '{'), take_until(0.., "}"), '}').parse_next(input)?;
            let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
            Ok(MathNode::ColorBox {
                bg_color: bg_color.to_string(),
                content: Box::new(content),
            })
        }
        "color" => {
            let color = delimited((space0, '{'), take_until(0.., "}"), '}').parse_next(input)?;
            let remaining_nodes: Vec<MathNode> =
                repeat(0.., preceded(space0, parse_node)).parse_next(input)?;
            let content = if remaining_nodes.len() == 1 {
                remaining_nodes.into_iter().next().unwrap()
            } else {
                MathNode::Row(remaining_nodes)
            };
            Ok(MathNode::Color {
                color: color.to_string(),
                content: Box::new(content),
            })
        }
        _ => unreachable!(),
    }
}

fn parse_boxed_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
    Ok(MathNode::Boxed(Box::new(content)))
}

fn parse_sideset_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    let left_scripts = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
    let right_scripts = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;

    fn extract_scripts(node: &MathNode) -> (Option<Box<MathNode>>, Option<Box<MathNode>>) {
        match node {
            MathNode::Scripts { sub, sup, .. } => (sub.clone(), sup.clone()),
            MathNode::Row(nodes) if nodes.len() == 1 => extract_scripts(&nodes[0]),
            _ => (None, None),
        }
    }

    let (pre_sub, pre_sup) = extract_scripts(&left_scripts);
    let (post_sub, post_sup) = extract_scripts(&right_scripts);
    let next_node = preceded(space0, parse_node).parse_next(input)?;

    Ok(MathNode::Scripts {
        base: Box::new(next_node),
        sub: post_sub,
        sup: post_sup,
        pre_sub,
        pre_sup,
        behavior: LimitBehavior::Default,
        is_large_op: false,
    })
}

fn parse_over_under_set_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode> {
    let modifier = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
    let base = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
    let (sub, sup) = if cmd == "overset" {
        (None, Some(Box::new(modifier)))
    } else {
        (Some(Box::new(modifier)), None)
    };
    Ok(MathNode::Scripts {
        base: Box::new(base),
        sub,
        sup,
        pre_sub: None,
        pre_sup: None,
        behavior: LimitBehavior::Limits,
        is_large_op: false,
    })
}

fn parse_phantom_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode> {
    let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
    let kind = match cmd {
        "phantom" => PhantomKind::Invisible,
        "vphantom" => PhantomKind::Vertical,
        "hphantom" => PhantomKind::Horizontal,
        _ => unreachable!(),
    };
    Ok(MathNode::Phantom {
        kind,
        content: Box::new(content),
    })
}

fn parse_cancel_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode> {
    let mode = match cmd {
        "cancel" => "updiagonalstrike",
        "bcancel" => "downdiagonalstrike",
        "xcancel" => "updiagonalstrike downdiagonalstrike",
        _ => unreachable!(),
    };
    let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
    Ok(MathNode::Cancel {
        mode: mode.to_string(),
        content: Box::new(content),
    })
}

fn parse_extensible_arrow_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode> {
    let arrow_char = match cmd {
        "xleftarrow" => "\u{2190}",
        "xrightarrow" => "\u{2192}",
        "xleftrightarrow" => "\u{2194}",
        "xRightarrow" => "\u{21D2}",
        "xLeftarrow" => "\u{21D0}",
        "xLeftrightarrow" => "\u{21D4}",
        "xmapsto" => "\u{21A6}",
        "xlongequal" => "=",
        "xhookleftarrow" => "\u{21A9}",
        "xhookrightarrow" => "\u{21AA}",
        "xtwoheadleftarrow" => "\u{219E}",
        "xtwoheadrightarrow" => "\u{21A0}",
        _ => unreachable!(),
    };

    let sub_str_opt =
        opt(delimited((space0, '['), take_till(0.., |c| c == ']'), ']')).parse_next(input)?;
    let sup = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;

    let sub = if let Some(mut s) = sub_str_opt {
        Some(Box::new(parse_row.parse_next(&mut s)?))
    } else {
        None
    };

    Ok(MathNode::Scripts {
        base: Box::new(MathNode::Operator(arrow_char.to_string())),
        sub,
        sup: Some(Box::new(sup)),
        pre_sub: None,
        pre_sup: None,
        behavior: LimitBehavior::Limits,
        is_large_op: true,
    })
}

fn parse_stretch_modifier_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode> {
    let (op_str, is_over) = match cmd {
        "underbrace" => ("⏟", false),
        "overbrace" => ("⏞", true),
        "underline" => ("_", false),
        "overline" => ("¯", true),
        "overrightarrow" => ("\u{2192}", true),
        "overleftarrow" => ("\u{2190}", true),
        "overleftrightarrow" => ("\u{2194}", true),
        "underrightarrow" => ("\u{2192}", false),
        "underleftarrow" => ("\u{2190}", false),
        "underleftrightarrow" => ("\u{2194}", false),
        _ => unreachable!(),
    };
    let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
    Ok(MathNode::StretchOp {
        op: op_str.to_string(),
        is_over,
        content: Box::new(content),
    })
}

fn parse_frac_style_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode> {
    let ds = match cmd {
        "dfrac" | "cfrac" => true,
        "tfrac" => false,
        _ => unreachable!(),
    };
    let num = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
    let den = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
    Ok(MathNode::StyledMath {
        displaystyle: ds,
        content: Box::new(MathNode::Fraction(Box::new(num), Box::new(den))),
    })
}

fn parse_operatorname_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode> {
    let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
    if cmd == "operatorname*" {
        Ok(MathNode::Scripts {
            base: Box::new(MathNode::OperatorName(Box::new(content))),
            sub: None,
            sup: None,
            pre_sub: None,
            pre_sup: None,
            behavior: LimitBehavior::Limits,
            is_large_op: true,
        })
    } else {
        Ok(MathNode::OperatorName(Box::new(content)))
    }
}

fn parse_not_modifier_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    let _ = space0.parse_next(input)?;
    let negated = if let Ok(next_cmd) =
        preceded::<_, _, _, winnow::error::ContextError, _, _>('\\', alpha1).parse_next(input)
    {
        match next_cmd {
            "in" | "isin" => "\u{2209}",
            "ni" | "owns" => "\u{220C}",
            "subset" => "\u{2284}",
            "supset" => "\u{2285}",
            "subseteq" => "\u{2288}",
            "supseteq" => "\u{2289}",
            "sim" => "\u{2241}",
            "approx" => "\u{2249}",
            "equiv" => "\u{2262}",
            "parallel" => "\u{2226}",
            "mid" => "\u{2224}",
            "vdash" => "\u{22AC}",
            "prec" => "\u{2280}",
            "succ" => "\u{2281}",
            "le" | "leq" => "\u{2270}",
            "ge" | "geq" => "\u{2271}",
            "leftarrow" => "\u{219A}",
            "rightarrow" => "\u{219B}",
            other => return Ok(MathNode::Identifier(format!("\\not\\{}", other))),
        }
    } else if opt(one_of::<_, _, winnow::error::ContextError>('='))
        .parse_next(input)?
        .is_some()
    {
        "\u{2260}"
    } else {
        "\u{0338}"
    };
    Ok(MathNode::Operator(negated.to_string()))
}

fn parse_font_style_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode> {
    let variant = match cmd {
        "mathbf" => "bold",
        "mathit" | "mit" => "italic",
        "mathbb" => "double-struck",
        "mathcal" => "script",
        "mathfrak" => "fraktur",
        "boldsymbol" => "bold-italic",
        "mathrm" | "mathup" | "rm" => "normal",
        "mathsf" => "sans-serif",
        "mathtt" => "monospace",
        _ => unreachable!(),
    };

    let content = if let Ok(c) = delimited::<_, _, _, _, winnow::error::ContextError, _, _, _>(
        (space0, '{'),
        parse_row,
        (space0, '}'),
    )
    .parse_next(input)
    {
        c
    } else {
        let remaining_nodes: Vec<MathNode> =
            repeat(0.., preceded(space0, parse_node)).parse_next(input)?;
        if remaining_nodes.is_empty() {
            MathNode::Row(vec![])
        } else if remaining_nodes.len() == 1 {
            remaining_nodes.into_iter().next().unwrap()
        } else {
            MathNode::Row(remaining_nodes)
        }
    };

    Ok(MathNode::Style {
        variant: variant.to_string(),
        content: Box::new(content),
    })
}

fn parse_accent_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode> {
    let mark = match cmd {
        "hat" | "widehat" => "^",
        "vec" => "→",
        "bar" => "¯",
        "dot" => "˙",
        "ddot" | "ddddot" => "¨",
        "tilde" | "widetilde" => "~",
        "check" => "ˇ",
        "breve" => "˘",
        _ => unreachable!(),
    };
    let content = alt((
        delimited((space0, '{'), parse_row, (space0, '}')),
        preceded(space0, parse_atom),
    ))
    .parse_next(input)?;

    Ok(MathNode::Accent {
        mark: mark.to_string(),
        content: Box::new(content),
    })
}

fn parse_sized_delimiter_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode> {
    let size = match cmd {
        "big" | "bigl" | "bigr" | "bigm" => "1.2em",
        "Big" | "Bigl" | "Bigr" | "Bigm" => "1.8em",
        "bigg" | "biggl" | "biggr" | "biggm" => "2.4em",
        "Bigg" | "Biggl" | "Biggr" | "Biggm" => "3.0em",
        _ => unreachable!(),
    };
    let delim = parse_fence_delim.parse_next(input)?;
    Ok(MathNode::SizedDelimiter {
        size: size.to_string(),
        delim: delim.to_string(),
    })
}

fn parse_special_limit_arrow_cmd<'s>(cmd: &str) -> ModalResult<MathNode> {
    let arrow = if cmd == "varinjlim" {
        "\u{2192}"
    } else {
        "\u{2190}"
    };
    Ok(MathNode::Scripts {
        base: Box::new(MathNode::Function("lim".to_string())),
        sub: Some(Box::new(MathNode::Operator(arrow.to_string()))),
        sup: None,
        pre_sub: None,
        pre_sup: None,
        behavior: LimitBehavior::Limits,
        is_large_op: true,
    })
}

fn parse_var_greek_cmd<'s>(cmd: &str) -> ModalResult<MathNode> {
    let letter = match cmd {
        "varGamma" => "Γ",
        "varDelta" => "Δ",
        "varTheta" => "Θ",
        "varLambda" => "Λ",
        "varXi" => "Ξ",
        "varPi" => "Π",
        "varSigma" => "Σ",
        "varUpsilon" => "Υ",
        "varPhi" => "Φ",
        "varPsi" => "Ψ",
        "varOmega" => "Ω",
        _ => unreachable!(),
    };
    Ok(MathNode::Style {
        variant: "italic".to_string(),
        content: Box::new(MathNode::Identifier(letter.to_string())),
    })
}

fn parse_spacing_cmd<'s>(cmd: &str) -> ModalResult<MathNode> {
    let width = match cmd {
        "quad" => "1em",
        "qquad" => "2em",
        "enspace" | "enskip" => "0.5em",
        "," | "thinspace" => "0.1667em",
        ":" | "medspace" => "0.2222em",
        ";" | "thickspace" => "0.2778em",
        "!" | "negthinspace" => "-0.1667em",
        _ => unreachable!(),
    };
    Ok(MathNode::Space(width.to_string()))
}

pub fn parse_command<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace("parse_command", |input: &mut &'s str| {
        let cmd = preceded(
            '\\',
            alt((
                alpha1,
                one_of([
                    ',', ';', ':', '!', '%', '$', '#', '&', '_', ' ', '{', '}', '|',
                ])
                .map(|c: char| c.to_string().leak() as &str),
            )),
        )
        .parse_next(input)?;

        match cmd {
            // 1. 文本
            "text" => parse_text_cmd(input),

            // 2. 颜色与盒子
            "color" | "textcolor" | "colorbox" => parse_color_cmd(cmd, input),
            "boxed" => parse_boxed_cmd(input),

            // 3. 上下限与特殊操作符
            "overset" | "underset" => parse_over_under_set_cmd(cmd, input),
            "sideset" => parse_sideset_cmd(input),
            "operatorname" | "operatorname*" => parse_operatorname_cmd(cmd, input),
            "not" => parse_not_modifier_cmd(input),

            // 4. 字体样式与数学分式
            "mathbf" | "mathit" | "mit" | "mathbb" | "mathcal" | "mathfrak" | "boldsymbol"
            | "mathrm" | "mathup" | "rm" | "mathsf" | "mathtt" => parse_font_style_cmd(cmd, input),
            "dfrac" | "tfrac" | "cfrac" => parse_frac_style_cmd(cmd, input),

            // 5. 箭头与拉伸修饰符
            "xleftarrow" | "xrightarrow" | "xleftrightarrow" | "xRightarrow" | "xLeftarrow"
            | "xLeftrightarrow" | "xmapsto" | "xlongequal" | "xhookleftarrow"
            | "xhookrightarrow" | "xtwoheadleftarrow" | "xtwoheadrightarrow" => {
                parse_extensible_arrow_cmd(cmd, input)
            }
            "underbrace"
            | "overbrace"
            | "underline"
            | "overline"
            | "overrightarrow"
            | "overleftarrow"
            | "overleftrightarrow"
            | "underrightarrow"
            | "underleftarrow"
            | "underleftrightarrow" => parse_stretch_modifier_cmd(cmd, input),

            // 6. 重音与划线
            "hat" | "widehat" | "vec" | "bar" | "dot" | "ddot" | "ddddot" | "tilde"
            | "widetilde" | "check" | "breve" => parse_accent_cmd(cmd, input),
            "cancel" | "bcancel" | "xcancel" => parse_cancel_cmd(cmd, input),

            // 7. 间距、占位与定界符
            "phantom" | "vphantom" | "hphantom" => parse_phantom_cmd(cmd, input),
            "quad" | "qquad" | "enspace" | "enskip" | "," | "thinspace" | ":" | "medspace"
            | ";" | "thickspace" | "!" | "negthinspace" => parse_spacing_cmd(cmd),
            "big" | "bigl" | "bigr" | "bigm" | "Big" | "Bigl" | "Bigr" | "Bigm" | "bigg"
            | "biggl" | "biggr" | "biggm" | "Bigg" | "Biggl" | "Biggr" | "Biggm" => {
                parse_sized_delimiter_cmd(cmd, input)
            }

            // 8. 字母宏与标准函数
            "N" | "R" | "Z" | "C" | "Q" | "H" => Ok(MathNode::Style {
                variant: "double-struck".to_string(),
                content: Box::new(MathNode::Identifier(cmd.to_string())),
            }),
            "sin" | "cos" | "tan" | "csc" | "sec" | "cot" | "arcsin" | "arccos" | "arctan"
            | "sinh" | "cosh" | "tanh" | "exp" | "log" | "ln" | "lg" | "lim" | "limsup"
            | "liminf" | "max" | "min" | "sup" | "inf" | "det" | "arg" | "dim" | "deg" | "ker"
            | "hom" | "Pr" | "gcd" | "injlim" | "projlim" => {
                Ok(MathNode::Function(cmd.to_string()))
            }
            "varinjlim" | "varprojlim" => parse_special_limit_arrow_cmd(cmd),

            // 9. 特殊符号别名与希腊字母变体
            "AA" => Ok(MathNode::Identifier("\u{00C5}".to_string())),
            "aa" => Ok(MathNode::Identifier("\u{00E5}".to_string())),
            "O" => Ok(MathNode::Identifier("\u{00D8}".to_string())),
            "o" => Ok(MathNode::Identifier("\u{00F8}".to_string())),
            "varGamma" | "varDelta" | "varTheta" | "varLambda" | "varXi" | "varPi" | "varSigma"
            | "varUpsilon" | "varPhi" | "varPsi" | "varOmega" => parse_var_greek_cmd(cmd),

            // 10. 排除特定结构命令（交给外层 parser）
            "frac" | "sqrt" | "left" | "right" => winnow::combinator::fail.parse_next(input),

            // 11. 查符号表或兜底为未识别的标识符
            _ => {
                if let Some(node) = crate::symbols::lookup_symbol(cmd) {
                    Ok(node)
                } else {
                    Ok(MathNode::Identifier(format!("\\{}", cmd)))
                }
            }
        }
    })
    .parse_next(input)
}
/// Parses a LaTeX environment enclosed in `\begin{env}` and `\end{env}` (e.g., `matrix`, `cases`).
// == 新增：辅助解析函数 ==

/// 解析一行中由 `&` 分隔的多个单元格 (Cell)
pub fn parse_cells_in_row<'s>(input: &mut &'s str) -> ModalResult<Vec<MathNode>> {
    separated(
        0..,
        delimited(space0, parse_row, space0),
        (space0, '&', space0),
    )
    .parse_next(input)
}

/// 解析换行符 `\\` 及其可选的垂直间距参数，例如 `\\[2mm]`
pub fn parse_newline_opt<'s>(input: &mut &'s str) -> ModalResult<Option<&'s str>> {
    preceded(
        literal("\\\\"),
        opt(delimited((space0, '['), take_till(0.., |c| c == ']'), ']')),
    )
    .parse_next(input)
}

// == 新增：解析 LaTeX Environment (\begin...\end) ==
pub fn parse_environment<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace("parse_environment", |input: &mut &'s str| {
        let begin_tag = preceded((literal("\\begin"), space0, '{'), alpha1).parse_next(input)?;
        let name = begin_tag.to_string();
        let _ = literal("}").parse_next(input)?;

        let mut format = None;
        if name == "array" {
            // 使用 opt 来利用上下文进行推导，这样编译器就能知道 Error 是 winnow::error::ContextError
            let fmt_opt: Option<&str> =
                opt(delimited((space0, '{'), take_until(0.., "}"), '}')).parse_next(input)?;
            if let Some(fmt_str) = fmt_opt {
                format = Some(fmt_str.to_string());
            }
        }

        let begin_pattern = format!("\\begin{{{}}}", name);
        let end_pattern = format!("\\end{{{}}}", name);

        // Fix 1 (root cause): nesting-aware body extraction.
        // The naive take_until(0.., end_pattern) is a flat substring search that stops
        // at the FIRST \end{name} it finds, even when that token belongs to a nested
        // same-name environment (e.g. \begin{aligned} inside \left[\begin{aligned}...\end{aligned}\right]).
        // This fix manually scans the input, counting \begin{name}/\end{name} depth,
        // and only stops when depth returns to zero — i.e. the truly matching \end.
        let (mut inner_str, is_closed) = {
            let full = *input;
            let mut depth = 1usize;
            let mut scanned = 0usize;
            let mut remaining = full;
            let mut matched_end_pos: Option<usize> = None;

            'scan: loop {
                let next_begin = remaining.find(begin_pattern.as_str());
                let next_end = remaining.find(end_pattern.as_str());

                match (next_begin, next_end) {
                    (_, None) => {
                        // No \end{name} found anywhere: unclosed environment
                        break 'scan;
                    }
                    (Some(b), Some(e)) if b < e => {
                        // A nested \begin{name} appears before the next \end{name}: push depth
                        depth += 1;
                        let skip = b + begin_pattern.len();
                        scanned += skip;
                        remaining = &remaining[skip..];
                    }
                    (_, Some(e)) => {
                        // \end{name} comes next (or no \begin{name} left): pop depth
                        depth -= 1;
                        if depth == 0 {
                            // Found the truly matching \end{name}
                            matched_end_pos = Some(scanned + e);
                            break 'scan;
                        }
                        let skip = e + end_pattern.len();
                        scanned += skip;
                        remaining = &remaining[skip..];
                    }
                }
            }

            if let Some(end_pos) = matched_end_pos {
                let inner = &full[..end_pos];
                *input = &full[end_pos + end_pattern.len()..];
                (inner, true)
            } else {
                // Unclosed: consume all remaining input
                let inner = full;
                *input = &full[full.len()..];
                (inner, false)
            }
        };

        let mut parse_cells_in_row = |row_input: &mut &'s str| -> ModalResult<Vec<MathNode>> {
            separated(
                0..,
                delimited(space0, parse_row, space0),
                (space0, '&', space0),
            )
            .parse_next(row_input)
        };

        // 指定类型为 ModalResult<Option<&str>>
        let mut parse_newline_opt = |input: &mut &'s str| -> ModalResult<Option<&str>> {
            preceded(
                literal("\\\\"),
                opt(delimited((space0, '['), take_until(0.., "]"), ']')),
            )
            .parse_next(input)
        };

        let mut rows: Vec<(Vec<MathNode>, Option<String>)> = Vec::new();

        loop {
            let _ = space0.parse_next(&mut inner_str)?;
            if inner_str.is_empty() {
                break;
            }
            // Fix 2: zero-progress guard — record position after consuming leading whitespace.
            // parse_cells_in_row uses separated(0..) and parse_row uses repeat(0..),
            // both of which always succeed even when consuming zero bytes. If a character
            // cannot be parsed as any atom (e.g. a stuck \left[...] after body truncation),
            // the loop would spin forever without this guard.
            let progress_mark = inner_str.len();

            if let Ok(cells) = parse_cells_in_row.parse_next(&mut inner_str) {
                let spacing = if let Ok(opt_spacing) = parse_newline_opt.parse_next(&mut inner_str)
                {
                    opt_spacing.map(|s: &str| s.to_string())
                } else {
                    None
                };

                let is_empty_row = cells.len() == 1
                    && match &cells[0] {
                        MathNode::Row(nodes) => nodes.is_empty(),
                        _ => false,
                    };

                // Ignore trailing empty rows caused by a final `\\` before `\end`
                if is_empty_row
                    && spacing.is_none()
                    && inner_str.trim().is_empty()
                    && !rows.is_empty()
                {
                    // Do not push this empty row
                } else {
                    rows.push((cells, spacing));
                }
            } else {
                // Should not happen as separated(0..) always matches something, but fallback
                break;
            }

            if inner_str.is_empty() {
                break;
            }
            // Zero-progress guard: if neither parse_cells_in_row nor parse_newline_opt
            // consumed any input, there is an irrecoverable stuck token — exit cleanly.
            if inner_str.len() == progress_mark {
                break;
            }
        }

        let env_node = MathNode::Environment {
            name: name.clone(),
            format,
            rows,
        };

        if !is_closed {
            Ok(MathNode::Row(vec![
                env_node,
                MathNode::Error(format!("Missing \\end{{{}}}", name)),
            ]))
        } else {
            Ok(env_node)
        }
    })
    .parse_next(input)
}

/// Parses an atomic mathematical element, which can be a number, identifier, operator, group, or command.
pub fn parse_atom<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace(
        "parse_atom",
        alt((
            parse_environment, // 环境优先级最高
            parse_left_right,
            parse_fraction,
            parse_sqrt,
            parse_group,
            parse_command, // 将通用命令解析器加入！
            parse_ident,
            parse_number,
            parse_operator, // 允许单字符操作符作为 atom（例如为它添加上下标 V^* 或 \lim_{x \to 0}^+）
            parse_fallback_char, // 允许原生 Unicode 字符（如 • 或 α）
        )),
    )
    .parse_next(input)
}

/// Fallback for raw Unicode characters not explicitly matched as operators or identifiers.
pub fn parse_fallback_char<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace(
        "parse_fallback_char",
        winnow::token::one_of(|c: char| !c.is_ascii_whitespace() && !"\\{}_^&%$#~".contains(c))
            .map(|c: char| {
                if c.is_alphabetic() {
                    MathNode::Identifier(c.to_string())
                } else if c.is_numeric() {
                    MathNode::Number(c.to_string())
                } else {
                    MathNode::Operator(c.to_string())
                }
            }),
    )
    .parse_next(input)
}

/// Parses subscripts (`_`) and superscripts (`^`) attached to a base mathematical element.
pub fn parse_script<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace("parse_script", |input: &mut &'s str| {
        let base = match parse_atom.parse_next(input) {
            Ok(b) => b,
            Err(e) => {
                let next_char = input.chars().next();
                if next_char == Some('_') || next_char == Some('^') || next_char == Some('\'') {
                    MathNode::Row(vec![])
                } else {
                    return Err(e);
                }
            }
        };

        // 探测 base 之后是否紧跟 \limits 或 \nolimits (这些通常用于覆盖默认的上下标排版)
        let behavior = if let Ok(_) =
            literal::<&str, &str, winnow::error::ContextError>("\\limits").parse_next(input)
        {
            LimitBehavior::Limits
        } else if let Ok(_) =
            literal::<&str, &str, winnow::error::ContextError>("\\nolimits").parse_next(input)
        {
            LimitBehavior::NoLimits
        } else {
            LimitBehavior::Default
        };

        let mut sub = None;
        let mut sup = None;

        // == 新增：撇号（prime）支持 ==
        // x' 和 x^{\prime} 等价；x'' 对应 x^{\prime\prime}（即双撇线 ″）
        let mut prime_count = 0usize;
        while opt(one_of::<_, _, winnow::error::ContextError>('\''))
            .parse_next(input)?
            .is_some()
        {
            prime_count += 1;
        }
        if prime_count > 0 {
            let prime_char = match prime_count {
                1 => "\u{2032}", // ′
                2 => "\u{2033}", // ″
                3 => "\u{2034}", // ‴
                _ => "\u{2057}", // ⁗ (4重撇及以上)
            };
            sup = Some(MathNode::Identifier(prime_char.to_string()));
        }

        loop {
            if sup.is_none() {
                if let Some(s) =
                    opt(preceded((space0, '^', space0), parse_atom)).parse_next(input)?
                {
                    sup = Some(s);
                    continue;
                }
            }
            if sub.is_none() {
                if let Some(s) =
                    opt(preceded((space0, '_', space0), parse_atom)).parse_next(input)?
                {
                    sub = Some(s);
                    continue;
                }
            }
            break;
        }

        // 判断 base 是否是要求使用 limits 渲染的大运算符或极限函数
        let is_large_operator = match &base {
            MathNode::Operator(op) => crate::symbols::is_large_op_symbol(op),
            MathNode::Function(f) => crate::symbols::is_large_math_function(f),
            MathNode::StretchOp { .. } => true, // 拉伸修饰符（underbrace 等）把附着物当 limits
            _ => false,
        };

        // 如果没有显式指定 \limits，且它是积分符号，我们覆盖为 NoLimits 行为 (右下/右上角标)
        // 除非用户显式写了 \limits，则保留 LimitBehavior::Limits。
        let final_behavior = if behavior == LimitBehavior::Default {
            match &base {
                MathNode::Operator(op) if crate::symbols::is_integral_symbol(op) => {
                    LimitBehavior::NoLimits
                }
                _ => behavior,
            }
        } else {
            behavior
        };

        if sub.is_none() && sup.is_none() && final_behavior == LimitBehavior::Default {
            return Ok(base);
        }

        Ok(MathNode::Scripts {
            base: Box::new(base),
            sub: sub.map(Box::new),
            sup: sup.map(Box::new),
            behavior: final_behavior,
            is_large_op: is_large_operator,
            pre_sub: None,
            pre_sup: None,
        })
    })
    .parse_next(input)
}

/// The main parser for a single mathematical node, handling scripts, atoms, and other constructs.
pub fn parse_math<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    let mut rows: Vec<(Vec<MathNode>, Option<String>)> = Vec::new();

    loop {
        let _ = space0.parse_next(input)?;
        if input.is_empty() {
            break;
        }
        // Fix 2 (parse_math): same zero-progress guard as parse_environment.
        let progress_mark = input.len();

        if let Ok(cells) = parse_cells_in_row.parse_next(input) {
            let spacing = if let Ok(opt_spacing) = parse_newline_opt.parse_next(input) {
                opt_spacing.map(|s: &str| s.to_string())
            } else {
                None
            };

            let is_empty_row = cells.len() == 1
                && match &cells[0] {
                    MathNode::Row(nodes) => nodes.is_empty(),
                    _ => false,
                };

            if is_empty_row && spacing.is_none() && input.trim().is_empty() && !rows.is_empty() {
                // Ignore trailing empty row
            } else {
                rows.push((cells, spacing));
            }
        } else {
            break;
        }

        if input.is_empty() {
            break;
        }
        // If no input was consumed in this iteration, break to avoid infinite loop
        if input.len() == progress_mark {
            break;
        }
    }

    if rows.len() == 1 && rows[0].0.len() == 1 {
        // Only 1 row and 1 cell
        let (mut row_cells, _) = rows.into_iter().next().unwrap();
        Ok(row_cells.remove(0))
    } else {
        // Multi-line or aligned top-level expression
        Ok(MathNode::Environment {
            name: "align*".to_string(), // use align* to support alternating right/left alignments without labels
            format: None,
            rows,
        })
    }
}

pub fn parse_node<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace("parse_node", alt((parse_script, parse_operator))).parse_next(input)
}

/// Parses a sequence (row) of mathematical nodes, optionally separated by spaces.
pub fn parse_row<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace(
        "parse_row",
        repeat(0.., preceded(space0, parse_node)).map(|nodes: Vec<MathNode>| {
            // == AST 智能折叠 Pass: 张量与前置角标 ==
            // 扫描平铺的数组，寻找: [Scripts(base: 空Row, sub: A, sup: B), Identifier(X)]
            // 并将它们合并为: Scripts(base: X, pre_sub: A, pre_sup: B)
            let mut folded_nodes: Vec<MathNode> = Vec::with_capacity(nodes.len());
            let mut i = 0;

            while i < nodes.len() {
                if i + 1 < nodes.len() {
                    // 检查当前节点是否是一个空基底的角标
                    if let MathNode::Scripts {
                        base,
                        sub,
                        sup,
                        pre_sub: None,
                        pre_sup: None,
                        behavior: LimitBehavior::Default,
                        ..
                    } = &nodes[i]
                    {
                        if let MathNode::Row(inner) = &**base {
                            if inner.is_empty() {
                                // 发现了一个完美的前置角标载体！
                                let next_node = nodes[i + 1].clone();

                                // 取出下一个节点。如果下一个节点本身也是一个 Scripts 节点，
                                // 我们就把前置角标合并进它的 pre_sub/pre_sup 里！
                                let merged_node = match next_node {
                                    MathNode::Scripts {
                                        base: next_base,
                                        sub: next_sub,
                                        sup: next_sup,
                                        behavior,
                                        is_large_op,
                                        ..
                                    } => MathNode::Scripts {
                                        base: next_base,
                                        sub: next_sub,
                                        sup: next_sup,
                                        pre_sub: sub.clone(),
                                        pre_sup: sup.clone(),
                                        behavior,
                                        is_large_op,
                                    },
                                    // 如果下一个只是个普通的原子 (比如 Identifier X)，包装它！
                                    _ => MathNode::Scripts {
                                        base: Box::new(next_node),
                                        sub: None,
                                        sup: None,
                                        pre_sub: sub.clone(),
                                        pre_sup: sup.clone(),
                                        behavior: LimitBehavior::Default,
                                        is_large_op: false,
                                    },
                                };

                                folded_nodes.push(merged_node);
                                i += 2; // 跳过这两个被合并的节点
                                continue;
                            }
                        }
                    }
                }
                folded_nodes.push(nodes[i].clone());
                i += 1;
            }

            if folded_nodes.len() == 1 {
                folded_nodes.into_iter().next().unwrap()
            } else {
                MathNode::Row(folded_nodes)
            }
        }),
    )
    .parse_next(input)
}

// ==========================================
// 3. 代码生成器 (AST -> MathML)
// ==========================================

fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\'', "&apos;")
        .replace('\"', "&quot;")
}

// ==========================================
// 3. 抽象渲染后端 (Pluggable Backends)
// ==========================================

/// An abstract backend interface for rendering a `MathNode` abstract syntax tree into an output format.
///
/// All backend renderers must implement this trait.
pub trait MathRenderer {
    fn render(&self, node: &MathNode, mode: RenderMode) -> String;
}

// ==========================================
// 4. 标准 MathML 渲染器实现
// ==========================================

/// The standard MathML rendering backend provided by tex2math.
///
/// Converts a `MathNode` AST into a MathML XML string.
pub struct MathMLRenderer;

impl MathMLRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl MathRenderer for MathMLRenderer {
    fn render(&self, node: &MathNode, mode: RenderMode) -> String {
        match node {
            MathNode::Number(n) => format!("<mn>{}</mn>", escape_xml(n)),
            MathNode::Identifier(i) => format!("<mi>{}</mi>", escape_xml(i)),
            MathNode::Operator(o) => {
                let is_arrow = [
                    "\u{2190}", "\u{2192}", "\u{2194}", "\u{21D2}", "\u{21D0}", "\u{21D4}",
                    "\u{21A6}", "\u{21A9}", "\u{21AA}", "\u{219E}", "\u{21A0}",
                ]
                .contains(&o.as_str());
                if is_arrow {
                    format!("<mo stretchy=\"true\">{}</mo>", escape_xml(o))
                } else {
                    format!("<mo>{}</mo>", escape_xml(o))
                }
            }
            MathNode::Fraction(num, den) => {
                format!(
                    "<mfrac>{}{}</mfrac>",
                    self.render(num, mode),
                    self.render(den, mode)
                )
            }
            MathNode::Scripts {
                base,
                sub,
                sup,
                pre_sub,
                pre_sup,
                behavior,
                is_large_op,
            } => {
                let base_str = self.render(base, mode);

                if pre_sub.is_some() || pre_sup.is_some() {
                    let s_sub = sub
                        .as_ref()
                        .map(|s| self.render(s, mode))
                        .unwrap_or_else(|| "<none/>".to_string());
                    let s_sup = sup
                        .as_ref()
                        .map(|s| self.render(s, mode))
                        .unwrap_or_else(|| "<none/>".to_string());
                    let p_sub = pre_sub
                        .as_ref()
                        .map(|s| self.render(s, mode))
                        .unwrap_or_else(|| "<none/>".to_string());
                    let p_sup = pre_sup
                        .as_ref()
                        .map(|s| self.render(s, mode))
                        .unwrap_or_else(|| "<none/>".to_string());

                    return format!(
                        "<mmultiscripts>{}{}{}<mprescripts/>{}{}</mmultiscripts>",
                        base_str, s_sub, s_sup, p_sub, p_sup
                    );
                }

                let sub_str = sub.as_ref().map(|s| self.render(s, mode));
                let sup_str = sup.as_ref().map(|s| self.render(s, mode));

                let render_as_limits = match behavior {
                    LimitBehavior::Limits => true,
                    LimitBehavior::NoLimits => false,
                    LimitBehavior::Default => *is_large_op && mode == RenderMode::Display,
                };

                match (render_as_limits, sub_str, sup_str) {
                    (true, Some(sub), Some(sup)) => {
                        format!("<munderover>{}{}{}</munderover>", base_str, sub, sup)
                    }
                    (true, Some(sub), None) => format!("<munder>{}{}</munder>", base_str, sub),
                    (true, None, Some(sup)) => format!("<mover>{}{}</mover>", base_str, sup),

                    (false, Some(sub), Some(sup)) => {
                        format!("<msubsup>{}{}{}</msubsup>", base_str, sub, sup)
                    }
                    (false, Some(sub), None) => format!("<msub>{}{}</msub>", base_str, sub),
                    (false, None, Some(sup)) => format!("<msup>{}{}</msup>", base_str, sup),

                    (_, None, None) => base_str,
                }
            }
            MathNode::Row(nodes) => {
                let inner: String = nodes.iter().map(|n| self.render(n, mode)).collect();
                format!("<mrow>{}</mrow>", inner)
            }
            MathNode::Sqrt(content) => {
                format!("<msqrt>{}</msqrt>", self.render(content, mode))
            }
            MathNode::Root { index, content } => {
                format!(
                    "<mroot>{}{}</mroot>",
                    self.render(content, mode),
                    self.render(index, mode)
                )
            }
            MathNode::Fenced {
                open,
                content,
                close,
            } => {
                let mo_open = if open == "." {
                    String::new()
                } else {
                    format!("<mo stretchy=\"true\">{}</mo>", escape_xml(open))
                };
                let mo_close = if close == "." {
                    String::new()
                } else {
                    format!("<mo stretchy=\"true\">{}</mo>", escape_xml(close))
                };
                // Wrap the rendered content in an inner <mrow> to prevent baseline shift
                // issues in WebKit/Blink when a stretchy fence immediately follows a <msup>/<msub>.
                format!(
                    "<mrow>{}<mrow>{}</mrow>{}</mrow>",
                    mo_open,
                    self.render(content, mode),
                    mo_close
                )
            }
            MathNode::Environment { name, format, rows } => {
                let mut custom_aligns = Vec::new();
                let mut custom_lines = Vec::new();

                if let Some(fmt_str) = format {
                    // 正确算法：跟踪列对齐和列间分隔符
                    // MathML columnlines 需要 N-1 个条目（N 为列数）
                    // 每个分隔符属于其左侧列的右边
                    let mut pending_sep = "none"; // 待添加的分隔符（在看到下一列字符时提交）
                    let mut separators: Vec<&str> = Vec::new();
                    for c in fmt_str.chars() {
                        match c {
                            'l' | 'c' | 'r' => {
                                let align = match c {
                                    'l' => "left",
                                    'c' => "center",
                                    _ => "right",
                                };
                                custom_aligns.push(align);
                                if !custom_aligns.is_empty() && custom_aligns.len() > 1 {
                                    // 不是第一列，提交上一列的右侧分隔符
                                    separators.push(pending_sep);
                                }
                                pending_sep = "none";
                            }
                            '|' => {
                                // |是当前列左边的分隔符（对下一列来说是左侧分隔）
                                pending_sep = "solid";
                            }
                            _ => {}
                        }
                    }
                    custom_lines = separators;
                }

                let table_attr = match name.as_str() {
                    "align" | "align*" | "eqnarray" | "eqnarray*" => {
                        let max_cols = rows.iter().map(|(r, _)| r.len()).max().unwrap_or(0);
                        let aligns: Vec<&str> = (0..max_cols)
                            .map(|i| if i % 2 == 0 { "right" } else { "left" })
                            .collect();
                        if aligns.is_empty() {
                            String::new()
                        } else {
                            format!(" columnalign=\"{}\"", aligns.join(" "))
                        }
                    }
                    "cases" => " columnalign=\"left\"".to_string(),
                    "array" => {
                        let mut attr = String::new();
                        if !custom_aligns.is_empty() {
                            attr.push_str(&format!(" columnalign=\"{}\"", custom_aligns.join(" ")));
                        }
                        // columnlines 数量应为 N-1（匹配列间分隔符数）
                        if custom_lines.contains(&"solid") {
                            attr.push_str(&format!(" columnlines=\"{}\"", custom_lines.join(" ")));
                        }
                        attr
                    }
                    _ => "".to_string(), // 默认居中 (matrix 等)
                };

                let mut table_xml = format!("<mtable{}>", table_attr);
                for (row, spacing) in rows {
                    let tr_attr = if let Some(space) = spacing {
                        format!(" style=\"margin-bottom: {};\"", escape_xml(space))
                    } else {
                        "".to_string()
                    };

                    table_xml.push_str(&format!("<mtr{}>", tr_attr));
                    for cell in row {
                        table_xml.push_str(&format!("<mtd>{}</mtd>", self.render(cell, mode)));
                    }
                    table_xml.push_str("</mtr>");
                }
                table_xml.push_str("</mtable>");

                match name.as_str() {
                    "pmatrix" => format!(
                        "<mrow><mo stretchy=\"true\">(</mo>{}<mo stretchy=\"true\">)</mo></mrow>",
                        table_xml
                    ),
                    "bmatrix" => format!(
                        "<mrow><mo stretchy=\"true\">[</mo>{}<mo stretchy=\"true\">]</mo></mrow>",
                        table_xml
                    ),
                    "Bmatrix" => format!(
                        "<mrow><mo stretchy=\"true\">{{</mo>{}<mo stretchy=\"true\">}}</mo></mrow>",
                        table_xml
                    ),
                    "vmatrix" => format!(
                        "<mrow><mo stretchy=\"true\">|</mo>{}<mo stretchy=\"true\">|</mo></mrow>",
                        table_xml
                    ),
                    "Vmatrix" => format!(
                        "<mrow><mo stretchy=\"true\">‖</mo>{}<mo stretchy=\"true\">‖</mo></mrow>",
                        table_xml
                    ),
                    "cases" => format!("<mrow><mo stretchy=\"true\">{{</mo>{}</mrow>", table_xml),
                    _ => table_xml,
                }
            }
            MathNode::Text(t) => format!("<mtext>{}</mtext>", escape_xml(t)),
            MathNode::Style { variant, content } => {
                if variant == "vphantom" {
                    // \vphantom: Height of the content, but zero width.
                    // <mphantom> makes it invisible but takes up full space.
                    // <mpadded width="0px"> makes its width zero.
                    format!(
                        "<mpadded width=\"0px\"><mphantom>{}</mphantom></mpadded>",
                        self.render(content, mode)
                    )
                } else if variant == "hphantom" {
                    // \hphantom: Width of the content, but zero height and depth.
                    format!(
                        "<mpadded height=\"0px\" depth=\"0px\"><mphantom>{}</mphantom></mpadded>",
                        self.render(content, mode)
                    )
                } else {
                    format!(
                        "<mstyle mathvariant=\"{}\">{}</mstyle>",
                        escape_xml(variant),
                        self.render(content, mode)
                    )
                }
            }
            MathNode::Accent { mark, content } => {
                format!(
                    "<mover accent=\"true\">{}<mo>{}</mo></mover>",
                    self.render(content, mode),
                    escape_xml(mark)
                )
            }
            MathNode::Function(f) => {
                let func_text = match f.as_str() {
                    "injlim" => "inj lim",
                    "projlim" => "proj lim",
                    _ => f.as_str(),
                };
                format!("<mi mathvariant=\"normal\">{}</mi>", escape_xml(func_text))
            }
            MathNode::OperatorName(content) => {
                // Do not use <mi> as a wrapper for complex layouts like <munder>,
                // because browsers (like Chrome/Safari) flatten or break layout elements inside token elements (<mi>, <mo>, <mn>).
                // Instead, use <mstyle mathvariant="normal"> wrapped in an <mrow>.
                format!(
                    "<mrow><mstyle mathvariant=\"normal\">{}</mstyle></mrow>",
                    self.render(content, mode)
                )
            }
            MathNode::SizedDelimiter { size, delim } => {
                // LaTeX \big, \Big, \bigg, \Bigg correspond to increasing sizes.
                // We use minsize and maxsize to force stretching to that exact size.
                format!(
                    "<mo minsize=\"{}\" maxsize=\"{}\">{}</mo>",
                    escape_xml(size),
                    escape_xml(size),
                    escape_xml(delim)
                )
            }
            MathNode::Space(width) => format!("<mspace width=\"{}\"/>", escape_xml(width)),
            MathNode::NewLine => "<mspace linebreak=\"newline\"/>".to_string(),

            MathNode::Color { color, content } => {
                format!(
                    "<mstyle mathcolor=\"{}\">{}</mstyle>",
                    escape_xml(color),
                    self.render(content, mode)
                )
            }
            MathNode::ColorBox { bg_color, content } => {
                format!(
                    "<mstyle mathbackground=\"{}\">{}</mstyle>",
                    escape_xml(bg_color),
                    self.render(content, mode)
                )
            }
            MathNode::Boxed(content) => {
                format!(
                    "<menclose notation=\"box\">{}</menclose>",
                    self.render(content, mode)
                )
            }
            MathNode::Phantom { kind, content } => {
                let rendered = self.render(content, mode);
                match kind {
                    PhantomKind::Invisible => format!("<mphantom>{}</mphantom>", rendered),
                    PhantomKind::Vertical => format!(
                        "<mpadded width=\"0px\"><mphantom>{}</mphantom></mpadded>",
                        rendered
                    ),
                    PhantomKind::Horizontal => format!(
                        "<mpadded height=\"0px\" depth=\"0px\"><mphantom>{}</mphantom></mpadded>",
                        rendered
                    ),
                }
            }
            MathNode::Cancel {
                mode: notation_mode,
                content,
            } => {
                format!(
                    "<menclose notation=\"{}\">{}</menclose>",
                    escape_xml(notation_mode),
                    self.render(content, mode)
                )
            }
            MathNode::StretchOp {
                op,
                is_over,
                content,
            } => {
                let stretchy_op = format!("<mo stretchy=\"true\">{}</mo>", escape_xml(op));
                let content_xml = self.render(content, mode);
                if *is_over {
                    format!("<mover>{}{}</mover>", content_xml, stretchy_op)
                } else {
                    format!("<munder>{}{}</munder>", content_xml, stretchy_op)
                }
            }
            MathNode::StyledMath {
                displaystyle,
                content,
            } => {
                // \dfrac → displaystyle="true"，\tfrac → displaystyle="false"
                let ds = if *displaystyle { "true" } else { "false" };
                format!(
                    "<mstyle displaystyle=\"{}\">{}</mstyle>",
                    ds,
                    self.render(content, mode)
                )
            }
            MathNode::Error(err_msg) => {
                format!(
                    "<merror><mtext mathcolor=\"red\">Syntax Error: {}</mtext></merror>",
                    escape_xml(err_msg)
                )
            }
        }
    }
}

// ==========================================

#[cfg(test)]
mod tests;

/// A convenience function to generate MathML from a `MathNode` AST directly.
///
/// This uses the `MathMLRenderer` under the hood to perform the translation.
/// Provides a simple, standard interface for backward compatibility.
///
/// # Arguments
/// * `node` - The root `MathNode` of the parsed formula.
/// * `mode` - The `RenderMode` (Inline or Display) determining layout rules.
///
/// # Returns
/// A `String` containing the generated MathML XML.
pub fn generate_mathml(node: &MathNode, mode: RenderMode) -> String {
    MathMLRenderer::new().render(node, mode)
}

#[cfg(feature = "wasm-bindgen")]
pub mod wasm;
