use super::basic::{escape_xml, try_extract_operator_text};
use super::MathMLRenderer;
use crate::ast::*;
use crate::renderer::MathRenderer;
use std::fmt::Write;

impl MathMLRenderer {
    pub(super) fn render_layout_node(
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

    fn render_operator_name(
        &self,
        content: &MathNode,
        mode: RenderMode,
        buf: &mut String,
    ) -> std::fmt::Result {
        if let Some(text) = try_extract_operator_text(content) {
            write!(
                buf,
                "<mi mathvariant=\"normal\">{}</mi>",
                escape_xml(text.as_ref())
            )?;
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
