//! Style / accent / color / phantom / stretch / operatorname emission.

use super::super::basic::{escape_xml, try_extract_operator_text};
use super::super::{MathMLRenderer, RenderCtx};
use super::{Frame, FrameStack};
use crate::ast::*;
use std::fmt;

impl MathMLRenderer {
    pub(super) fn expand_style<'n, 's>(
        &self,
        node: &'n MathNode<'s>,
        ctx: &mut RenderCtx<'_>,
        stack: &mut FrameStack<'n, 's>,
    ) -> fmt::Result {
        match node {
            MathNode::Style { variant, content } => {
                stack.push(Frame::Lit("</mstyle>"));
                stack.push(Frame::Node(content));
                stack.push(Frame::OpenTagWithAttr {
                    tag: "mstyle",
                    attr: "mathvariant",
                    val: variant.clone(),
                    intent: if ctx.options.emit_intent {
                        Some("style")
                    } else {
                        None
                    },
                });
            }
            MathNode::Accent { mark, content } => {
                stack.push(Frame::AccentClose(mark.clone()));
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
                stack.push(Frame::OpenTagWithAttr {
                    tag: "mstyle",
                    attr: "mathcolor",
                    val: color.clone(),
                    intent: if ctx.options.emit_intent {
                        Some("color")
                    } else {
                        None
                    },
                });
            }
            MathNode::ColorBox { bg_color, content } => {
                stack.push(Frame::Lit("</mstyle>"));
                stack.push(Frame::Node(content));
                stack.push(Frame::OpenTagWithAttr {
                    tag: "mstyle",
                    attr: "mathbackground",
                    val: bg_color.clone(),
                    intent: if ctx.options.emit_intent {
                        Some("colorbox")
                    } else {
                        None
                    },
                });
            }
            MathNode::MathClass { class, content } => {
                let (lspace, rspace) = class.default_spaces();
                // Prefer a single spaced <mo> when the body is a simple operator/token.
                if let Some(text) = try_extract_operator_text(content) {
                    if ctx.options.emit_intent {
                        write!(
                            ctx.out,
                            "<mo lspace=\"{}\" rspace=\"{}\" intent=\"math-class:{}\">{}</mo>",
                            lspace,
                            rspace,
                            class.as_str(),
                            escape_xml(text.as_ref())
                        )?;
                    } else {
                        write!(
                            ctx.out,
                            "<mo lspace=\"{}\" rspace=\"{}\">{}</mo>",
                            lspace,
                            rspace,
                            escape_xml(text.as_ref())
                        )?;
                    }
                } else {
                    stack.push(Frame::Lit("</mrow>"));
                    stack.push(Frame::Node(content));
                    if ctx.options.emit_intent {
                        stack.push(Frame::OpenTagWithAttr {
                            tag: "mrow",
                            attr: "intent",
                            val: std::borrow::Cow::Owned(format!("math-class:{}", class.as_str())),
                            intent: None,
                        });
                    } else {
                        stack.push(Frame::Lit("<mrow>"));
                    }
                }
            }
            MathNode::StyledMath {
                displaystyle,
                content,
            } => {
                stack.push(Frame::Lit("</mstyle>"));
                stack.push(Frame::Node(content));
                let tag = match (*displaystyle, ctx.options.emit_intent) {
                    (true, true) => "<mstyle displaystyle=\"true\" intent=\"displaystyle\">",
                    (true, false) => "<mstyle displaystyle=\"true\">",
                    (false, true) => "<mstyle displaystyle=\"false\" intent=\"textstyle\">",
                    (false, false) => "<mstyle displaystyle=\"false\">",
                };
                stack.push(Frame::Lit(tag));
            }
            MathNode::Phantom { kind, content } => match (kind, ctx.options.emit_intent) {
                (PhantomKind::Invisible, true) => {
                    stack.push(Frame::Lit("</mphantom>"));
                    stack.push(Frame::Node(content));
                    stack.push(Frame::Lit("<mphantom intent=\"phantom\">"));
                }
                (PhantomKind::Invisible, false) => {
                    stack.push(Frame::Lit("</mphantom>"));
                    stack.push(Frame::Node(content));
                    stack.push(Frame::Lit("<mphantom>"));
                }
                (PhantomKind::Vertical, true) => {
                    stack.push(Frame::Lit("</mphantom></mpadded>"));
                    stack.push(Frame::Node(content));
                    stack.push(Frame::Lit(
                        "<mpadded width=\"0px\" intent=\"vphantom\"><mphantom>",
                    ));
                }
                (PhantomKind::Vertical, false) => {
                    stack.push(Frame::Lit("</mphantom></mpadded>"));
                    stack.push(Frame::Node(content));
                    stack.push(Frame::Lit("<mpadded width=\"0px\"><mphantom>"));
                }
                (PhantomKind::Horizontal, true) => {
                    stack.push(Frame::Lit("</mphantom></mpadded>"));
                    stack.push(Frame::Node(content));
                    stack.push(Frame::Lit(
                        "<mpadded height=\"0px\" depth=\"0px\" intent=\"hphantom\"><mphantom>",
                    ));
                }
                (PhantomKind::Horizontal, false) => {
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
                if *is_over {
                    stack.push(Frame::Lit("</mover>"));
                    stack.push(Frame::StretchyMo(op.clone()));
                    stack.push(Frame::Node(content));
                    if ctx.options.emit_intent {
                        stack.push(Frame::Lit("<mover intent=\"stretch-over\">"));
                    } else {
                        stack.push(Frame::Lit("<mover>"));
                    }
                } else {
                    stack.push(Frame::Lit("</munder>"));
                    stack.push(Frame::StretchyMo(op.clone()));
                    stack.push(Frame::Node(content));
                    if ctx.options.emit_intent {
                        stack.push(Frame::Lit("<munder intent=\"stretch-under\">"));
                    } else {
                        stack.push(Frame::Lit("<munder>"));
                    }
                }
            }
            MathNode::OperatorName(content) => {
                if let Some(text) = try_extract_operator_text(content) {
                    if ctx.options.emit_intent {
                        write!(
                            ctx.out,
                            "<mi mathvariant=\"normal\" intent=\"operator-name\">{}</mi>",
                            escape_xml(text.as_ref())
                        )?;
                    } else {
                        write!(
                            ctx.out,
                            "<mi mathvariant=\"normal\">{}</mi>",
                            escape_xml(text.as_ref())
                        )?;
                    }
                } else {
                    stack.push(Frame::Lit("</mstyle></mrow>"));
                    stack.push(Frame::Node(content));
                    if ctx.options.emit_intent {
                        stack.push(Frame::Lit(
                            "<mrow intent=\"operator-name\"><mstyle mathvariant=\"normal\">",
                        ));
                    } else {
                        stack.push(Frame::Lit("<mrow><mstyle mathvariant=\"normal\">"));
                    }
                }
            }
            _ => unreachable!("expand_style called with non-style node"),
        }
        Ok(())
    }
}
