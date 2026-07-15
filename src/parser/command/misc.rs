//! Mod, math-class, substack, middle, tag command parsers.

use std::borrow::Cow;
use winnow::ascii::multispace0 as space0;
use winnow::combinator::delimited;
use winnow::prelude::*;

use super::super::atoms::take_balanced_braces;
use super::super::{parse_fence_delim, parse_row};
use crate::ast::*;

pub(super) fn parse_mod_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode<'s>> {
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

pub(super) fn parse_math_class_cmd<'s>(
    cmd: &str,
    input: &mut &'s str,
) -> ModalResult<MathNode<'s>> {
    let class = crate::registry::math_class_of(cmd).expect("math class pre-filtered");
    let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
    Ok(MathNode::MathClass {
        class,
        content: Box::new(content),
    })
}

pub(super) fn parse_substack_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let (inner, _closed) = take_balanced_braces(input)?;
    let mut cursor = inner;
    let mut rows: Vec<(Vec<MathNode<'s>>, Option<Cow<'s, str>>)> = Vec::new();
    loop {
        let _ = space0.parse_next(&mut cursor)?;
        if cursor.is_empty() {
            break;
        }
        let mark = cursor.len();
        if let Ok(cells) = super::super::environment::parse_cells_in_row.parse_next(&mut cursor) {
            let spacing = if let Ok(opt_sp) =
                super::super::environment::parse_newline_opt.parse_next(&mut cursor)
            {
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

pub(super) fn parse_middle_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let delim = parse_fence_delim.parse_next(input)?;
    Ok(MathNode::Middle(Cow::Owned(delim)))
}

/// Equation tags (`\tag{…}`).
///
/// **Product boundary:** tags are inlined as parenthesized content with leading
/// space (not right-aligned equation numbers). Full `align`/`equation` numbering
/// is out of scope for the MathML fragment emitter.
pub(super) fn parse_tag_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
    Ok(MathNode::Row(vec![
        MathNode::Space(Cow::Borrowed("1em")),
        MathNode::Operator(Cow::Borrowed("(")),
        content,
        MathNode::Operator(Cow::Borrowed(")")),
    ]))
}
