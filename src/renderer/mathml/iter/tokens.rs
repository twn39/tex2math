//! Leaf token emission (numbers, identifiers, operators, text, space, errors).

use super::super::basic::escape_xml;
use super::super::{MathMLRenderer, RenderCtx};
use crate::ast::*;
use std::fmt;

impl MathMLRenderer {
    pub(super) fn expand_token(&self, node: &MathNode<'_>, ctx: &mut RenderCtx<'_>) -> fmt::Result {
        match node {
            MathNode::Number(n) => {
                write!(ctx.out, "<mn>{}</mn>", escape_xml(n.as_ref()))?;
            }
            MathNode::Identifier(i) => {
                write!(ctx.out, "<mi>{}</mi>", escape_xml(i.as_ref()))?;
            }
            MathNode::Operator(o) => {
                let is_arrow = [
                    "\u{2190}", "\u{2192}", "\u{2194}", "\u{21D2}", "\u{21D0}", "\u{21D4}",
                    "\u{21A6}", "\u{21A9}", "\u{21AA}", "\u{219E}", "\u{21A0}",
                ]
                .contains(&o.as_ref());
                if is_arrow {
                    write!(
                        ctx.out,
                        "<mo stretchy=\"true\">{}</mo>",
                        escape_xml(o.as_ref())
                    )?;
                } else {
                    write!(ctx.out, "<mo>{}</mo>", escape_xml(o.as_ref()))?;
                }
            }
            MathNode::Text(t) => {
                write!(ctx.out, "<mtext>{}</mtext>", escape_xml(t.as_ref()))?;
            }
            MathNode::Space(width) => {
                write!(
                    ctx.out,
                    "<mspace width=\"{}\"/>",
                    escape_xml(width.as_ref())
                )?;
            }
            MathNode::Function(f) => {
                let func_text = match f.as_ref() {
                    "injlim" => "inj lim",
                    "projlim" => "proj lim",
                    _ => f.as_ref(),
                };
                write!(
                    ctx.out,
                    "<mi mathvariant=\"normal\">{}</mi>",
                    escape_xml(func_text)
                )?;
            }
            MathNode::Error(err_msg) => {
                write!(
                    ctx.out,
                    "<merror><mtext mathcolor=\"red\">Syntax Error: {}</mtext></merror>",
                    escape_xml(err_msg.as_ref())
                )?;
            }
            MathNode::SizedDelimiter { size, delim } => {
                let esc_size = escape_xml(size.as_ref());
                write!(
                    ctx.out,
                    "<mo minsize=\"{}\" maxsize=\"{}\">{}</mo>",
                    esc_size,
                    esc_size,
                    escape_xml(delim.as_ref())
                )?;
            }
            MathNode::Middle(delim) => {
                write!(
                    ctx.out,
                    "<mo stretchy=\"true\">{}</mo>",
                    escape_xml(delim.as_ref())
                )?;
            }
            // Should have been folded away; keep a harmless fallback.
            MathNode::ChooseMarker => {
                write!(ctx.out, "<mo>/</mo>")?;
            }
            _ => unreachable!("expand_token called with non-token node"),
        }
        Ok(())
    }
}
