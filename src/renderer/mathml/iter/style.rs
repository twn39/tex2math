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
                    if ctx.options.emit_intent {
                        stack.push(Frame::Owned(format!(
                            "<mstyle mathvariant=\"{}\" intent=\"style\">",
                            escape_xml(variant.as_ref())
                        )));
                    } else {
                        stack.push(Frame::Owned(format!(
                            "<mstyle mathvariant=\"{}\">",
                            escape_xml(variant.as_ref())
                        )));
                    }
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
                if ctx.options.emit_intent {
                    stack.push(Frame::Owned(format!(
                        "<mstyle mathcolor=\"{}\" intent=\"color\">",
                        escape_xml(color.as_ref())
                    )));
                } else {
                    stack.push(Frame::Owned(format!(
                        "<mstyle mathcolor=\"{}\">",
                        escape_xml(color.as_ref())
                    )));
                }
            }
            MathNode::ColorBox { bg_color, content } => {
                stack.push(Frame::Lit("</mstyle>"));
                stack.push(Frame::Node(content));
                if ctx.options.emit_intent {
                    stack.push(Frame::Owned(format!(
                        "<mstyle mathbackground=\"{}\" intent=\"colorbox\">",
                        escape_xml(bg_color.as_ref())
                    )));
                } else {
                    stack.push(Frame::Owned(format!(
                        "<mstyle mathbackground=\"{}\">",
                        escape_xml(bg_color.as_ref())
                    )));
                }
            }
            MathNode::MathClass { class, content } => {
                let (lspace, rspace) = class.default_spaces();
                // Prefer a single spaced <mo> when the body is a simple operator/token.
                if let Some(text) = try_extract_operator_text(content) {
                    let intent = if ctx.options.emit_intent {
                        format!(" intent=\"math-class:{}\"", class.as_str())
                    } else {
                        String::new()
                    };
                    write!(
                        ctx.out,
                        "<mo lspace=\"{}\" rspace=\"{}\"{}>{}</mo>",
                        lspace,
                        rspace,
                        intent,
                        escape_xml(text.as_ref())
                    )?;
                } else {
                    stack.push(Frame::Lit("</mrow>"));
                    stack.push(Frame::Node(content));
                    if ctx.options.emit_intent {
                        stack.push(Frame::Owned(format!(
                            "<mrow intent=\"math-class:{}\">",
                            class.as_str()
                        )));
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
                let ds = if *displaystyle { "true" } else { "false" };
                if ctx.options.emit_intent {
                    let intent = if *displaystyle {
                        "displaystyle"
                    } else {
                        "textstyle"
                    };
                    stack.push(Frame::Owned(format!(
                        "<mstyle displaystyle=\"{}\" intent=\"{}\">",
                        ds, intent
                    )));
                } else {
                    stack.push(Frame::Owned(format!("<mstyle displaystyle=\"{}\">", ds)));
                }
            }
            MathNode::Phantom { kind, content } => {
                let intent_attr = if ctx.options.emit_intent {
                    match kind {
                        PhantomKind::Invisible => " intent=\"phantom\"",
                        PhantomKind::Vertical => " intent=\"vphantom\"",
                        PhantomKind::Horizontal => " intent=\"hphantom\"",
                    }
                } else {
                    ""
                };
                match kind {
                    PhantomKind::Invisible => {
                        stack.push(Frame::Lit("</mphantom>"));
                        stack.push(Frame::Node(content));
                        if intent_attr.is_empty() {
                            stack.push(Frame::Lit("<mphantom>"));
                        } else {
                            stack.push(Frame::Owned(format!("<mphantom{intent_attr}>")));
                        }
                    }
                    PhantomKind::Vertical => {
                        stack.push(Frame::Lit("</mphantom></mpadded>"));
                        stack.push(Frame::Node(content));
                        if intent_attr.is_empty() {
                            stack.push(Frame::Lit("<mpadded width=\"0px\"><mphantom>"));
                        } else {
                            stack.push(Frame::Owned(format!(
                                "<mpadded width=\"0px\"{intent_attr}><mphantom>"
                            )));
                        }
                    }
                    PhantomKind::Horizontal => {
                        stack.push(Frame::Lit("</mphantom></mpadded>"));
                        stack.push(Frame::Node(content));
                        if intent_attr.is_empty() {
                            stack.push(Frame::Lit(
                                "<mpadded height=\"0px\" depth=\"0px\"><mphantom>",
                            ));
                        } else {
                            stack.push(Frame::Owned(format!(
                                "<mpadded height=\"0px\" depth=\"0px\"{intent_attr}><mphantom>"
                            )));
                        }
                    }
                }
            }
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
                    if ctx.options.emit_intent {
                        stack.push(Frame::Lit("<mover intent=\"stretch-over\">"));
                    } else {
                        stack.push(Frame::Lit("<mover>"));
                    }
                } else {
                    stack.push(Frame::Lit("</munder>"));
                    stack.push(Frame::Owned(stretchy));
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
