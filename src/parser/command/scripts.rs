//! Script / stretch / arrow / not / operatorname command parsers.

use std::borrow::Cow;
use winnow::ascii::{alpha1, multispace0 as space0};
use winnow::combinator::{delimited, opt, preceded};
use winnow::prelude::*;
use winnow::token::one_of;

use super::super::atoms::take_balanced_brackets;
use super::super::{parse_atom, parse_node, parse_row};
use crate::ast::*;

fn extract_scripts<'s>(
    node: &MathNode<'s>,
) -> (Option<Box<MathNode<'s>>>, Option<Box<MathNode<'s>>>) {
    match node {
        MathNode::Scripts { sub, sup, .. } => (sub.clone(), sup.clone()),
        MathNode::Row(nodes) if nodes.len() == 1 => extract_scripts(&nodes[0]),
        _ => (None, None),
    }
}

pub(super) fn parse_sideset_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
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

pub(super) fn parse_over_under_set_cmd<'s>(
    cmd: &str,
    input: &mut &'s str,
) -> ModalResult<MathNode<'s>> {
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

pub(super) fn parse_extensible_arrow_cmd<'s>(
    cmd: &str,
    input: &mut &'s str,
) -> ModalResult<MathNode<'s>> {
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

pub(super) fn parse_stretch_modifier_cmd<'s>(
    cmd: &str,
    input: &mut &'s str,
) -> ModalResult<MathNode<'s>> {
    let (op_str, is_over) = crate::registry::lookup_stretch(cmd).expect("stretch op pre-filtered");
    let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
    Ok(MathNode::StretchOp {
        op: Cow::Borrowed(op_str),
        is_over,
        content: Box::new(content),
    })
}

pub(super) fn parse_operatorname_cmd<'s>(
    cmd: &str,
    input: &mut &'s str,
) -> ModalResult<MathNode<'s>> {
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

pub(super) fn parse_not_modifier_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
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

        *input = checkpoint;
    } else if opt(one_of::<_, _, winnow::error::ContextError>('='))
        .parse_next(input)?
        .is_some()
    {
        return Ok(MathNode::Operator(Cow::Borrowed("\u{2260}")));
    }

    if let Ok(next_node) = parse_atom.parse_next(input) {
        Ok(MathNode::Row(vec![
            next_node,
            MathNode::Operator(Cow::Borrowed("\u{0338}")),
        ]))
    } else {
        Ok(MathNode::Operator(Cow::Borrowed("\u{0338}")))
    }
}
