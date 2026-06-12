use super::basic::escape_xml;
use super::MathMLRenderer;
use crate::ast::*;
use crate::renderer::MathRenderer;
use std::fmt::Write;

impl MathMLRenderer {
    pub(super) fn render_environment(
        &self,
        name: &str,
        format: &Option<String>,
        rows: &[(Vec<MathNode>, Option<String>)],
        mode: RenderMode,
        buf: &mut String,
    ) -> std::fmt::Result {
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
                        if !custom_aligns.is_empty() && custom_aligns.len() > 1 {
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

        if open_fence.is_some() || close_fence.is_some() || name == "cases" {
            buf.push_str("<mrow>");
        }

        if let Some(f) = open_fence {
            write!(buf, "<mo stretchy=\"true\">{}</mo>", escape_xml(f))?;
        }

        buf.push_str("<mtable");
        match name {
            "align" | "align*" | "eqnarray" | "eqnarray*" => {
                let max_cols = rows.iter().map(|(r, _)| r.len()).max().unwrap_or(0);
                let aligns: Vec<&str> = (0..max_cols)
                    .map(|i| if i % 2 == 0 { "right" } else { "left" })
                    .collect();
                if !aligns.is_empty() {
                    write!(buf, " columnalign=\"{}\"", aligns.join(" "))?;
                }
            }
            "cases" => buf.push_str(" columnalign=\"left\""),
            "array" => {
                if !custom_aligns.is_empty() {
                    write!(buf, " columnalign=\"{}\"", custom_aligns.join(" "))?;
                }
                if custom_lines.contains(&"solid") {
                    write!(buf, " columnlines=\"{}\"", custom_lines.join(" "))?;
                }
            }
            _ => {}
        };
        buf.push('>');

        for (row, spacing) in rows {
            buf.push_str("<mtr");
            if let Some(space) = spacing {
                write!(buf, " style=\"margin-bottom: {};\"", escape_xml(space))?;
            }
            buf.push('>');
            for cell in row {
                buf.push_str("<mtd>");
                self.render_into(cell, mode, buf)?;
                buf.push_str("</mtd>");
            }
            buf.push_str("</mtr>");
        }
        buf.push_str("</mtable>");

        if let Some(f) = close_fence {
            write!(buf, "<mo stretchy=\"true\">{}</mo>", escape_xml(f))?;
        }

        if open_fence.is_some() || close_fence.is_some() || name == "cases" {
            buf.push_str("</mrow>");
        }
        Ok(())
    }
}
