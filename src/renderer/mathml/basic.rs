use super::MathMLRenderer;
use crate::ast::*;
use std::fmt::Write;

pub(super) struct EscapedXml<'a>(&'a str);

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

impl MathMLRenderer {
    pub(super) fn render_basic_node(
        &self,
        node: &MathNode,
        _mode: RenderMode,
        buf: &mut String,
    ) -> std::fmt::Result {
        match node {
            MathNode::Number(n) => write!(buf, "<mn>{}</mn>", escape_xml(n)),
            MathNode::Identifier(i) => write!(buf, "<mi>{}</mi>", escape_xml(i)),
            MathNode::Operator(o) => {
                let is_arrow = [
                    "\u{2190}", "\u{2192}", "\u{2194}", "\u{21D2}", "\u{21D0}", "\u{21D4}",
                    "\u{21A6}", "\u{21A9}", "\u{21AA}", "\u{219E}", "\u{21A0}",
                ]
                .contains(&o.as_str());
                if is_arrow {
                    write!(buf, "<mo stretchy=\"true\">{}</mo>", escape_xml(o))
                } else {
                    write!(buf, "<mo>{}</mo>", escape_xml(o))
                }
            }
            MathNode::Text(t) => write!(buf, "<mtext>{}</mtext>", escape_xml(t)),
            MathNode::Space(width) => write!(buf, "<mspace width=\"{}\"/>", escape_xml(width)),
            MathNode::Function(f) => {
                let func_text = match f.as_str() {
                    "injlim" => "inj lim",
                    "projlim" => "proj lim",
                    _ => f.as_str(),
                };
                write!(
                    buf,
                    "<mi mathvariant=\"normal\">{}</mi>",
                    escape_xml(func_text)
                )
            }
            _ => unreachable!(),
        }
    }
}

pub(super) fn try_extract_operator_text<'a>(
    node: &'a MathNode,
) -> Option<std::borrow::Cow<'a, str>> {
    match node {
        MathNode::Identifier(s)
        | MathNode::Number(s)
        | MathNode::Operator(s)
        | MathNode::Function(s) => Some(std::borrow::Cow::Borrowed(s.as_str())),
        MathNode::Space(s) => Some(std::borrow::Cow::Borrowed(match s.as_str() {
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
