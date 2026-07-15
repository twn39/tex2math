//! Scripts, limits, and multiscripts emission.

use super::super::{MathMLRenderer, RenderCtx};
use super::Frame;
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
        stack: &mut Vec<Frame<'n, 's>>,
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

        let tag = match (render_as_limits, sub.is_some(), sup.is_some()) {
            (true, true, true) => "munderover",
            (true, true, false) => "munder",
            (true, false, true) => "mover",
            (false, true, true) => "msubsup",
            (false, true, false) => "msub",
            (false, false, true) => "msup",
            (_, false, false) => {
                stack.push(Frame::Node(base));
                return Ok(());
            }
        };

        stack.push(Frame::Owned(format!("</{}>", tag)));
        if let Some(s) = sup {
            stack.push(Frame::Node(s));
        }
        if let Some(s) = sub {
            stack.push(Frame::Node(s));
        }
        stack.push(Frame::Node(base));
        if ctx.options.emit_intent {
            stack.push(Frame::Owned(format!("<{} intent=\"scripts\">", tag)));
        } else {
            stack.push(Frame::Owned(format!("<{}>", tag)));
        }
        Ok(())
    }
}
