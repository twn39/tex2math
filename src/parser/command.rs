use std::borrow::Cow;
use winnow::ascii::{alpha1, multispace0 as space0};
use winnow::combinator::{alt, delimited, opt, preceded, repeat, trace};
use winnow::prelude::*;
use winnow::token::one_of;

use super::atoms::{take_balanced_braces, take_balanced_brackets};
use super::{parse_atom, parse_fence_delim, parse_node, parse_row};
use crate::ast::*;

fn parse_text_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let (inner_text, is_closed) = take_balanced_braces(input)?;
    let text_node = MathNode::Text(Cow::Borrowed(inner_text));
    if is_closed {
        Ok(text_node)
    } else {
        Ok(MathNode::Row(vec![
            text_node,
            MathNode::Error(Cow::Borrowed("Missing '}' in text command")),
        ]))
    }
}

fn parse_color_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    match cmd {
        "textcolor" => {
            let (color, _) = take_balanced_braces(input)?;
            let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
            Ok(MathNode::Color {
                color: Cow::Borrowed(color),
                content: Box::new(content),
            })
        }
        "colorbox" => {
            let (bg_color, _) = take_balanced_braces(input)?;
            let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
            Ok(MathNode::ColorBox {
                bg_color: Cow::Borrowed(bg_color),
                content: Box::new(content),
            })
        }
        "color" => {
            let (color, _) = take_balanced_braces(input)?;
            let remaining_nodes: Vec<MathNode<'s>> =
                repeat(0.., preceded(space0, parse_node)).parse_next(input)?;
            let content = if remaining_nodes.len() == 1 {
                remaining_nodes.into_iter().next().unwrap()
            } else {
                MathNode::Row(remaining_nodes)
            };
            Ok(MathNode::Color {
                color: Cow::Borrowed(color),
                content: Box::new(content),
            })
        }
        _ => unreachable!(),
    }
}

fn parse_boxed_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
    Ok(MathNode::Boxed(Box::new(content)))
}

fn extract_scripts<'s>(
    node: &MathNode<'s>,
) -> (Option<Box<MathNode<'s>>>, Option<Box<MathNode<'s>>>) {
    match node {
        MathNode::Scripts { sub, sup, .. } => (sub.clone(), sup.clone()),
        MathNode::Row(nodes) if nodes.len() == 1 => extract_scripts(&nodes[0]),
        _ => (None, None),
    }
}

fn parse_sideset_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let left_scripts = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
    let right_scripts = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;

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

fn parse_over_under_set_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode<'s>> {
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

fn parse_phantom_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
    let kind = match cmd {
        "phantom" => PhantomKind::Invisible,
        "vphantom" => PhantomKind::Vertical,
        "hphantom" => PhantomKind::Horizontal,
        _ => unreachable!("phantom pre-filtered"),
    };
    Ok(MathNode::Phantom {
        kind,
        content: Box::new(content),
    })
}

fn parse_cancel_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let mode =
        crate::registry::lookup(crate::registry::CANCEL_MODES, cmd).expect("cancel pre-filtered");
    let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
    Ok(MathNode::Cancel {
        mode: Cow::Borrowed(mode),
        content: Box::new(content),
    })
}

fn parse_extensible_arrow_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let arrow_char = crate::registry::lookup(crate::registry::EXTENSIBLE_ARROWS, cmd)
        .expect("extensible arrow pre-filtered");

    let sub_str_opt = opt(take_balanced_brackets.map(|(content, _)| content)).parse_next(input)?;
    let sup = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;

    let sub = if let Some(mut s) = sub_str_opt {
        Some(Box::new(parse_row.parse_next(&mut s)?))
    } else {
        None
    };

    Ok(MathNode::Scripts {
        base: Box::new(MathNode::Operator(Cow::Borrowed(arrow_char))),
        sub,
        sup: Some(Box::new(sup)),
        pre_sub: None,
        pre_sup: None,
        behavior: LimitBehavior::Limits,
    })
}

