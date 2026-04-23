use crate::ast::*;
use crate::renderer::MathRenderer;
use std::fmt::Write;

struct EscapedXml<'a>(&'a str);

impl<'a> std::fmt::Display for EscapedXml<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for c in self.0.chars() {
            match c {
                '&' => f.write_str("&amp;")?,
                '<' => f.write_str("&lt;")?,
                '>' => f.write_str("&gt;")?,
                '\'' => f.write_str("&apos;")?,
                '"' => f.write_str("&quot;")?,
                _ => f.write_char(c)?,
            }
        }
        Ok(())
    }
}

#[inline]
fn escape_xml(input: &str) -> EscapedXml<'_> {
    EscapedXml(input)
}

/// The standard MathML rendering backend provided by tex2math.
///
/// Converts a `MathNode` AST into a MathML XML string.
pub struct MathMLRenderer;

impl MathMLRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl MathRenderer for MathMLRenderer {
    fn render_into(&self, node: &MathNode, mode: RenderMode, buf: &mut String) -> std::fmt::Result {
        match node {
            MathNode::Number(n) => write!(buf, "<mn>{}</mn>", escape_xml(n)),
            MathNode::Identifier(i) => write!(buf, "<mi>{}</mi>", escape_xml(i)),
            MathNode::Operator(o) => {
                let is_arrow = [
                    "\u{2190}", "\u{2192}", "\u{2194}", "\u{21D2}", "\u{21D0}", "\u{21D4}",
                    "\u{21A6}", "\u{21A9}", "\u{21AA}", "\u{219E}", "\u{21A0}",
                ]
                .contains(&o.as_str());
                if is_arrow {
                    write!(buf, "<mo stretchy=\"true\">{}</mo>", escape_xml(o))
                } else {
                    write!(buf, "<mo>{}</mo>", escape_xml(o))
                }
            }
            MathNode::Fraction(num, den) => {
                buf.push_str("<mfrac>");
                self.render_into(num, mode, buf)?;
                self.render_into(den, mode, buf)?;
                buf.push_str("</mfrac>");
                Ok(())
            }
            MathNode::Scripts {
                base,
                sub,
                sup,
                pre_sub,
                pre_sup,
                behavior,
            } => {
                if pre_sub.is_some() || pre_sup.is_some() {
                    buf.push_str("<mmultiscripts>");
                    self.render_into(base, mode, buf)?;

                    if let Some(s) = sub {
                        self.render_into(s, mode, buf)?;
                    } else {
                        buf.push_str("<none/>");
                    }
                    if let Some(s) = sup {
                        self.render_into(s, mode, buf)?;
                    } else {
                        buf.push_str("<none/>");
                    }

                    buf.push_str("<mprescripts/>");

                    if let Some(s) = pre_sub {
                        self.render_into(s, mode, buf)?;
                    } else {
                        buf.push_str("<none/>");
                    }
                    if let Some(s) = pre_sup {
                        self.render_into(s, mode, buf)?;
                    } else {
                        buf.push_str("<none/>");
                    }

                    buf.push_str("</mmultiscripts>");
                    return Ok(());
                }

                let render_as_limits = match behavior {
                    LimitBehavior::Limits => true,
                    LimitBehavior::NoLimits => false,
                    LimitBehavior::Default => base.is_large_op() && mode == RenderMode::Display,
                };

                let tag = match (render_as_limits, sub.is_some(), sup.is_some()) {
                    (true, true, true) => "munderover",
                    (true, true, false) => "munder",
                    (true, false, true) => "mover",
                    (false, true, true) => "msubsup",
                    (false, true, false) => "msub",
                    (false, false, true) => "msup",
                    (_, false, false) => {
                        self.render_into(base, mode, buf)?;
                        return Ok(());
                    }
                };

                write!(buf, "<{}>", tag)?;
                self.render_into(base, mode, buf)?;
                if let Some(s) = sub {
                    self.render_into(s, mode, buf)?;
                }
                if let Some(s) = sup {
                    self.render_into(s, mode, buf)?;
                }
                write!(buf, "</{}>", tag)?;
                Ok(())
            }
            MathNode::Row(nodes) => {
                buf.push_str("<mrow>");
                for n in nodes {
                    self.render_into(n, mode, buf)?;
                }
                buf.push_str("</mrow>");
                Ok(())
            }
            MathNode::Sqrt(content) => {
                buf.push_str("<msqrt>");
                self.render_into(content, mode, buf)?;
                buf.push_str("</msqrt>");
                Ok(())
            }
            MathNode::Root { index, content } => {
                buf.push_str("<mroot>");
                self.render_into(content, mode, buf)?;
                self.render_into(index, mode, buf)?;
                buf.push_str("</mroot>");
                Ok(())
            }
            MathNode::Fenced {
                open,
                content,
                close,
            } => {
                buf.push_str("<mrow>");
                if open != "." {
                    write!(buf, "<mo stretchy=\"true\">{}</mo>", escape_xml(open))?;
                }
                buf.push_str("<mrow>");
                self.render_into(content, mode, buf)?;
                buf.push_str("</mrow>");
                if close != "." {
                    write!(buf, "<mo stretchy=\"true\">{}</mo>", escape_xml(close))?;
                }
                buf.push_str("</mrow>");
                Ok(())
            }
            MathNode::Environment { name, format, rows } => {
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

                let (open_fence, close_fence) = match name.as_str() {
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
                match name.as_str() {
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
            MathNode::Text(t) => write!(buf, "<mtext>{}</mtext>", escape_xml(t)),
            MathNode::Style { variant, content } => {
                if variant == "vphantom" {
                    buf.push_str("<mpadded width=\"0px\"><mphantom>");
                    self.render_into(content, mode, buf)?;
                    buf.push_str("</mphantom></mpadded>");
                    Ok(())
                } else if variant == "hphantom" {
                    buf.push_str("<mpadded height=\"0px\" depth=\"0px\"><mphantom>");
                    self.render_into(content, mode, buf)?;
                    buf.push_str("</mphantom></mpadded>");
                    Ok(())
                } else {
                    write!(buf, "<mstyle mathvariant=\"{}\">", escape_xml(variant))?;
                    self.render_into(content, mode, buf)?;
                    buf.push_str("</mstyle>");
                    Ok(())
                }
            }
            MathNode::Accent { mark, content } => {
                buf.push_str("<mover accent=\"true\">");
                self.render_into(content, mode, buf)?;
                write!(buf, "<mo>{}</mo></mover>", escape_xml(mark))
            }
            MathNode::Function(f) => {
                let func_text = match f.as_str() {
                    "injlim" => "inj lim",
                    "projlim" => "proj lim",
                    _ => f.as_str(),
                };
                write!(
                    buf,
                    "<mi mathvariant=\"normal\">{}</mi>",
                    escape_xml(func_text)
                )
            }
            MathNode::OperatorName(content) => {
                if let Some(text) = try_extract_operator_text(content) {
                    write!(
                        buf,
                        "<mi mathvariant=\"normal\">{}</mi>",
                        escape_xml(&text)
                    )?;
                } else {
                    buf.push_str("<mrow><mstyle mathvariant=\"normal\">");
                    self.render_into(content, mode, buf)?;
                    buf.push_str("</mstyle></mrow>");
                }
                Ok(())
            }
            MathNode::SizedDelimiter { size, delim } => {
                let esc_size = escape_xml(size);
                write!(
                    buf,
                    "<mo minsize=\"{}\" maxsize=\"{}\">{}</mo>",
                    esc_size,
                    esc_size,
                    escape_xml(delim)
                )
            }
            MathNode::Space(width) => write!(buf, "<mspace width=\"{}\"/>", escape_xml(width)),

            MathNode::Color { color, content } => {
                write!(buf, "<mstyle mathcolor=\"{}\">", escape_xml(color))?;
                self.render_into(content, mode, buf)?;
                buf.push_str("</mstyle>");
                Ok(())
            }
            MathNode::ColorBox { bg_color, content } => {
                write!(buf, "<mstyle mathbackground=\"{}\">", escape_xml(bg_color))?;
                self.render_into(content, mode, buf)?;
                buf.push_str("</mstyle>");
                Ok(())
            }
            MathNode::Boxed(content) => {
                buf.push_str("<menclose notation=\"box\">");
                self.render_into(content, mode, buf)?;
                buf.push_str("</menclose>");
                Ok(())
            }
            MathNode::Phantom { kind, content } => {
                match kind {
                    PhantomKind::Invisible => buf.push_str("<mphantom>"),
                    PhantomKind::Vertical => buf.push_str("<mpadded width=\"0px\"><mphantom>"),
                    PhantomKind::Horizontal => {
                        buf.push_str("<mpadded height=\"0px\" depth=\"0px\"><mphantom>")
                    }
                }
                self.render_into(content, mode, buf)?;
                match kind {
                    PhantomKind::Invisible => buf.push_str("</mphantom>"),
                    _ => buf.push_str("</mphantom></mpadded>"),
                }
                Ok(())
            }
            MathNode::Cancel {
                mode: notation_mode,
                content,
            } => {
                write!(buf, "<menclose notation=\"{}\">", escape_xml(notation_mode))?;
                self.render_into(content, mode, buf)?;
                buf.push_str("</menclose>");
                Ok(())
            }
            MathNode::StretchOp {
                op,
                is_over,
                content,
            } => {
                let stretchy_op = format!("<mo stretchy=\"true\">{}</mo>", escape_xml(op));
                if *is_over {
                    buf.push_str("<mover>");
                    self.render_into(content, mode, buf)?;
                    buf.push_str(&stretchy_op);
                    buf.push_str("</mover>");
                } else {
                    buf.push_str("<munder>");
                    self.render_into(content, mode, buf)?;
                    buf.push_str(&stretchy_op);
                    buf.push_str("</munder>");
                }
                Ok(())
            }
            MathNode::StyledMath {
                displaystyle,
                content,
            } => {
                let ds = if *displaystyle { "true" } else { "false" };
                write!(buf, "<mstyle displaystyle=\"{}\">", ds)?;
                self.render_into(content, mode, buf)?;
                buf.push_str("</mstyle>");
                Ok(())
            }
            MathNode::Error(err_msg) => {
                write!(
                    buf,
                    "<merror><mtext mathcolor=\"red\">Syntax Error: {}</mtext></merror>",
                    escape_xml(err_msg)
                )
            }
        }
    }
}

/// A convenience function to generate MathML from a `MathNode` AST directly.
///
/// This uses the `MathMLRenderer` under the hood to perform the translation.
/// Provides a simple, standard interface for backward compatibility.
///
/// # Arguments
/// * `node` - The root `MathNode` of the parsed formula.
/// * `mode` - The `RenderMode` (Inline or Display) determining layout rules.
///
/// # Returns
/// A `String` containing the generated MathML XML.
pub fn generate_mathml(node: &MathNode, mode: RenderMode) -> String {
    MathMLRenderer::new().render(node, mode)
}


fn try_extract_operator_text(node: &MathNode) -> Option<String> {
    match node {
        MathNode::Identifier(s) | MathNode::Number(s) | MathNode::Operator(s) | MathNode::Function(s) => Some(s.clone()),
        MathNode::Space(s) => {
            Some(match s.as_str() {
                "0.1667em" => " ".to_string(), //   (thin space, ,)
                "0.2222em" => " ".to_string(), //   (medium space, :)
                "0.2778em" => " ".to_string(), //   (thick space, ;)
                "1em" => " ".to_string(),      //   (quad)
                "2em" => " ".to_string(),      //   (qquad)
                _ => " ".to_string(),
            })
        }
        MathNode::Row(nodes) => {
            let mut text = String::new();
            for n in nodes {
                text.push_str(&try_extract_operator_text(n)?);
            }
            Some(text)
        }
        _ => None,
    }
}
