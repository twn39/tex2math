//! Shared MathML helpers (escaping, operator-name text extraction).

use crate::ast::*;
use std::fmt::Write;

pub(super) struct EscapedXml<'a>(pub(super) &'a str);

impl<'a> std::fmt::Display for EscapedXml<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for c in self.0.chars() {
            match c {
                '&' => f.write_str("&amp;")?,
                '<' => f.write_str("&lt;")?,
                '>' => f.write_str("&gt;")?,
                '\'' => f.write_str("&apos;")?,
                '"' => f.write_str("&quot;")?,
                _ => f.write_char(c)?,
            }
        }
        Ok(())
    }
}

#[inline]
pub(super) fn escape_xml(input: &str) -> EscapedXml<'_> {
    EscapedXml(input)
}

pub(super) fn try_extract_operator_text<'a>(
    node: &'a MathNode<'_>,
) -> Option<std::borrow::Cow<'a, str>> {
    match node {
        MathNode::Identifier(s)
        | MathNode::Number(s)
        | MathNode::Operator(s)
        | MathNode::Function(s) => Some(std::borrow::Cow::Borrowed(s.as_ref())),
        MathNode::Space(s) => Some(std::borrow::Cow::Borrowed(match s.as_ref() {
            "0.1667em" => " ",
            "0.2222em" => " ",
            "0.2778em" => " ",
            "1em" => " ",
            "2em" => " ",
            _ => " ",
        })),
        MathNode::Row(nodes) => {
            let mut text = String::new();
            for n in nodes {
                text.push_str(try_extract_operator_text(n)?.as_ref());
            }
            Some(std::borrow::Cow::Owned(text))
        }
        _ => None,
    }
}
