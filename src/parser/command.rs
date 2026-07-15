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

fn parse_binom_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let style = crate::registry::BINOM_STYLES
        .iter()
        .find(|&&(k, _)| k == cmd)
        .map(|&(_, s)| s)
        .expect("binom pre-filtered");
    let upper = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
    let lower = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
    let binom = MathNode::Binom(Box::new(upper), Box::new(lower));
    Ok(match style {
        None => binom,
        Some(ds) => MathNode::StyledMath {
            displaystyle: ds,
            content: Box::new(binom),
        },
    })
}

fn parse_mod_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    // Spacing and shape follow common TeX/KaTeX conventions (simplified MathML).
    // `\pmod` / `\pod` take a braced argument; `\bmod` / `\mod` are infix operators
    // (optional braces for a trailing operand are accepted but uncommon).
    Ok(match cmd {
        "pmod" => {
            let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
            MathNode::Row(vec![
                MathNode::Space(Cow::Borrowed("0.35em")),
                MathNode::Operator(Cow::Borrowed("(")),
                MathNode::Function(Cow::Borrowed("mod")),
                MathNode::Space(Cow::Borrowed("0.1667em")),
                content,
                MathNode::Operator(Cow::Borrowed(")")),
            ])
        }
        "pod" => {
            let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
            MathNode::Row(vec![
                MathNode::Space(Cow::Borrowed("0.35em")),
                MathNode::Operator(Cow::Borrowed("(")),
                content,
                MathNode::Operator(Cow::Borrowed(")")),
            ])
        }
        "bmod" => {
            if input.trim_start().starts_with('{') {
                let content =
                    delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
                MathNode::Row(vec![
                    MathNode::Space(Cow::Borrowed("0.2222em")),
                    MathNode::Function(Cow::Borrowed("mod")),
                    MathNode::Space(Cow::Borrowed("0.2222em")),
                    content,
                ])
            } else {
                MathNode::Row(vec![
                    MathNode::Space(Cow::Borrowed("0.2222em")),
                    MathNode::Function(Cow::Borrowed("mod")),
                    MathNode::Space(Cow::Borrowed("0.2222em")),
                ])
            }
        }
        // `\mod` — relational spacing before "mod", optional braced operand
        _ => {
            if input.trim_start().starts_with('{') {
                let content =
                    delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
                MathNode::Row(vec![
                    MathNode::Space(Cow::Borrowed("0.35em")),
                    MathNode::Function(Cow::Borrowed("mod")),
                    MathNode::Space(Cow::Borrowed("0.1667em")),
                    content,
                ])
            } else {
                MathNode::Row(vec![
                    MathNode::Space(Cow::Borrowed("0.35em")),
                    MathNode::Function(Cow::Borrowed("mod")),
                    MathNode::Space(Cow::Borrowed("0.1667em")),
                ])
            }
        }
    })
}

fn parse_math_class_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    // Class is a spacing hint in TeX; we accept the argument so it is not treated as unknown.
    delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)
}

fn parse_style_switch_cmd<'s>(
    displaystyle: bool,
    input: &mut &'s str,
) -> ModalResult<MathNode<'s>> {
    let content = if input.trim_start().starts_with('{') {
        delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?
    } else {
        // Apply to the next atom (TeX switch approximation inside a row).
        preceded(space0, parse_atom).parse_next(input)?
    };
    Ok(MathNode::StyledMath {
        displaystyle,
        content: Box::new(content),
    })
}

/// Parse a TeX dimension token: optional braces, e.g. `1em`, `0.5ex`, `5mu`, `-0.1em`.
fn parse_dimension<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
    let _ = space0.parse_next(input)?;
    if input.starts_with('{') {
        let (inner, _) = take_balanced_braces(input)?;
        return Ok(inner.trim());
    }
    // bare: optional sign, digits, optional fraction, unit
    let start = *input;
    let mut i = 0;
    let bytes = start.as_bytes();
    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
        i += 1;
    }
    let dig_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
    }
    if i == dig_start {
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }
    // unit: em, ex, mu, pt, pc, in, cm, mm, bp, dd, cc, sp, or px
    let unit_start = i;
    while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
        i += 1;
    }
    if i == unit_start {
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }
    let dim = &start[..i];
    *input = &start[i..];
    Ok(dim)
}

fn normalize_space_dim(cmd: &str, dim: &str) -> String {
    // `\mkern` / `\mskip` use math units (mu); approximate 1mu ≈ 1/18 em.
    if matches!(cmd, "mkern" | "mskip") {
        let trimmed = dim.trim();
        if let Some(num) = trimmed.strip_suffix("mu") {
            if let Ok(v) = num.trim().parse::<f64>() {
                return format!("{}em", v / 18.0);
            }
        }
    }
    dim.trim().to_string()
}

fn parse_dim_space_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let dim = parse_dimension(input)?;
    let width = normalize_space_dim(cmd, dim);
    Ok(MathNode::Space(Cow::Owned(width)))
}