fn parse_stretch_modifier_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let (op_str, is_over) = crate::registry::lookup_stretch(cmd).expect("stretch op pre-filtered");
    let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
    Ok(MathNode::StretchOp {
        op: Cow::Borrowed(op_str),
        is_over,
        content: Box::new(content),
    })
}

fn parse_frac_style_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let ds = crate::registry::lookup_bool(crate::registry::FRAC_STYLES, cmd)
        .expect("frac style pre-filtered");
    let num = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
    let den = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
    Ok(MathNode::StyledMath {
        displaystyle: ds,
        content: Box::new(MathNode::Fraction(Box::new(num), Box::new(den))),
    })
}

fn parse_operatorname_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode<'s>> {
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

fn parse_not_modifier_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
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
            return Ok(MathNode::Operator(Cow::Borrowed(negated)));
        }

        // 恢复游标，允许后续解析器正常处理未知命令
        *input = checkpoint;
    } else if opt(one_of::<_, _, winnow::error::ContextError>('='))
        .parse_next(input)?
        .is_some()
    {
        return Ok(MathNode::Operator(Cow::Borrowed("\u{2260}")));
    }

    // 回退处理：正常解析下一个原子，并在其后跟随 U+0338 COMBINING LONG SOLIDUS OVERLAY
    if let Ok(next_node) = parse_atom.parse_next(input) {
        Ok(MathNode::Row(vec![
            next_node,
            MathNode::Operator(Cow::Borrowed("\u{0338}")),
        ]))
    } else {
        Ok(MathNode::Operator(Cow::Borrowed("\u{0338}")))
    }
}

fn parse_font_style_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let variant = crate::registry::lookup(crate::registry::FONT_STYLES, cmd)
        .expect("font style cmd pre-filtered");

    let content = if let Ok(c) = delimited::<_, _, _, _, winnow::error::ContextError, _, _, _>(
        (space0, '{'),
        parse_row,
        (space0, '}'),
    )
    .parse_next(input)
    {
        c
    } else {
        let remaining_nodes: Vec<MathNode<'s>> =
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
        variant: Cow::Borrowed(variant),
        content: Box::new(content),
    })
}

fn parse_accent_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let mark = crate::registry::lookup(crate::registry::ACCENTS, cmd).expect("accent pre-filtered");
    let content = alt((
        delimited((space0, '{'), parse_row, (space0, '}')),
        preceded(space0, parse_atom),
    ))
    .parse_next(input)?;

    Ok(MathNode::Accent {
        mark: Cow::Borrowed(mark),
        content: Box::new(content),
    })
}

fn parse_sized_delimiter_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let size = crate::registry::lookup(crate::registry::SIZED_DELIMS, cmd)
        .expect("sized delim pre-filtered");
    let delim = parse_fence_delim.parse_next(input)?;
    Ok(MathNode::SizedDelimiter {
        size: Cow::Borrowed(size),
        delim: Cow::Owned(delim),
    })
}

// ---------------------------------------------------------------------------
// Zero-argument + table-driven commands — names live only in `crate::registry`.
// ---------------------------------------------------------------------------

