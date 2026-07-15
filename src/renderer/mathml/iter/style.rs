//! Style / accent / color / phantom / stretch / operatorname emission.

use super::super::basic::{escape_xml, try_extract_operator_text};
use super::super::{MathMLRenderer, RenderCtx};
use super::Frame;
use crate::ast::*;
use std::fmt;

impl MathMLRenderer {
    pub(super) fn expand_style<'n, 's>(
        &self,
        node: &'n MathNode<'s>,
        ctx: &mut RenderCtx<'_>,
        stack: &mut Vec<Frame<'n, 's>>,
    ) -> fmt::Result {
        match node {
            MathNode::Style { variant, content } => {
                if variant.as_ref() == "vphantom" {
                    stack.push(Frame::Lit("</mphantom></mpadded>"));
                    stack.push(Frame::Node(content));
                    stack.push(Frame::Lit("<mpadded width=\"0px\"><mphantom>"));
                } else if variant.as_ref() == "hphantom" {
                    stack.push(Frame::Lit("</mphantom></mpadded>"));
                    stack.push(Frame::Node(content));
                    stack.push(Frame::Lit(
                        "<mpadded height=\"0px\" depth=\"0px\"><mphantom>",
                    ));
                } else {
                    stack.push(Frame::Lit("</mstyle>"));
                    stack.push(Frame::Node(content));
                    stack.push(Frame::Owned(format!(
                        "<mstyle mathvariant=\"{}\">",
                        escape_xml(variant.as_ref())
                    )));
                }
            }
            MathNode::Accent { mark, content } => {
                stack.push(Frame::Owned(format!(
                    "<mo>{}</mo></mover>",
                    escape_xml(mark.as_ref())
                )));
                stack.push(Frame::Node(content));
                if ctx.options.emit_intent {
                    stack.push(Frame::Lit("<mover accent=\"true\" intent=\"accent\">"));
                } else {
                    stack.push(Frame::Lit("<mover accent=\"true\">"));
                }
            }
            MathNode::Color { color, content } => {
                stack.push(Frame::Lit("</mstyle>"));
                stack.push(Frame::Node(content));
                stack.push(Frame::Owned(format!(
                    "<mstyle mathcolor=\"{}\">",
                    escape_xml(color.as_ref())
                )));
            }
            MathNode::ColorBox { bg_color, content } => {
                stack.push(Frame::Lit("</mstyle>"));
                stack.push(Frame::Node(content));
                stack.push(Frame::Owned(format!(
                    "<mstyle mathbackground=\"{}\">",
                    escape_xml(bg_color.as_ref())
                )));
            }
            MathNode::StyledMath {
                displaystyle,
                content,
            } => {
                stack.push(Frame::Lit("</mstyle>"));
                stack.push(Frame::Node(content));
                let ds = if *displaystyle { "true" } else { "false" };
                stack.push(Frame::Owned(format!("<mstyle displaystyle=\"{}\">", ds)));
            }
            MathNode::Phantom { kind, content } => match kind {
                PhantomKind::Invisible => {
                    stack.push(Frame::Lit("</mphantom>"));
                    stack.push(Frame::Node(content));
                    stack.push(Frame::Lit("<mphantom>"));
                }
                PhantomKind::Vertical => {
                    stack.push(Frame::Lit("</mphantom></mpadded>"));
                    stack.push(Frame::Node(content));
                    stack.push(Frame::Lit("<mpadded width=\"0px\"><mphantom>"));
                }
                PhantomKind::Horizontal => {
                    stack.push(Frame::Lit("</mphantom></mpadded>"));
                    stack.push(Frame::Node(content));
                    stack.push(Frame::Lit(
                        "<mpadded height=\"0px\" depth=\"0px\"><mphantom>",
                    ));
                }
            },
            MathNode::StretchOp {
                op,
                is_over,
                content,
            } => {
                let stretchy = format!("<mo stretchy=\"true\">{}</mo>", escape_xml(op.as_ref()));
                if *is_over {
                    stack.push(Frame::Lit("</mover>"));
                    stack.push(Frame::Owned(stretchy));
                    stack.push(Frame::Node(content));
                    stack.push(Frame::Lit("<mover>"));
                } else {
                    stack.push(Frame::Lit("</munder>"));
                    stack.push(Frame::Owned(stretchy));
                    stack.push(Frame::Node(content));
                    stack.push(Frame::Lit("<munder>"));
                }
            }
            MathNode::OperatorName(content) => {
                if let Some(text) = try_extract_operator_text(content) {
                    write!(
                        ctx.out,
                        "<mi mathvariant=\"normal\">{}</mi>",
                        escape_xml(text.as_ref())
                    )?;
                } else {
                    stack.push(Frame::Lit("</mstyle></mrow>"));
                    stack.push(Frame::Node(content));
                    stack.push(Frame::Lit("<mrow><mstyle mathvariant=\"normal\">"));
                }
            }
            _ => unreachable!("expand_style called with non-style node"),
        }
        Ok(())
    }
}