fn parse_substack_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let (inner, _closed) = take_balanced_braces(input)?;
    let mut cursor = inner;
    let mut rows: Vec<(Vec<MathNode<'s>>, Option<Cow<'s, str>>)> = Vec::new();
    loop {
        let _ = space0.parse_next(&mut cursor)?;
        if cursor.is_empty() {
            break;
        }
        let mark = cursor.len();
        if let Ok(cells) = super::environment::parse_cells_in_row.parse_next(&mut cursor) {
            let spacing =
                if let Ok(opt_sp) = super::environment::parse_newline_opt.parse_next(&mut cursor) {
                    opt_sp.map(Cow::Borrowed)
                } else {
                    None
                };
            rows.push((cells, spacing));
        } else {
            break;
        }
        if cursor.len() == mark {
            break;
        }
    }
    if rows.is_empty() {
        rows.push((vec![MathNode::Row(vec![])], None));
    }
    Ok(MathNode::Environment {
        name: Cow::Borrowed("substack"),
        format: None,
        rows,
    })
}

fn parse_middle_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let delim = parse_fence_delim.parse_next(input)?;
    Ok(MathNode::Middle(Cow::Owned(delim)))
}

fn thickness_is_zero(thick: &str) -> bool {
    let t = thick.trim();
    if t.is_empty() || t == "0" {
        return true;
    }
    // 0pt, 0em, 0.0mu, …
    let num: String = t
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '+' || *c == '-')
        .collect();
    num.parse::<f64>().map(|v| v == 0.0).unwrap_or(false)
}

fn parse_genfrac_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    // \genfrac{left}{right}{thickness}{style}{num}{den}
    let (left, _) = take_balanced_braces(input)?;
    let (right, _) = take_balanced_braces(input)?;
    let (thick, _) = take_balanced_braces(input)?;
    let (style, _) = take_balanced_braces(input)?;
    let num = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
    let den = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;

    let left = left.trim();
    let right = right.trim();
    let paren_delims = left == "(" && right == ")";
    let no_delims = left.is_empty() && right.is_empty();

    // Zero rule + () → Binom; zero rule without delims still uses Binom (adds ());
    // non-zero rule → Fraction, optionally fenced.
    let body = if thickness_is_zero(thick) && (paren_delims || no_delims) {
        MathNode::Binom(Box::new(num), Box::new(den))
    } else {
        let frac = MathNode::Fraction(Box::new(num), Box::new(den));
        if no_delims {
            frac
        } else {
            MathNode::Fenced {
                open: Cow::Owned(if left.is_empty() {
                    ".".into()
                } else {
                    left.to_string()
                }),
                content: Box::new(frac),
                close: Cow::Owned(if right.is_empty() {
                    ".".into()
                } else {
                    right.to_string()
                }),
            }
        }
    };

    Ok(match style.trim() {
        "0" => MathNode::StyledMath {
            displaystyle: true,
            content: Box::new(body),
        },
        "1" | "2" | "3" => MathNode::StyledMath {
            displaystyle: false,
            content: Box::new(body),
        },
        _ => body,
    })
}

fn parse_tag_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
    Ok(MathNode::Row(vec![
        MathNode::Space(Cow::Borrowed("1em")),
        MathNode::Operator(Cow::Borrowed("(")),
        content,
        MathNode::Operator(Cow::Borrowed(")")),
    ]))
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
            lookup, lookup_bool, lookup_stretch, ACCENTS, BINOM_STYLES, CANCEL_MODES,
            DIM_SPACE_CMDS, EXTENSIBLE_ARROWS, FONT_STYLES, FRAC_STYLES, MATH_CLASS_CMDS, MOD_CMDS,
            PHANTOM_KINDS, SIZED_DELIMS, STRUCTURAL_CMDS, STYLE_SWITCH_CMDS,
        };

        if lookup(FONT_STYLES, cmd).is_some() {
            return parse_font_style_cmd(cmd, input);
        }
        if lookup_bool(FRAC_STYLES, cmd).is_some() {
            return parse_frac_style_cmd(cmd, input);
        }
        if BINOM_STYLES.iter().any(|&(k, _)| k == cmd) {
            return parse_binom_cmd(cmd, input);
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
        if MOD_CMDS.contains(&cmd) {
            return parse_mod_cmd(cmd, input);
        }
        if MATH_CLASS_CMDS.contains(&cmd) {
            return parse_math_class_cmd(input);
        }
        if let Some(&(_, ds)) = STYLE_SWITCH_CMDS.iter().find(|&&(k, _)| k == cmd) {
            return parse_style_switch_cmd(ds, input);
        }
        if DIM_SPACE_CMDS.contains(&cmd) {
            return parse_dim_space_cmd(cmd, input);
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
            "overset" | "underset" | "stackrel" => {
                parse_over_under_set_cmd(if cmd == "stackrel" { "overset" } else { cmd }, input)
            }
            "sideset" => parse_sideset_cmd(input),
            "operatorname" | "operatorname*" => parse_operatorname_cmd(cmd, input),
            "not" => parse_not_modifier_cmd(input),
            "choose" => Ok(MathNode::ChooseMarker),
            "genfrac" => parse_genfrac_cmd(input),
            "substack" => parse_substack_cmd(input),
            "middle" => parse_middle_cmd(input),
            "tag" => parse_tag_cmd(input),
            "notag" => Ok(MathNode::Row(vec![])),
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