/// Resolve zero-argument table-driven commands. Returns `None` if `cmd` is not in any table.
fn resolve_zero_arg_cmd<'s>(cmd: &'s str) -> Option<MathNode<'s>> {
    use crate::registry::{
        lookup, BLACKBOARD_LETTERS, IDENT_ALIASES, MATH_FUNCTIONS, SPACING_CMDS, VAR_GREEK,
        VAR_LIM_CMDS,
    };

    if MATH_FUNCTIONS.contains(&cmd) {
        return Some(MathNode::Function(Cow::Borrowed(cmd)));
    }
    if let Some(width) = lookup(SPACING_CMDS, cmd) {
        return Some(MathNode::Space(Cow::Borrowed(width)));
    }
    if BLACKBOARD_LETTERS.contains(&cmd) {
        return Some(MathNode::Style {
            variant: Cow::Borrowed("double-struck"),
            content: Box::new(MathNode::Identifier(Cow::Borrowed(cmd))),
        });
    }
    if let Some(ch) = lookup(IDENT_ALIASES, cmd) {
        return Some(MathNode::Identifier(Cow::Borrowed(ch)));
    }
    if let Some(letter) = lookup(VAR_GREEK, cmd) {
        return Some(MathNode::Style {
            variant: Cow::Borrowed("italic"),
            content: Box::new(MathNode::Identifier(Cow::Borrowed(letter))),
        });
    }
    if let Some(arrow) = lookup(VAR_LIM_CMDS, cmd) {
        return Some(MathNode::Scripts {
            base: Box::new(MathNode::Function(Cow::Borrowed("lim"))),
            sub: Some(Box::new(MathNode::Operator(Cow::Borrowed(arrow)))),
            sup: None,
            pre_sub: None,
            pre_sup: None,
            behavior: LimitBehavior::Limits,
        });
    }
    None
}

fn unknown_command_node<'s>(cmd: &str) -> MathNode<'s> {
    use crate::ast::UnknownCommandPolicy;
    match crate::depth::unknown_command_policy() {
        UnknownCommandPolicy::Identifier => MathNode::Identifier(Cow::Owned(format!("\\{cmd}"))),
        UnknownCommandPolicy::Error => {
            MathNode::Error(Cow::Owned(format!("Unknown command \\{cmd}")))
        }
    }
}

pub fn parse_command<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
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

        // Fast path: pure zero-arg macros from static tables.
        if let Some(node) = resolve_zero_arg_cmd(cmd) {
            return Ok(node);
        }

        // Table-driven families — names only in `registry` (no duplicated match lists).
        use crate::registry::{
            lookup, lookup_bool, lookup_stretch, ACCENTS, CANCEL_MODES, EXTENSIBLE_ARROWS,
            FONT_STYLES, FRAC_STYLES, PHANTOM_KINDS, SIZED_DELIMS, STRUCTURAL_CMDS,
        };

        if lookup(FONT_STYLES, cmd).is_some() {
            return parse_font_style_cmd(cmd, input);
        }
        if lookup_bool(FRAC_STYLES, cmd).is_some() {
            return parse_frac_style_cmd(cmd, input);
        }
        if lookup(EXTENSIBLE_ARROWS, cmd).is_some() {
            return parse_extensible_arrow_cmd(cmd, input);
        }
        if lookup_stretch(cmd).is_some() {
            return parse_stretch_modifier_cmd(cmd, input);
        }
        if lookup(ACCENTS, cmd).is_some() {
            return parse_accent_cmd(cmd, input);
        }
        if lookup(CANCEL_MODES, cmd).is_some() {
            return parse_cancel_cmd(cmd, input);
        }
        if PHANTOM_KINDS.contains(&cmd) {
            return parse_phantom_cmd(cmd, input);
        }
        if lookup(SIZED_DELIMS, cmd).is_some() {
            return parse_sized_delimiter_cmd(cmd, input);
        }
        if STRUCTURAL_CMDS.contains(&cmd) {
            // Handled by outer parser (`frac` / `sqrt` / `left` / `right`).
            return winnow::combinator::fail.parse_next(input);
        }

        // Irregular multi-arg combinators (not expressible as simple tables).
        match cmd {
            "text" => parse_text_cmd(input),
            "color" | "textcolor" | "colorbox" => parse_color_cmd(cmd, input),
            "boxed" => parse_boxed_cmd(input),
            "overset" | "underset" => parse_over_under_set_cmd(cmd, input),
            "sideset" => parse_sideset_cmd(input),
            "operatorname" | "operatorname*" => parse_operatorname_cmd(cmd, input),
            "not" => parse_not_modifier_cmd(input),
            _ => {
                if let Some(node) = crate::symbols::lookup_symbol(cmd) {
                    Ok(node)
                } else {
                    Ok(unknown_command_node(cmd))
                }
            }
        }
    })
    .parse_next(input)
}
