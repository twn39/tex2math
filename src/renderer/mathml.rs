use crate::ast::*;
use crate::renderer::MathRenderer;

fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\'', "&apos;")
        .replace('\"', "&quot;")
}

// ==========================================
// 4. 标准 MathML 渲染器实现
// ==========================================

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
    fn render(&self, node: &MathNode, mode: RenderMode) -> String {
        match node {
            MathNode::Number(n) => format!("<mn>{}</mn>", escape_xml(n)),
            MathNode::Identifier(i) => format!("<mi>{}</mi>", escape_xml(i)),
            MathNode::Operator(o) => {
                let is_arrow = [
                    "\u{2190}", "\u{2192}", "\u{2194}", "\u{21D2}", "\u{21D0}", "\u{21D4}",
                    "\u{21A6}", "\u{21A9}", "\u{21AA}", "\u{219E}", "\u{21A0}",
                ]
                .contains(&o.as_str());
                if is_arrow {
                    format!("<mo stretchy=\"true\">{}</mo>", escape_xml(o))
                } else {
                    format!("<mo>{}</mo>", escape_xml(o))
                }
            }
            MathNode::Fraction(num, den) => {
                format!(
                    "<mfrac>{}{}</mfrac>",
                    self.render(num, mode),
                    self.render(den, mode)
                )
            }
            MathNode::Scripts {
                base,
                sub,
                sup,
                pre_sub,
                pre_sup,
                behavior,
                is_large_op,
            } => {
                let base_str = self.render(base, mode);

                if pre_sub.is_some() || pre_sup.is_some() {
                    let s_sub = sub
                        .as_ref()
                        .map(|s| self.render(s, mode))
                        .unwrap_or_else(|| "<none/>".to_string());
                    let s_sup = sup
                        .as_ref()
                        .map(|s| self.render(s, mode))
                        .unwrap_or_else(|| "<none/>".to_string());
                    let p_sub = pre_sub
                        .as_ref()
                        .map(|s| self.render(s, mode))
                        .unwrap_or_else(|| "<none/>".to_string());
                    let p_sup = pre_sup
                        .as_ref()
                        .map(|s| self.render(s, mode))
                        .unwrap_or_else(|| "<none/>".to_string());

                    return format!(
                        "<mmultiscripts>{}{}{}<mprescripts/>{}{}</mmultiscripts>",
                        base_str, s_sub, s_sup, p_sub, p_sup
                    );
                }

                let sub_str = sub.as_ref().map(|s| self.render(s, mode));
                let sup_str = sup.as_ref().map(|s| self.render(s, mode));

                let render_as_limits = match behavior {
                    LimitBehavior::Limits => true,
                    LimitBehavior::NoLimits => false,
                    LimitBehavior::Default => *is_large_op && mode == RenderMode::Display,
                };

                match (render_as_limits, sub_str, sup_str) {
                    (true, Some(sub), Some(sup)) => {
                        format!("<munderover>{}{}{}</munderover>", base_str, sub, sup)
                    }
                    (true, Some(sub), None) => format!("<munder>{}{}</munder>", base_str, sub),
                    (true, None, Some(sup)) => format!("<mover>{}{}</mover>", base_str, sup),

                    (false, Some(sub), Some(sup)) => {
                        format!("<msubsup>{}{}{}</msubsup>", base_str, sub, sup)
                    }
                    (false, Some(sub), None) => format!("<msub>{}{}</msub>", base_str, sub),
                    (false, None, Some(sup)) => format!("<msup>{}{}</msup>", base_str, sup),

                    (_, None, None) => base_str,
                }
            }
            MathNode::Row(nodes) => {
                let inner: String = nodes.iter().map(|n| self.render(n, mode)).collect();
                format!("<mrow>{}</mrow>", inner)
            }
            MathNode::Sqrt(content) => {
                format!("<msqrt>{}</msqrt>", self.render(content, mode))
            }
            MathNode::Root { index, content } => {
                format!(
                    "<mroot>{}{}</mroot>",
                    self.render(content, mode),
                    self.render(index, mode)
                )
            }
            MathNode::Fenced {
                open,
                content,
                close,
            } => {
                let mo_open = if open == "." {
                    String::new()
                } else {
                    format!("<mo stretchy=\"true\">{}</mo>", escape_xml(open))
                };
                let mo_close = if close == "." {
                    String::new()
                } else {
                    format!("<mo stretchy=\"true\">{}</mo>", escape_xml(close))
                };
                // Wrap the rendered content in an inner <mrow> to prevent baseline shift
                // issues in WebKit/Blink when a stretchy fence immediately follows a <msup>/<msub>.
                format!(
                    "<mrow>{}<mrow>{}</mrow>{}</mrow>",
                    mo_open,
                    self.render(content, mode),
                    mo_close
                )
            }
            MathNode::Environment { name, format, rows } => {
                let mut custom_aligns = Vec::new();
                let mut custom_lines = Vec::new();

                if let Some(fmt_str) = format {
                    // 正确算法：跟踪列对齐和列间分隔符
                    // MathML columnlines 需要 N-1 个条目（N 为列数）
                    // 每个分隔符属于其左侧列的右边
                    let mut pending_sep = "none"; // 待添加的分隔符（在看到下一列字符时提交）
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
                                    // 不是第一列，提交上一列的右侧分隔符
                                    separators.push(pending_sep);
                                }
                                pending_sep = "none";
                            }
                            '|' => {
                                // |是当前列左边的分隔符（对下一列来说是左侧分隔）
                                pending_sep = "solid";
                            }
                            _ => {}
                        }
                    }
                    custom_lines = separators;
                }

                let table_attr = match name.as_str() {
                    "align" | "align*" | "eqnarray" | "eqnarray*" => {
                        let max_cols = rows.iter().map(|(r, _)| r.len()).max().unwrap_or(0);
                        let aligns: Vec<&str> = (0..max_cols)
                            .map(|i| if i % 2 == 0 { "right" } else { "left" })
                            .collect();
                        if aligns.is_empty() {
                            String::new()
                        } else {
                            format!(" columnalign=\"{}\"", aligns.join(" "))
                        }
                    }
                    "cases" => " columnalign=\"left\"".to_string(),
                    "array" => {
                        let mut attr = String::new();
                        if !custom_aligns.is_empty() {
                            attr.push_str(&format!(" columnalign=\"{}\"", custom_aligns.join(" ")));
                        }
                        // columnlines 数量应为 N-1（匹配列间分隔符数）
                        if custom_lines.contains(&"solid") {
                            attr.push_str(&format!(" columnlines=\"{}\"", custom_lines.join(" ")));
                        }
                        attr
                    }
                    _ => "".to_string(), // 默认居中 (matrix 等)
                };

                let mut table_xml = format!("<mtable{}>", table_attr);
                for (row, spacing) in rows {
                    let tr_attr = if let Some(space) = spacing {
                        format!(" style=\"margin-bottom: {};\"", escape_xml(space))
                    } else {
                        "".to_string()
                    };

                    table_xml.push_str(&format!("<mtr{}>", tr_attr));
                    for cell in row {
                        table_xml.push_str(&format!("<mtd>{}</mtd>", self.render(cell, mode)));
                    }
                    table_xml.push_str("</mtr>");
                }
                table_xml.push_str("</mtable>");

                match name.as_str() {
                    "pmatrix" => format!(
                        "<mrow><mo stretchy=\"true\">(</mo>{}<mo stretchy=\"true\">)</mo></mrow>",
                        table_xml
                    ),
                    "bmatrix" => format!(
                        "<mrow><mo stretchy=\"true\">[</mo>{}<mo stretchy=\"true\">]</mo></mrow>",
                        table_xml
                    ),
                    "Bmatrix" => format!(
                        "<mrow><mo stretchy=\"true\">{{</mo>{}<mo stretchy=\"true\">}}</mo></mrow>",
                        table_xml
                    ),
                    "vmatrix" => format!(
                        "<mrow><mo stretchy=\"true\">|</mo>{}<mo stretchy=\"true\">|</mo></mrow>",
                        table_xml
                    ),
                    "Vmatrix" => format!(
                        "<mrow><mo stretchy=\"true\">‖</mo>{}<mo stretchy=\"true\">‖</mo></mrow>",
                        table_xml
                    ),
                    "cases" => format!("<mrow><mo stretchy=\"true\">{{</mo>{}</mrow>", table_xml),
                    _ => table_xml,
                }
            }
            MathNode::Text(t) => format!("<mtext>{}</mtext>", escape_xml(t)),
            MathNode::Style { variant, content } => {
                if variant == "vphantom" {
                    // \vphantom: Height of the content, but zero width.
                    // <mphantom> makes it invisible but takes up full space.
                    // <mpadded width="0px"> makes its width zero.
                    format!(
                        "<mpadded width=\"0px\"><mphantom>{}</mphantom></mpadded>",
                        self.render(content, mode)
                    )
                } else if variant == "hphantom" {
                    // \hphantom: Width of the content, but zero height and depth.
                    format!(
                        "<mpadded height=\"0px\" depth=\"0px\"><mphantom>{}</mphantom></mpadded>",
                        self.render(content, mode)
                    )
                } else {
                    format!(
                        "<mstyle mathvariant=\"{}\">{}</mstyle>",
                        escape_xml(variant),
                        self.render(content, mode)
                    )
                }
            }
            MathNode::Accent { mark, content } => {
                format!(
                    "<mover accent=\"true\">{}<mo>{}</mo></mover>",
                    self.render(content, mode),
                    escape_xml(mark)
                )
            }
            MathNode::Function(f) => {
                let func_text = match f.as_str() {
                    "injlim" => "inj lim",
                    "projlim" => "proj lim",
                    _ => f.as_str(),
                };
                format!("<mi mathvariant=\"normal\">{}</mi>", escape_xml(func_text))
            }
            MathNode::OperatorName(content) => {
                // Do not use <mi> as a wrapper for complex layouts like <munder>,
                // because browsers (like Chrome/Safari) flatten or break layout elements inside token elements (<mi>, <mo>, <mn>).
                // Instead, use <mstyle mathvariant="normal"> wrapped in an <mrow>.
                format!(
                    "<mrow><mstyle mathvariant=\"normal\">{}</mstyle></mrow>",
                    self.render(content, mode)
                )
            }
            MathNode::SizedDelimiter { size, delim } => {
                // LaTeX \big, \Big, \bigg, \Bigg correspond to increasing sizes.
                // We use minsize and maxsize to force stretching to that exact size.
                format!(
                    "<mo minsize=\"{}\" maxsize=\"{}\">{}</mo>",
                    escape_xml(size),
                    escape_xml(size),
                    escape_xml(delim)
                )
            }
            MathNode::Space(width) => format!("<mspace width=\"{}\"/>", escape_xml(width)),
            MathNode::NewLine => "<mspace linebreak=\"newline\"/>".to_string(),

            MathNode::Color { color, content } => {
                format!(
                    "<mstyle mathcolor=\"{}\">{}</mstyle>",
                    escape_xml(color),
                    self.render(content, mode)
                )
            }
            MathNode::ColorBox { bg_color, content } => {
                format!(
                    "<mstyle mathbackground=\"{}\">{}</mstyle>",
                    escape_xml(bg_color),
                    self.render(content, mode)
                )
            }
            MathNode::Boxed(content) => {
                format!(
                    "<menclose notation=\"box\">{}</menclose>",
                    self.render(content, mode)
                )
            }
            MathNode::Phantom { kind, content } => {
                let rendered = self.render(content, mode);
                match kind {
                    PhantomKind::Invisible => format!("<mphantom>{}</mphantom>", rendered),
                    PhantomKind::Vertical => format!(
                        "<mpadded width=\"0px\"><mphantom>{}</mphantom></mpadded>",
                        rendered
                    ),
                    PhantomKind::Horizontal => format!(
                        "<mpadded height=\"0px\" depth=\"0px\"><mphantom>{}</mphantom></mpadded>",
                        rendered
                    ),
                }
            }
            MathNode::Cancel {
                mode: notation_mode,
                content,
            } => {
                format!(
                    "<menclose notation=\"{}\">{}</menclose>",
                    escape_xml(notation_mode),
                    self.render(content, mode)
                )
            }
            MathNode::StretchOp {
                op,
                is_over,
                content,
            } => {
                let stretchy_op = format!("<mo stretchy=\"true\">{}</mo>", escape_xml(op));
                let content_xml = self.render(content, mode);
                if *is_over {
                    format!("<mover>{}{}</mover>", content_xml, stretchy_op)
                } else {
                    format!("<munder>{}{}</munder>", content_xml, stretchy_op)
                }
            }
            MathNode::StyledMath {
                displaystyle,
                content,
            } => {
                // \dfrac → displaystyle="true"，\tfrac → displaystyle="false"
                let ds = if *displaystyle { "true" } else { "false" };
                format!(
                    "<mstyle displaystyle=\"{}\">{}</mstyle>",
                    ds,
                    self.render(content, mode)
                )
            }
            MathNode::Error(err_msg) => {
                format!(
                    "<merror><mtext mathcolor=\"red\">Syntax Error: {}</mtext></merror>",
                    escape_xml(err_msg)
                )
            }
        }
    }
}

// ==========================================

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
