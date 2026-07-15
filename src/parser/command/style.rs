//! Font, color, accent, cancel, phantom, style-switch command parsers.

use std::borrow::Cow;
use winnow::ascii::multispace0 as space0;
use winnow::combinator::{alt, delimited, preceded, repeat};
use winnow::prelude::*;

use super::super::atoms::take_balanced_braces;
use super::super::{parse_atom, parse_node, parse_row};
use crate::ast::*;

pub(super) fn parse_text_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
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

pub(super) fn parse_color_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode<'s>> {
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

pub(super) fn parse_boxed_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
    Ok(MathNode::Boxed(Box::new(content)))
}

pub(super) fn parse_cancel_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let mode =
        crate::registry::lookup(crate::registry::CANCEL_MODES, cmd).expect("cancel pre-filtered");
    let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
    Ok(MathNode::Cancel {
        mode: Cow::Borrowed(mode),
        content: Box::new(content),
    })
}

pub(super) fn parse_phantom_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode<'s>> {
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

pub(super) fn parse_style_switch_cmd<'s>(
    displaystyle: bool,
    input: &mut &'s str,
) -> ModalResult<MathNode<'s>> {
    let content = if input.trim_start().starts_with('{') {
        delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?
    } else {
        preceded(space0, parse_atom).parse_next(input)?
    };
    Ok(MathNode::StyledMath {
        displaystyle,
        content: Box::new(content),
    })
}

pub(super) fn parse_font_style_cmd<'s>(
    cmd: &str,
    input: &mut &'s str,
) -> ModalResult<MathNode<'s>> {
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

pub(super) fn parse_accent_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode<'s>> {
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
