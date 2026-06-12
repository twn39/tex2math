use winnow::ascii::{alpha1, multispace0 as space0};
use winnow::combinator::{alt, delimited, opt, preceded, repeat, trace};
use winnow::prelude::*;
use winnow::token::one_of;

use super::atoms::{take_balanced_braces, take_balanced_brackets};
use super::{parse_atom, parse_fence_delim, parse_node, parse_row};
use crate::ast::*;

fn parse_text_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    let (inner_text, is_closed) = take_balanced_braces(input)?;
    let text_node = MathNode::Text(inner_text.to_string());
    if is_closed {
        Ok(text_node)
    } else {
        Ok(MathNode::Row(vec![
            text_node,
            MathNode::Error("Missing '}' in text command".to_string()),
        ]))
    }
}

fn parse_color_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode> {
    match cmd {
        "textcolor" => {
            let (color, _) = take_balanced_braces(input)?;
            let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
            Ok(MathNode::Color {
                color: color.to_string(),
                content: Box::new(content),
            })
        }
        "colorbox" => {
            let (bg_color, _) = take_balanced_braces(input)?;
            let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
            Ok(MathNode::ColorBox {
                bg_color: bg_color.to_string(),
                content: Box::new(content),
            })
        }
        "color" => {
            let (color, _) = take_balanced_braces(input)?;
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

    let sub_str_opt = opt(take_balanced_brackets.map(|(content, _)| content)).parse_next(input)?;
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
