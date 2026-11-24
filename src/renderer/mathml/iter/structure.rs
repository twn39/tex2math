//! Structural containers: fraction, root, row, fenced, boxed, cancel.

use super::super::{MathMLRenderer, RenderCtx};
use super::{Frame, FrameStack};
use crate::ast::*;
use std::fmt;

impl MathMLRenderer {
    pub(super) fn expand_structure<'n, 's>(
        &self,
        node: &'n MathNode<'s>,
        ctx: &mut RenderCtx<'_>,
        stack: &mut FrameStack<'n, 's>,
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
            MathNode::Atop(upper, lower) => {
                stack.push(Frame::Lit("</mfrac>"));
                stack.push(Frame::Node(lower));
                stack.push(Frame::Node(upper));
                if ctx.options.emit_intent {
                    stack.push(Frame::Lit("<mfrac linethickness=\"0\" intent=\"atop\">"));
                } else {
                    stack.push(Frame::Lit("<mfrac linethickness=\"0\">"));
                }
            }
            MathNode::Binom(upper, lower) => {
                // ( n ⁄ k ) with zero rule thickness
                stack.push(Frame::Lit("</mrow>"));
                stack.push(Frame::Lit("<mo>)</mo>"));
                stack.push(Frame::Lit("</mfrac>"));
                stack.push(Frame::Node(lower));
                stack.push(Frame::Node(upper));
                if ctx.options.emit_intent {
                    stack.push(Frame::Lit(
                        "<mfrac linethickness=\"0\" intent=\"binomial\">",
                    ));
                } else {
                    stack.push(Frame::Lit("<mfrac linethickness=\"0\">"));
                }
                stack.push(Frame::Lit("<mo>(</mo>"));
                if ctx.options.emit_intent {
                    stack.push(Frame::Lit("<mrow intent=\"binomial\">"));
                } else {
                    stack.push(Frame::Lit("<mrow>"));
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
                        match mode.as_ref() {
                            "updiagonalstrike" => {
                                stack.push(Frame::Lit("<mrow intent=\"cancel:updiagonalstrike\">"))
                            }
                            "downdiagonalstrike" => stack
                                .push(Frame::Lit("<mrow intent=\"cancel:downdiagonalstrike\">")),
                            "updiagonalstrike downdiagonalstrike" => stack.push(Frame::Lit(
                                "<mrow intent=\"cancel:updiagonalstrike downdiagonalstrike\">",
                            )),
                            _ => stack.push(Frame::OpenTagWithAttr {
                                tag: "mrow",
                                attr: "intent",
                                val: std::borrow::Cow::Owned(format!("cancel:{}", mode)),
                                intent: None,
                            }),
                        }
                    } else {
                        stack.push(Frame::Lit("<mrow>"));
                    }
                } else {
                    stack.push(Frame::Lit("</menclose>"));
                    stack.push(Frame::Node(content));
                    stack.push(Frame::OpenTagWithAttr {
                        tag: "menclose",
                        attr: "notation",
                        val: mode.clone(),
                        intent: None,
                    });
                }
            }
            MathNode::Fenced {
                open,
                content,
                close,
            } => {
                stack.push(Frame::Lit("</mrow>"));
                if close.as_ref() != "." {
                    stack.push(Frame::StretchyMo(close.clone()));
                }
                stack.push(Frame::Lit("</mrow>"));
                stack.push(Frame::Node(content));
                stack.push(Frame::Lit("<mrow>"));
                if open.as_ref() != "." {
                    stack.push(Frame::StretchyMo(open.clone()));
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
