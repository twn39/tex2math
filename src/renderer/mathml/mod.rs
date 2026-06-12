use crate::ast::*;
use crate::renderer::MathRenderer;

mod basic;
mod decorated;
mod env;
mod layout;

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
