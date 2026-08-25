//! Shared MathML helpers (escaping, operator-name text extraction).

use crate::ast::*;

use crate::renderer::sink::MathSink;

pub(super) struct EscapedXml<'a>(pub(super) &'a str);

impl<'a> std::fmt::Display for EscapedXml<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = self.0.as_bytes();
        let mut last_match = 0;
        let mut idx = 0;

        while idx < bytes.len() {
            let b = bytes[idx];
            let replacement = match b {
                b'&' => "&amp;",
                b'<' => "&lt;",
                b'>' => "&gt;",
                b'\'' => "&apos;",
                b'"' => "&quot;",
                _ => {
                    idx += 1;
                    continue;
                }
            };

            if idx > last_match {
                f.write_str(&self.0[last_match..idx])?;
            }
            f.write_str(replacement)?;
            idx += 1;
            last_match = idx;
        }

        if last_match < self.0.len() {
            f.write_str(&self.0[last_match..])?;
        }
        Ok(())
    }
}

#[inline]
pub(super) fn escape_xml(input: &str) -> EscapedXml<'_> {
    EscapedXml(input)
}

#[inline]
pub(super) fn write_escaped_xml(input: &str, out: &mut dyn MathSink) -> std::fmt::Result {
    let bytes = input.as_bytes();
    let mut last_match = 0;
    let mut idx = 0;

    while idx < bytes.len() {
        let b = bytes[idx];
        let replacement = match b {
            b'&' => "&amp;",
            b'<' => "&lt;",
            b'>' => "&gt;",
            b'\'' => "&apos;",
            b'"' => "&quot;",
            _ => {
                idx += 1;
                continue;
            }
        };

        if idx > last_match {
            out.push_str(&input[last_match..idx])?;
        }
        out.push_str(replacement)?;
        idx += 1;
        last_match = idx;
    }

    if last_match < input.len() {
        out.push_str(&input[last_match..])?;
    }
    Ok(())
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
