//! Heap-stack iterative MathML emission (tex2math 2.2).
//!
//! Avoids native call-stack growth on deeply nested ASTs (e.g. thousands of
//! nested fractions). Frame work is stored on the Rust heap `Vec`.
//!
//! Expand logic is split by AST family:
//! - [`tokens`] — leaves (number, identifier, operator, text, space, …)
//! - [`structure`] — frac / root / row / fenced / boxed / cancel
//! - [`style`] — style, color, accent, phantom, stretch, operatorname
//! - [`scripts`] — scripts / limits / multiscripts
//! - [`environment`] — align / array / matrix environments

mod environment;
mod scripts;
mod structure;
mod style;
mod tokens;

use super::{MathMLRenderer, RenderCtx};
use crate::ast::*;
use std::fmt;

/// Work-list items; processed LIFO so children are pushed in reverse order.
pub(super) enum Frame<'n, 's> {
    /// Literal fragment already finalized.
    Lit(&'static str),
    /// Owned fragment (dynamic tags / escaped snippets).
    Owned(String),
    /// Expand this AST node into more frames / output.
    Node(&'n MathNode<'s>),
}

impl MathMLRenderer {
    /// Iterative entry: expand `node` onto a heap stack until drained.
    pub(super) fn render_node_iter<'n, 's>(
        &self,
        node: &'n MathNode<'s>,
        ctx: &mut RenderCtx<'_>,
    ) -> fmt::Result {
        // Cap frames to avoid unbounded memory on pathological trees.
        const MAX_FRAMES: usize = 1_000_000;
        let mut stack: Vec<Frame<'n, 's>> = Vec::with_capacity(64);
        stack.push(Frame::Node(node));

        while let Some(frame) = stack.pop() {
            if stack.len() > MAX_FRAMES {
                return write!(
                    ctx.out,
                    "<merror><mtext>Render frame limit exceeded</mtext></merror>"
                );
            }
            match frame {
                Frame::Lit(s) => ctx.out.push_str(s)?,
                Frame::Owned(s) => ctx.out.push_str(&s)?,
                Frame::Node(n) => self.expand_node(n, ctx, &mut stack)?,
            }
        }
        Ok(())
    }

    fn expand_node<'n, 's>(
        &self,
        node: &'n MathNode<'s>,
        ctx: &mut RenderCtx<'_>,
        stack: &mut Vec<Frame<'n, 's>>,
    ) -> fmt::Result {
        match node {
            // Leaves / tokens
            MathNode::Number(_)
            | MathNode::Identifier(_)
            | MathNode::Operator(_)
            | MathNode::Text(_)
            | MathNode::Space(_)
            | MathNode::Function(_)
            | MathNode::Error(_)
            | MathNode::SizedDelimiter { .. }
            | MathNode::Middle(_)
            | MathNode::ChooseMarker => self.expand_token(node, ctx),

            // Structural containers
            MathNode::Fraction(_, _)
            | MathNode::Binom(_, _)
            | MathNode::Sqrt(_)
            | MathNode::Root { .. }
            | MathNode::Row(_)
            | MathNode::Boxed(_)
            | MathNode::Cancel { .. }
            | MathNode::Fenced { .. } => self.expand_structure(node, ctx, stack),

            // Style / accent / phantom / stretch
            MathNode::Style { .. }
            | MathNode::Accent { .. }
            | MathNode::Color { .. }
            | MathNode::ColorBox { .. }
            | MathNode::StyledMath { .. }
            | MathNode::Phantom { .. }
            | MathNode::StretchOp { .. }
            | MathNode::OperatorName(_)
            | MathNode::MathClass { .. } => self.expand_style(node, ctx, stack),

            MathNode::Scripts {
                base,
                sub,
                sup,
                pre_sub,
                pre_sup,
                behavior,
            } => self.expand_scripts(base, sub, sup, pre_sub, pre_sup, *behavior, ctx, stack),

            MathNode::Environment { name, format, rows } => {
                self.expand_environment(name.as_ref(), format, rows, ctx, stack)
            }
        }
    }
}
