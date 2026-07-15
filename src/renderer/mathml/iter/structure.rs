//! Structural containers: fraction, root, row, fenced, boxed, cancel.

use super::super::basic::escape_xml;
use super::super::{MathMLRenderer, RenderCtx};
use super::Frame;
use crate::ast::*;
use std::fmt;

impl MathMLRenderer {
    pub(super) fn expand_structure<'n, 's>(
        &self,
        node: &'n MathNode<'s>,
        ctx: &mut RenderCtx<'_>,
        stack: &mut Vec<Frame<'n, 's>>,
    ) -> fmt::Result {
        match node {
            MathNode::Fraction(num, den) => {
                stack.push(Frame::Lit("</mfrac>"));
                stack.push(Frame::Node(den));
                stack.push(Frame::Node(num));
                if ctx.options.emit_intent {
                    stack.push(Frame::Lit("<mfrac intent=\"fraction\">"));
                } else {
                    stack.push(Frame::Lit("<mfrac>"));
                }
            }
            MathNode::Sqrt(content) => {
                stack.push(Frame::Lit("</msqrt>"));
                stack.push(Frame::Node(content));
                if ctx.options.emit_intent {
                    stack.push(Frame::Lit("<msqrt intent=\"square-root\">"));
                } else {
                    stack.push(Frame::Lit("<msqrt>"));
                }
            }
            MathNode::Root { index, content } => {
                stack.push(Frame::Lit("</mroot>"));
                stack.push(Frame::Node(index));
                stack.push(Frame::Node(content));
                if ctx.options.emit_intent {
                    stack.push(Frame::Lit("<mroot intent=\"root\">"));
                } else {
                    stack.push(Frame::Lit("<mroot>"));
                }
            }
            MathNode::Row(nodes) => {
                stack.push(Frame::Lit("</mrow>"));
                for n in nodes.iter().rev() {
                    stack.push(Frame::Node(n));
                }
                if ctx.options.emit_intent {
                    stack.push(Frame::Lit("<mrow intent=\"group\">"));
                } else {
                    stack.push(Frame::Lit("<mrow>"));
                }
            }
            MathNode::Boxed(content) => {
                if ctx.options.mathml_core {
                    stack.push(Frame::Lit("</mrow>"));
                    stack.push(Frame::Node(content));
                    if ctx.options.emit_intent {
                        stack.push(Frame::Lit("<mrow intent=\"boxed\">"));
                    } else {
                        stack.push(Frame::Lit("<mrow>"));
                    }
                } else {
                    stack.push(Frame::Lit("</menclose>"));
                    stack.push(Frame::Node(content));
                    stack.push(Frame::Lit("<menclose notation=\"box\">"));
                }
            }
            MathNode::Cancel { mode, content } => {
                if ctx.options.mathml_core {
                    stack.push(Frame::Lit("</mrow>"));
                    stack.push(Frame::Node(content));
                    if ctx.options.emit_intent {
                        stack.push(Frame::Owned(format!(
                            "<mrow intent=\"cancel:{}\">",
                            escape_xml(mode.as_ref())
                        )));
                    } else {
                        stack.push(Frame::Lit("<mrow>"));
                    }
                } else {
                    stack.push(Frame::Lit("</menclose>"));
                    stack.push(Frame::Node(content));
                    stack.push(Frame::Owned(format!(
                        "<menclose notation=\"{}\">",
                        escape_xml(mode.as_ref())
                    )));
                }
            }
            MathNode::Fenced {
                open,
                content,
                close,
            } => {
                stack.push(Frame::Lit("</mrow>"));
                if close.as_ref() != "." {
                    stack.push(Frame::Owned(format!(
                        "<mo stretchy=\"true\">{}</mo>",
                        escape_xml(close.as_ref())
                    )));
                }
                stack.push(Frame::Lit("</mrow>"));
                stack.push(Frame::Node(content));
                stack.push(Frame::Lit("<mrow>"));
                if open.as_ref() != "." {
                    stack.push(Frame::Owned(format!(
                        "<mo stretchy=\"true\">{}</mo>",
                        escape_xml(open.as_ref())
                    )));
                }
                if ctx.options.emit_intent {
                    stack.push(Frame::Lit("<mrow intent=\"fenced\">"));
                } else {
                    stack.push(Frame::Lit("<mrow>"));
                }
            }
            _ => unreachable!("expand_structure called with non-structure node"),
        }
        Ok(())
    }
}
