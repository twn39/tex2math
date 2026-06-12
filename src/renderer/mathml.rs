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

    // 1. 基础叶子原子节点渲染
    fn render_basic_node(
        &self,
        node: &MathNode,
        _mode: RenderMode,
        buf: &mut String,
    ) -> std::fmt::Result {
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
            MathNode::Text(t) => write!(buf, "<mtext>{}</mtext>", escape_xml(t)),
            MathNode::Space(width) => write!(buf, "<mspace width=\"{}\"/>", escape_xml(width)),
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
            _ => unreachable!(),
        }
    }

    // 2. 局部修饰/样式节点路由
    fn render_decorated_node(
        &self,
        node: &MathNode,
        mode: RenderMode,
        buf: &mut String,
    ) -> std::fmt::Result {
        match node {
            MathNode::Style { variant, content } => self.render_style(variant, content, mode, buf),
            MathNode::Accent { mark, content } => self.render_accent(mark, content, mode, buf),
            MathNode::Color { color, content } => self.render_color(color, content, mode, buf),
            MathNode::ColorBox { bg_color, content } => {
                self.render_color_box(bg_color, content, mode, buf)
            }
            MathNode::Boxed(content) => self.render_boxed(content, mode, buf),
            MathNode::Cancel {
                mode: c_mode,
                content,
            } => self.render_cancel(c_mode, content, mode, buf),
            MathNode::Error(err_msg) => self.render_error(err_msg, buf),
            _ => unreachable!(),
        }
    }

    // 3. 复杂布局与环境节点路由
    fn render_layout_node(
        &self,
        node: &MathNode,
        mode: RenderMode,
        buf: &mut String,
    ) -> std::fmt::Result {
        match node {
            MathNode::Fraction(num, den) => self.render_fraction(num, den, mode, buf),
            MathNode::Scripts {
                base,
                sub,
                sup,
                pre_sub,
                pre_sup,
                behavior,
            } => self.render_scripts(base, sub, sup, pre_sub, pre_sup, *behavior, mode, buf),
            MathNode::Row(nodes) => self.render_row(nodes, mode, buf),
            MathNode::Sqrt(content) => self.render_sqrt(content, mode, buf),
            MathNode::Root { index, content } => self.render_root(index, content, mode, buf),
            MathNode::Fenced {
                open,
                content,
                close,
            } => self.render_fenced(open, content, close, mode, buf),
            MathNode::Environment { name, format, rows } => {
                self.render_environment(name, format, rows, mode, buf)
            }
            MathNode::OperatorName(content) => self.render_operator_name(content, mode, buf),
            MathNode::SizedDelimiter { size, delim } => {
                self.render_sized_delimiter(size, delim, buf)
            }
            MathNode::Phantom { kind, content } => self.render_phantom(kind, content, mode, buf),
            MathNode::StretchOp {
                op,
                is_over,
                content,
            } => self.render_stretch_op(op, *is_over, content, mode, buf),
            MathNode::StyledMath {
                displaystyle,
                content,
            } => self.render_styled_math(*displaystyle, content, mode, buf),
            _ => unreachable!(),
        }
    }

    // --- 各具体变体子渲染方法 ---

    fn render_style(
        &self,
        variant: &str,
        content: &MathNode,
        mode: RenderMode,
        buf: &mut String,
    ) -> std::fmt::Result {
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

    fn render_accent(
        &self,
        mark: &str,
        content: &MathNode,
        mode: RenderMode,
        buf: &mut String,
    ) -> std::fmt::Result {
        buf.push_str("<mover accent=\"true\">");
        self.render_into(content, mode, buf)?;
        write!(buf, "<mo>{}</mo></mover>", escape_xml(mark))
    }

    fn render_color(
        &self,
        color: &str,
        content: &MathNode,
        mode: RenderMode,
        buf: &mut String,
    ) -> std::fmt::Result {
        write!(buf, "<mstyle mathcolor=\"{}\">", escape_xml(color))?;
        self.render_into(content, mode, buf)?;
        buf.push_str("</mstyle>");
        Ok(())
    }

    fn render_color_box(
        &self,
        bg_color: &str,
        content: &MathNode,
        mode: RenderMode,
        buf: &mut String,
    ) -> std::fmt::Result {
        write!(buf, "<mstyle mathbackground=\"{}\">", escape_xml(bg_color))?;
        self.render_into(content, mode, buf)?;
        buf.push_str("</mstyle>");
        Ok(())
    }

    fn render_boxed(
        &self,
        content: &MathNode,
        mode: RenderMode,
        buf: &mut String,
    ) -> std::fmt::Result {
        buf.push_str("<menclose notation=\"box\">");
        self.render_into(content, mode, buf)?;
        buf.push_str("</menclose>");
        Ok(())
    }

    fn render_cancel(
        &self,
        notation_mode: &str,
        content: &MathNode,
        mode: RenderMode,
        buf: &mut String,
    ) -> std::fmt::Result {
        write!(buf, "<menclose notation=\"{}\">", escape_xml(notation_mode))?;
        self.render_into(content, mode, buf)?;
        buf.push_str("</menclose>");
        Ok(())
    }

    fn render_error(&self, err_msg: &str, buf: &mut String) -> std::fmt::Result {
        write!(
            buf,
            "<merror><mtext mathcolor=\"red\">Syntax Error: {}</mtext></merror>",
            escape_xml(err_msg)
        )
    }

    fn render_fraction(
        &self,
        num: &MathNode,
        den: &MathNode,
        mode: RenderMode,
        buf: &mut String,
    ) -> std::fmt::Result {
        buf.push_str("<mfrac>");
        self.render_into(num, mode, buf)?;
        self.render_into(den, mode, buf)?;
        buf.push_str("</mfrac>");
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn render_scripts(
        &self,
        base: &MathNode,
        sub: &Option<Box<MathNode>>,
        sup: &Option<Box<MathNode>>,
        pre_sub: &Option<Box<MathNode>>,
        pre_sup: &Option<Box<MathNode>>,
        behavior: LimitBehavior,
        mode: RenderMode,
        buf: &mut String,
    ) -> std::fmt::Result {
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

    fn render_row(
        &self,
        nodes: &[MathNode],
        mode: RenderMode,
        buf: &mut String,
    ) -> std::fmt::Result {
        buf.push_str("<mrow>");
        for n in nodes {
            self.render_into(n, mode, buf)?;
        }
        buf.push_str("</mrow>");
        Ok(())
    }

    fn render_sqrt(
        &self,
        content: &MathNode,
        mode: RenderMode,
        buf: &mut String,
    ) -> std::fmt::Result {
        buf.push_str("<msqrt>");
        self.render_into(content, mode, buf)?;
        buf.push_str("</msqrt>");
        Ok(())
    }

    fn render_root(
        &self,
        index: &MathNode,
        content: &MathNode,
        mode: RenderMode,
        buf: &mut String,
    ) -> std::fmt::Result {
        buf.push_str("<mroot>");
        self.render_into(content, mode, buf)?;
        self.render_into(index, mode, buf)?;
        buf.push_str("</mroot>");
        Ok(())
    }

    fn render_fenced(
        &self,
        open: &str,
        content: &MathNode,
        close: &str,
        mode: RenderMode,
        buf: &mut String,
    ) -> std::fmt::Result {
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

    fn render_environment(
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

    fn render_operator_name(
        &self,
        content: &MathNode,
        mode: RenderMode,
        buf: &mut String,
    ) -> std::fmt::Result {
        if let Some(text) = try_extract_operator_text(content) {
            write!(buf, "<mi mathvariant=\"normal\">{}</mi>", escape_xml(&text))?;
        } else {
            buf.push_str("<mrow><mstyle mathvariant=\"normal\">");
            self.render_into(content, mode, buf)?;
            buf.push_str("</mstyle></mrow>");
        }
        Ok(())
    }

    fn render_sized_delimiter(
        &self,
        size: &str,
        delim: &str,
        buf: &mut String,
    ) -> std::fmt::Result {
        let esc_size = escape_xml(size);
        write!(
            buf,
            "<mo minsize=\"{}\" maxsize=\"{}\">{}</mo>",
            esc_size,
            esc_size,
            escape_xml(delim)
        )
    }

    fn render_phantom(
        &self,
        kind: &PhantomKind,
        content: &MathNode,
        mode: RenderMode,
        buf: &mut String,
    ) -> std::fmt::Result {
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

    fn render_stretch_op(
        &self,
        op: &str,
        is_over: bool,
        content: &MathNode,
        mode: RenderMode,
        buf: &mut String,
    ) -> std::fmt::Result {
        let stretchy_op = format!("<mo stretchy=\"true\">{}</mo>", escape_xml(op));
        if is_over {
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

    fn render_styled_math(
        &self,
        displaystyle: bool,
        content: &MathNode,
        mode: RenderMode,
        buf: &mut String,
    ) -> std::fmt::Result {
        let ds = if displaystyle { "true" } else { "false" };
        write!(buf, "<mstyle displaystyle=\"{}\">", ds)?;
        self.render_into(content, mode, buf)?;
        buf.push_str("</mstyle>");
        Ok(())
    }
}

impl MathRenderer for MathMLRenderer {
    fn render_into(&self, node: &MathNode, mode: RenderMode, buf: &mut String) -> std::fmt::Result {
        match node {
            // 1. 基础叶子原子节点
            MathNode::Number(_)
            | MathNode::Identifier(_)
            | MathNode::Operator(_)
            | MathNode::Text(_)
            | MathNode::Space(_)
            | MathNode::Function(_) => self.render_basic_node(node, mode, buf),

            // 2. 局部修饰与样式变换节点
            MathNode::Style { .. }
            | MathNode::Accent { .. }
            | MathNode::Color { .. }
            | MathNode::ColorBox { .. }
            | MathNode::Boxed(_)
            | MathNode::Cancel { .. }
            | MathNode::Error(_) => self.render_decorated_node(node, mode, buf),

            // 3. 复杂布局与环境节点
            MathNode::Fraction(..)
            | MathNode::Scripts { .. }
            | MathNode::Row(_)
            | MathNode::Sqrt(_)
            | MathNode::Root { .. }
            | MathNode::Fenced { .. }
            | MathNode::Environment { .. }
            | MathNode::OperatorName(_)
            | MathNode::SizedDelimiter { .. }
            | MathNode::Phantom { .. }
            | MathNode::StretchOp { .. }
            | MathNode::StyledMath { .. } => self.render_layout_node(node, mode, buf),
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
        MathNode::Identifier(s)
        | MathNode::Number(s)
        | MathNode::Operator(s)
        | MathNode::Function(s) => Some(s.clone()),
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
