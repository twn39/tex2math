//! Dimension / spacing / sized-delimiter command parsers.

use std::borrow::Cow;
use winnow::ascii::multispace0 as space0;
use winnow::prelude::*;

use super::super::atoms::take_balanced_braces;
use super::super::parse_fence_delim;
use crate::ast::*;

/// Parse a TeX dimension token: optional braces, e.g. `1em`, `0.5ex`, `5mu`, `-0.1em`.
pub(super) fn parse_dimension<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
    let _ = space0.parse_next(input)?;
    if input.starts_with('{') {
        let (inner, _) = take_balanced_braces(input)?;
        return Ok(inner.trim());
    }
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

pub(super) fn parse_dim_space_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let dim = parse_dimension(input)?;
    let width = normalize_space_dim(cmd, dim);
    Ok(MathNode::Space(Cow::Owned(width)))
}

pub(super) fn parse_sized_delimiter_cmd<'s>(
    cmd: &str,
    input: &mut &'s str,
) -> ModalResult<MathNode<'s>> {
    let size = crate::registry::lookup(crate::registry::SIZED_DELIMS, cmd)
        .expect("sized delim pre-filtered");
    let delim = parse_fence_delim.parse_next(input)?;
    Ok(MathNode::SizedDelimiter {
        size: Cow::Borrowed(size),
        delim: Cow::Owned(delim),
    })
}
