//! Leaf token emission (numbers, identifiers, operators, text, space, errors).

use super::super::basic::{escape_xml, write_escaped_xml};
use super::super::{MathMLRenderer, RenderCtx};
use crate::ast::*;
use std::fmt;

impl MathMLRenderer {
    pub(super) fn expand_token(&self, node: &MathNode<'_>, ctx: &mut RenderCtx<'_>) -> fmt::Result {
        match node {
            MathNode::Number(n) => {
                if ctx.options.emit_intent {
                    ctx.out.push_str("<mn intent=\"number\">")?;
                } else {
                    ctx.out.push_str("<mn>")?;
                }
                write_escaped_xml(n.as_ref(), ctx.out)?;
                ctx.out.push_str("</mn>")?;
            }
            MathNode::Identifier(i) => {
                if ctx.options.emit_intent {
                    ctx.out.push_str("<mi intent=\"identifier\">")?;
                } else {
                    ctx.out.push_str("<mi>")?;
                }
                write_escaped_xml(i.as_ref(), ctx.out)?;
                ctx.out.push_str("</mi>")?;
            }
            MathNode::Operator(o) => {
                let is_arrow = [
                    "\u{2190}", "\u{2192}", "\u{2194}", "\u{21D2}", "\u{21D0}", "\u{21D4}",
                    "\u{21A6}", "\u{21A9}", "\u{21AA}", "\u{219E}", "\u{21A0}",
                ]
                .contains(&o.as_ref());
                let tag = match (is_arrow, ctx.options.emit_intent) {
                    (true, true) => "<mo stretchy=\"true\" intent=\"operator\">",
                    (true, false) => "<mo stretchy=\"true\">",
                    (false, true) => "<mo intent=\"operator\">",
                    (false, false) => "<mo>",
                };
                ctx.out.push_str(tag)?;
                write_escaped_xml(o.as_ref(), ctx.out)?;
                ctx.out.push_str("</mo>")?;
            }
            MathNode::Text(t) => {
                if ctx.options.emit_intent {
                    ctx.out.push_str("<mtext intent=\"text\">")?;
                } else {
                    ctx.out.push_str("<mtext>")?;
                }
                write_escaped_xml(t.as_ref(), ctx.out)?;
                ctx.out.push_str("</mtext>")?;
            }
            MathNode::Space(width) => {
                ctx.out.push_str("<mspace width=\"")?;
                write_escaped_xml(width.as_ref(), ctx.out)?;
                if ctx.options.emit_intent {
                    ctx.out.push_str("\" intent=\"space\"/>")?;
                } else {
                    ctx.out.push_str("\"/>")?;
                }
            }
            MathNode::Function(f) => {
                let func_text = match f.as_ref() {
                    "injlim" => "inj lim",
                    "projlim" => "proj lim",
                    _ => f.as_ref(),
                };
                if ctx.options.emit_intent {
                    ctx.out
                        .push_str("<mi mathvariant=\"normal\" intent=\"function\">")?;
                } else {
                    ctx.out.push_str("<mi mathvariant=\"normal\">")?;
                }
                write_escaped_xml(func_text, ctx.out)?;
                ctx.out.push_str("</mi>")?;
            }
            MathNode::Error(err_msg) => {
                if ctx.options.emit_intent {
                    ctx.out.push_str(
                        "<merror intent=\"error\"><mtext mathcolor=\"red\">Syntax Error: ",
                    )?;
                } else {
                    ctx.out
                        .push_str("<merror><mtext mathcolor=\"red\">Syntax Error: ")?;
                }
                write_escaped_xml(err_msg.as_ref(), ctx.out)?;
                ctx.out.push_str("</mtext></merror>")?;
            }
            MathNode::SizedDelimiter { size, delim } => {
                let esc_size = escape_xml(size.as_ref());
                let intent = if ctx.options.emit_intent {
                    " intent=\"sized-delimiter\""
                } else {
                    ""
                };
                write!(
                    ctx.out,
                    "<mo minsize=\"{}\" maxsize=\"{}\"{intent}>",
                    esc_size, esc_size,
                )?;
                write_escaped_xml(delim.as_ref(), ctx.out)?;
                ctx.out.push_str("</mo>")?;
            }
            MathNode::Middle(delim) => {
                if ctx.options.emit_intent {
                    ctx.out
                        .push_str("<mo stretchy=\"true\" intent=\"middle\">")?;
                } else {
                    ctx.out.push_str("<mo stretchy=\"true\">")?;
                }
                write_escaped_xml(delim.as_ref(), ctx.out)?;
                ctx.out.push_str("</mo>")?;
            }
            // Should have been folded away; keep a harmless fallback.
            MathNode::ChooseMarker => {
                ctx.out.push_str("<mo>/</mo>")?;
            }
            _ => unreachable!("expand_token called with non-token node"),
        }
        Ok(())
    }
}
