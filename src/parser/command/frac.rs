//! Fraction / binomial / genfrac command parsers.

use std::borrow::Cow;
use winnow::ascii::multispace0 as space0;
use winnow::combinator::delimited;
use winnow::prelude::*;

use super::super::atoms::take_balanced_braces;
use super::super::parse_row;
use crate::ast::*;

pub(super) fn parse_frac_style_cmd<'s>(
    cmd: &str,
    input: &mut &'s str,
) -> ModalResult<MathNode<'s>> {
    let ds = crate::registry::lookup_bool(crate::registry::FRAC_STYLES, cmd)
        .expect("frac style pre-filtered");
    let num = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
    let den = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
    Ok(MathNode::StyledMath {
        displaystyle: ds,
        content: Box::new(MathNode::Fraction(Box::new(num), Box::new(den))),
    })
}

pub(super) fn parse_binom_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let style = crate::registry::lookup_opt_bool(crate::registry::BINOM_STYLES, cmd)
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

fn thickness_is_zero(thick: &str) -> bool {
    let t = thick.trim();
    if t.is_empty() || t == "0" {
        return true;
    }
    let num: String = t
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '+' || *c == '-')
        .collect();
    num.parse::<f64>().map(|v| v == 0.0).unwrap_or(false)
}

pub(super) fn parse_genfrac_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
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
