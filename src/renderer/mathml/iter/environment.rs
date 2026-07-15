//! Align / array / matrix environment emission.

use super::super::basic::escape_xml;
use super::super::{MathMLRenderer, RenderCtx};
use super::Frame;
use crate::ast::*;
use std::fmt;

impl MathMLRenderer {
    pub(super) fn expand_environment<'n, 's>(
        &self,
        name: &str,
        format: &Option<std::borrow::Cow<'_, str>>,
        rows: &'n [(Vec<MathNode<'s>>, Option<std::borrow::Cow<'s, str>>)],
        ctx: &mut RenderCtx<'_>,
        stack: &mut Vec<Frame<'n, 's>>,
    ) -> fmt::Result {
        // Build open/close strings eagerly; push cells as Node frames.
        let mut custom_aligns = Vec::new();
        let mut custom_lines = Vec::new();
        if let Some(fmt_str) = format {
            let mut pending_sep = "none";
            let mut separators: Vec<&str> = Vec::new();
            for c in fmt_str.chars() {
                match c {
                    'l' | 'c' | 'r' => {
                        let align = match c {
                            'l' => "left",
                            'c' => "center",
                            _ => "right",
                        };
                        custom_aligns.push(align);
                        if custom_aligns.len() > 1 {
                            separators.push(pending_sep);
                        }
                        pending_sep = "none";
                    }
                    '|' => pending_sep = "solid",
                    _ => {}
                }
            }
            custom_lines = separators;
        }

        let (open_fence, close_fence) = match name {
            "pmatrix" => (Some("("), Some(")")),
            "bmatrix" => (Some("["), Some("]")),
            "Bmatrix" => (Some("{"), Some("}")),
            "vmatrix" => (Some("|"), Some("|")),
            "Vmatrix" => (Some("‖"), Some("‖")),
            "cases" => (Some("{"), None),
            _ => (None, None),
        };

        let wrap = open_fence.is_some() || close_fence.is_some() || name == "cases";

        if wrap {
            stack.push(Frame::Lit("</mrow>"));
        }
        if let Some(f) = close_fence {
            stack.push(Frame::Owned(format!(
                "<mo stretchy=\"true\">{}</mo>",
                escape_xml(f)
            )));
        }
        stack.push(Frame::Lit("</mtable>"));

        for (row, spacing) in rows.iter().rev() {
            stack.push(Frame::Lit("</mtr>"));
            for cell in row.iter().rev() {
                stack.push(Frame::Lit("</mtd>"));
                stack.push(Frame::Node(cell));
                stack.push(Frame::Lit("<mtd>"));
            }
            if let Some(space) = spacing {
                stack.push(Frame::Owned(format!(
                    "<mtr style=\"margin-bottom: {};\">",
                    escape_xml(space.as_ref())
                )));
            } else {
                stack.push(Frame::Lit("<mtr>"));
            }
        }

        let mut open = String::from("<mtable");
        match name {
            "align" | "align*" | "eqnarray" | "eqnarray*" => {
                let max_cols = rows.iter().map(|(r, _)| r.len()).max().unwrap_or(0);
                let aligns: Vec<&str> = (0..max_cols)
                    .map(|i| if i % 2 == 0 { "right" } else { "left" })
                    .collect();
                if !aligns.is_empty() {
                    open.push_str(&format!(" columnalign=\"{}\"", aligns.join(" ")));
                }
            }
            "cases" => open.push_str(" columnalign=\"left\""),
            "array" => {
                if !custom_aligns.is_empty() {
                    open.push_str(&format!(" columnalign=\"{}\"", custom_aligns.join(" ")));
                }
                if custom_lines.contains(&"solid") {
                    open.push_str(&format!(" columnlines=\"{}\"", custom_lines.join(" ")));
                }
            }
            _ => {}
        }
        if ctx.options.emit_intent {
            open.push_str(" intent=\"table\"");
        }
        open.push('>');
        stack.push(Frame::Owned(open));

        if let Some(f) = open_fence {
            stack.push(Frame::Owned(format!(
                "<mo stretchy=\"true\">{}</mo>",
                escape_xml(f)
            )));
        }
        if wrap {
            stack.push(Frame::Lit("<mrow>"));
        }
        Ok(())
    }
}
