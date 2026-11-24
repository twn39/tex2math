use std::borrow::Cow;
use winnow::ascii::{digit1, multispace0 as space0};
use winnow::combinator::{alt, opt, preceded, trace};
use winnow::prelude::*;
use winnow::token::one_of;

use super::parse_row;
use crate::ast::*;

/// Parses a numeric literal (e.g., `123`, `3.14`, `.14`, `10.`) into a `MathNode::Number`.
pub fn parse_number<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    trace(
        "parse_number",
        alt((
            // Format 1: Normal decimal or integer (e.g., "123", "3.14", "10.")
            (digit1, opt(('.', opt(digit1)))).take(),
            // Format 2: Leading decimal (e.g., ".14")
            ('.', digit1).take(),
        ))
        .map(|s: &str| MathNode::Number(Cow::Borrowed(s))),
    )
    .parse_next(input)
}

/// Parses a single alphabetic character as an identifier into a `MathNode::Identifier`.
pub fn parse_ident<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    trace(
        "parse_ident",
        winnow::token::one_of(|c: char| c.is_ascii_alphabetic())
            .take()
            .map(|s: &'s str| MathNode::Identifier(Cow::Borrowed(s))),
    )
    .parse_next(input)
}

/// Parses common mathematical operators (e.g., `+`, `-`, `=`, `!`) into a `MathNode::Operator`.
pub fn parse_operator<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    trace(
        "parse_operator",
        one_of([
            '+', '-', '=', '<', '>', '(', ')', '[', ']', '|', ',', '/', '*', '.', ':',
        ])
        .take()
        .map(|s: &'s str| MathNode::Operator(Cow::Borrowed(s))),
    )
    .parse_next(input)
}

/// Parses a grouped expression enclosed in curly braces (`{...}`) into a `MathNode::Row` or a single node.
pub fn parse_group<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    trace("parse_group", |input: &mut &'s str| {
        // 匹配左大括号
        let _ = preceded(space0, winnow::token::literal("{")).parse_next(input)?;

        // 尝试正常解析一行内容
        let content = parse_row.parse_next(input)?;

        // 尝试匹配右大括号
        if opt(preceded(space0, winnow::token::literal("}")))
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
                MathNode::Error(Cow::Owned(format!("Missing '}}', found: '{}'", remaining))),
            ]))
        }
    })
    .parse_next(input)
}

/// Fallback for raw Unicode characters not explicitly matched as operators or identifiers.
pub fn parse_fallback_char<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    trace(
        "parse_fallback_char",
        winnow::token::one_of(|c: char| !c.is_ascii_whitespace() && !"\\{}_^&%$#~".contains(c))
            .with_taken()
            .map(|(c, s): (char, &'s str)| {
                if c.is_alphabetic() {
                    MathNode::Identifier(Cow::Borrowed(s))
                } else if c.is_numeric() {
                    MathNode::Number(Cow::Borrowed(s))
                } else {
                    MathNode::Operator(Cow::Borrowed(s))
                }
            }),
    )
    .parse_next(input)
}

/// Scans a balanced delimiter pair (`open`..`close`, e.g. `{}` / `[]` / `()`),
/// honoring backslash escapes via `memchr3`.
/// Returns `(inner_slice, found_closing_delimiter)`.
#[inline]
pub(crate) fn take_balanced_delims<'s>(
    input: &mut &'s str,
    open: u8,
    close: u8,
) -> ModalResult<(&'s str, bool)> {
    let _ = preceded(space0, winnow::token::one_of(open as char)).parse_next(input)?;

    let full = *input;
    let bytes = full.as_bytes();
    let mut depth = 1usize;
    let mut offset = 0usize;
    let mut len = 0usize;

    while offset < bytes.len() {
        if let Some(pos) = memchr::memchr3(open, close, b'\\', &bytes[offset..]) {
            let found_idx = offset + pos;
            let byte = bytes[found_idx];
            if byte == b'\\' {
                let mut backslashes = 1usize;
                while found_idx + backslashes < bytes.len()
                    && bytes[found_idx + backslashes] == b'\\'
                {
                    backslashes += 1;
                }
                if !backslashes.is_multiple_of(2) {
                    offset = (found_idx + backslashes + 1).min(bytes.len());
                } else {
                    offset = found_idx + backslashes;
                }
            } else if byte == open {
                depth += 1;
                offset = found_idx + 1;
            } else if byte == close {
                depth -= 1;
                if depth == 0 {
                    len = found_idx;
                    break;
                }
                offset = found_idx + 1;
            }
        } else {
            break;
        }
    }

    if depth == 0 {
        let content = &full[..len];
        *input = &full[len + 1..];
        Ok((content, true))
    } else {
        let content = full;
        *input = &full[full.len()..];
        Ok((content, false))
    }
}

/// Parses a string slice enclosed in matching curly braces `{...}`, keeping track of nesting depth and escaping backslashes.
/// Returns the inner content (excluding the outermost braces) and whether the closing brace was found.
#[inline]
pub fn take_balanced_braces<'s>(input: &mut &'s str) -> ModalResult<(&'s str, bool)> {
    take_balanced_delims(input, b'{', b'}')
}

/// Parses a string slice enclosed in matching square brackets `[...]`, keeping track of nesting depth and escaping backslashes.
/// Returns the inner content (excluding the outermost brackets) and whether the closing bracket was found.
#[inline]
pub fn take_balanced_brackets<'s>(input: &mut &'s str) -> ModalResult<(&'s str, bool)> {
    take_balanced_delims(input, b'[', b']')
}
