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
        // Compact script-size tables (KaTeX-style smallmatrix / substack).
        let script_level = matches!(name, "smallmatrix" | "substack");

        if script_level {
            stack.push(Frame::Lit("</mstyle>"));
        }
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

        // Prefer MathML `rowspacing` (between-row gaps) over CSS on <mtr>.
        let mut rowspacings: Vec<&str> = Vec::new();
        let use_rowspacing_attr = rows.iter().any(|(_, sp)| sp.is_some());
        if use_rowspacing_attr && rows.len() > 1 {
            for (_, spacing) in rows.iter().take(rows.len() - 1) {
                rowspacings.push(spacing.as_ref().map(|s| s.as_ref()).unwrap_or("0.5ex"));
            }
        }

        for (row, spacing) in rows.iter().rev() {
            stack.push(Frame::Lit("</mtr>"));
            for cell in row.iter().rev() {
                stack.push(Frame::Lit("</mtd>"));
                stack.push(Frame::Node(cell));
                stack.push(Frame::Lit("<mtd>"));
            }
            if !use_rowspacing_attr {
                if let Some(space) = spacing {
                    stack.push(Frame::Owned(format!(
                        "<mtr style=\"margin-bottom: {};\">",
                        escape_xml(space.as_ref())
                    )));
                } else {
                    stack.push(Frame::Lit("<mtr>"));
                }
            } else {
                stack.push(Frame::Lit("<mtr>"));
            }
        }

        let mut open = String::from("<mtable");
        match name {
            // Alternating right/left columns (relation alignment).
            "align" | "align*" | "aligned" | "alignedat" | "split" | "eqnarray" | "eqnarray*" => {
                let max_cols = rows.iter().map(|(r, _)| r.len()).max().unwrap_or(0);
                let aligns: Vec<&str> = (0..max_cols)
                    .map(|i| if i % 2 == 0 { "right" } else { "left" })
                    .collect();
                if !aligns.is_empty() {
                    open.push_str(&format!(" columnalign=\"{}\"", aligns.join(" ")));
                }
            }
            "cases" => {
                // Condition column left-aligned; generous gap between value and condition.
                open.push_str(" columnalign=\"left left\" columnspacing=\"1em\"");
            }
            "gathered" | "gather" | "gather*" | "multline" | "multline*" => {
                open.push_str(" columnalign=\"center\"")
            }
            // Compact matrix for scripts / inline.
            "smallmatrix" | "substack" => {
                open.push_str(
                    " columnalign=\"center\" rowspacing=\"0.1em\" columnspacing=\"0.1667em\"",
                );
            }
            "array" => {
                if !custom_aligns.is_empty() {
                    open.push_str(&format!(" columnalign=\"{}\"", custom_aligns.join(" ")));
                }
                if custom_lines.contains(&"solid") {
                    open.push_str(&format!(" columnlines=\"{}\"", custom_lines.join(" ")));
                }
            }
            "matrix" | "pmatrix" | "bmatrix" | "Bmatrix" | "vmatrix" | "Vmatrix" => {
                open.push_str(" columnalign=\"center\"")
            }
            _ => {
                // Unknown / generic environments: center by default.
                open.push_str(" columnalign=\"center\"");
            }
        }
        if use_rowspacing_attr && !rowspacings.is_empty() {
            open.push_str(&format!(" rowspacing=\"{}\"", rowspacings.join(" ")));
        }
        if ctx.options.emit_intent {
            if crate::registry::is_known_environment(name) {
                open.push_str(" intent=\"table\"");
            } else {
                open.push_str(" intent=\"table:unknown\"");
            }
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
        if script_level {
            stack.push(Frame::Lit("<mstyle scriptlevel=\"+1\">"));
        }
        Ok(())
    }
}
