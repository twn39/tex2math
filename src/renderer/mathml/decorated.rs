use super::basic::escape_xml;
use super::MathMLRenderer;
use crate::ast::*;
use crate::renderer::MathRenderer;
use std::fmt::Write;

impl MathMLRenderer {
    pub(super) fn render_decorated_node(
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
}
