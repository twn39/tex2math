//! Scripts, limits, and multiscripts emission.

use super::super::{MathMLRenderer, RenderCtx};
use super::{Frame, FrameStack};
use crate::ast::*;
use std::fmt;

impl MathMLRenderer {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn expand_scripts<'n, 's>(
        &self,
        base: &'n MathNode<'s>,
        sub: &'n Option<Box<MathNode<'s>>>,
        sup: &'n Option<Box<MathNode<'s>>>,
        pre_sub: &'n Option<Box<MathNode<'s>>>,
        pre_sup: &'n Option<Box<MathNode<'s>>>,
        behavior: LimitBehavior,
        ctx: &RenderCtx<'_>,
        stack: &mut FrameStack<'n, 's>,
    ) -> fmt::Result {
        if pre_sub.is_some() || pre_sup.is_some() {
            stack.push(Frame::Lit("</mmultiscripts>"));
            if let Some(s) = pre_sup {
                stack.push(Frame::Node(s));
            } else {
                stack.push(Frame::Lit("<none/>"));
            }
            if let Some(s) = pre_sub {
                stack.push(Frame::Node(s));
            } else {
                stack.push(Frame::Lit("<none/>"));
            }
            stack.push(Frame::Lit("<mprescripts/>"));
            if let Some(s) = sup {
                stack.push(Frame::Node(s));
            } else {
                stack.push(Frame::Lit("<none/>"));
            }
            if let Some(s) = sub {
                stack.push(Frame::Node(s));
            } else {
                stack.push(Frame::Lit("<none/>"));
            }
            stack.push(Frame::Node(base));
            if ctx.options.emit_intent {
                stack.push(Frame::Lit("<mmultiscripts intent=\"scripts\">"));
            } else {
                stack.push(Frame::Lit("<mmultiscripts>"));
            }
            return Ok(());
        }

        let render_as_limits = match behavior {
            LimitBehavior::Limits => true,
            LimitBehavior::NoLimits => false,
            LimitBehavior::Default => base.is_large_op() && ctx.mode == RenderMode::Display,
        };

        let (open_tag, close_tag) = match (render_as_limits, sub.is_some(), sup.is_some()) {
            (true, true, true) => {
                if ctx.options.emit_intent {
                    ("<munderover intent=\"scripts\">", "</munderover>")
                } else {
                    ("<munderover>", "</munderover>")
                }
            }
            (true, true, false) => {
                if ctx.options.emit_intent {
                    ("<munder intent=\"scripts\">", "</munder>")
                } else {
                    ("<munder>", "</munder>")
                }
            }
            (true, false, true) => {
                if ctx.options.emit_intent {
                    ("<mover intent=\"scripts\">", "</mover>")
                } else {
                    ("<mover>", "</mover>")
                }
            }
            (false, true, true) => {
                if ctx.options.emit_intent {
                    ("<msubsup intent=\"scripts\">", "</msubsup>")
                } else {
                    ("<msubsup>", "</msubsup>")
                }
            }
            (false, true, false) => {
                if ctx.options.emit_intent {
                    ("<msub intent=\"scripts\">", "</msub>")
                } else {
                    ("<msub>", "</msub>")
                }
            }
            (false, false, true) => {
                if ctx.options.emit_intent {
                    ("<msup intent=\"scripts\">", "</msup>")
                } else {
                    ("<msup>", "</msup>")
                }
            }
            (_, false, false) => {
                stack.push(Frame::Node(base));
                return Ok(());
            }
        };

        stack.push(Frame::Lit(close_tag));
        if let Some(s) = sup {
            stack.push(Frame::Node(s));
        }
        if let Some(s) = sub {
            stack.push(Frame::Node(s));
        }
        stack.push(Frame::Node(base));
        stack.push(Frame::Lit(open_tag));
        Ok(())
    }
}
