use winnow::ascii::{alpha1, digit1, multispace0 as space0};
use winnow::combinator::{alt, delimited, opt, preceded, repeat, separated, trace};
use winnow::prelude::*;
use winnow::token::{literal, one_of, take_till, take_until};

use crate::ast::*;

/// Parses a numeric literal (e.g., `123`, `3.14`, `.14`, `10.`) into a `MathNode::Number`.
pub fn parse_number<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace(
        "parse_number",
        alt((
            // Format 1: Normal decimal or integer (e.g., "123", "3.14", "10.")
            (digit1, opt(('.', opt(digit1)))).take(),
            // Format 2: Leading decimal (e.g., ".14")
            ('.', digit1).take(),
        ))
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
        let _ = space0.parse_next(input)?;

        let mut index_node_opt = None;

        if opt(literal::<&str, &str, winnow::error::ContextError>("["))
            .parse_next(input)?
            .is_some()
        {
            let mut index_nodes = Vec::new();
            loop {
                let _ = space0.parse_next(input)?;
                if input.is_empty() || input.starts_with(']') {
                    break;
                }

                let progress = input.len();
                if let Ok(node) = parse_node.parse_next(input) {
                    index_nodes.push(node);
                } else {
                    break;
                }
                if input.len() == progress {
                    break;
                }
            }

            // 消耗右侧的 ']'
            let _ =
                opt(literal::<&str, &str, winnow::error::ContextError>("]")).parse_next(input)?;

            // 使用复用的 AST 折叠逻辑
            index_node_opt = Some(fold_row_nodes(index_nodes));
        }

        let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;

        if let Some(index_node) = index_node_opt {
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
///// Parses a general LaTeX command starting with a backslash (e.g., `\alpha`, `\int`, `\mathbf{x}`).

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
        })
    } else {
        Ok(MathNode::OperatorName(Box::new(content)))
    }
}

fn parse_not_modifier_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    let _ = space0.parse_next(input)?;
    let checkpoint = *input;

    if let Ok(next_cmd) =
        preceded::<_, _, _, winnow::error::ContextError, _, _>('\\', alpha1).parse_next(input)
    {
        let negated_opt = match next_cmd {
            "in" | "isin" => Some("\u{2209}"),
            "ni" | "owns" => Some("\u{220C}"),
            "subset" => Some("\u{2284}"),
            "supset" => Some("\u{2285}"),
            "subseteq" => Some("\u{2288}"),
            "supseteq" => Some("\u{2289}"),
            "sim" => Some("\u{2241}"),
            "approx" => Some("\u{2249}"),
            "equiv" => Some("\u{2262}"),
            "parallel" => Some("\u{2226}"),
            "mid" => Some("\u{2224}"),
            "vdash" => Some("\u{22AC}"),
            "prec" => Some("\u{2280}"),
            "succ" => Some("\u{2281}"),
            "le" | "leq" => Some("\u{2270}"),
            "ge" | "geq" => Some("\u{2271}"),
            "leftarrow" => Some("\u{219A}"),
            "rightarrow" => Some("\u{219B}"),
            _ => None,
        };

        if let Some(negated) = negated_opt {
            return Ok(MathNode::Operator(negated.to_string()));
        }

        // 恢复游标，允许后续解析器正常处理未知命令
        *input = checkpoint;
    } else if opt(one_of::<_, _, winnow::error::ContextError>('='))
        .parse_next(input)?
        .is_some()
    {
        return Ok(MathNode::Operator("\u{2260}".to_string()));
    }

    // 回退处理：正常解析下一个原子，并在其后跟随 U+0338 COMBINING LONG SOLIDUS OVERLAY
    if let Ok(next_node) = parse_atom.parse_next(input) {
        Ok(MathNode::Row(vec![
            next_node,
            MathNode::Operator("\u{0338}".to_string()),
        ]))
    } else {
        Ok(MathNode::Operator("\u{0338}".to_string()))
    }
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

fn parse_special_limit_arrow_cmd(cmd: &str) -> ModalResult<MathNode> {
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
    })
}

fn parse_var_greek_cmd(cmd: &str) -> ModalResult<MathNode> {
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

fn parse_spacing_cmd(cmd: &str) -> ModalResult<MathNode> {
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
                .take(),
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
///// Parses a LaTeX environment enclosed in `\begin{env}` and `\end{env}` (e.g., `matrix`, `cases`).
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

pub fn fold_row_nodes(nodes: Vec<MathNode>) -> MathNode {
    // == AST 智能折叠 Pass: 张量与前置角标 ==
    let mut folded_nodes: Vec<MathNode> = Vec::with_capacity(nodes.len());
    let mut i = 0;

    while i < nodes.len() {
        if i + 1 < nodes.len() {
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
                        let next_node = nodes[i + 1].clone();
                        let merged_node = match next_node {
                            MathNode::Scripts {
                                base: next_base,
                                sub: next_sub,
                                sup: next_sup,
                                behavior,
                                ..
                            } => MathNode::Scripts {
                                base: next_base,
                                sub: next_sub,
                                sup: next_sup,
                                pre_sub: sub.clone(),
                                pre_sup: sup.clone(),
                                behavior,
                            },
                            _ => MathNode::Scripts {
                                base: Box::new(next_node),
                                sub: None,
                                sup: None,
                                pre_sub: sub.clone(),
                                pre_sup: sup.clone(),
                                behavior: LimitBehavior::Default,
                            },
                        };

                        folded_nodes.push(merged_node);
                        i += 2;
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
}

pub fn parse_row<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace(
        "parse_row",
        repeat(0.., preceded(space0, parse_node)).map(fold_row_nodes),
    )
    .parse_next(input)
}
