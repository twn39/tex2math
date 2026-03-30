use winnow::ascii::{alpha1, digit1, space0};
use winnow::combinator::{alt, delimited, preceded, repeat, trace};
use winnow::prelude::*;
use winnow::token::literal;

// ==========================================
// 1. AST (抽象语法树) 定义
// ==========================================
#[derive(Debug, Clone, PartialEq)]
pub enum MathNode {
    Number(String),                         // 对应 <mn>
    Identifier(String),                     // 对应 <mi>
    Fraction(Box<MathNode>, Box<MathNode>), // 对应 <mfrac>
    Row(Vec<MathNode>),                     // 对应 <mrow>，用于包裹同级多个元素
}

// ==========================================
// 2. Winnow 解析器 (Parser)
// ==========================================

/// 解析数字: "123" -> Number("123")
fn parse_number<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace(
        "parse_number",
        digit1.map(|s: &str| MathNode::Number(s.to_string())),
    )
    .parse_next(input)
}

/// 解析标识符 (字母): "a" -> Identifier("a")
fn parse_ident<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace(
        "parse_ident",
        alpha1.map(|s: &str| MathNode::Identifier(s.to_string())),
    )
    .parse_next(input)
}

/// 解析分数: "\frac{a}{b}" -> Fraction(a, b)
fn parse_fraction<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace(
        "parse_fraction",
        preceded(
            // 匹配 "\frac"
            literal("\\frac"),
            // 匹配两个花括号包裹的内容，并在任何地方允许空格
            (
                delimited((space0, '{'), parse_row, (space0, '}')),
                delimited((space0, '{'), parse_row, (space0, '}')),
            ),
        )
        // 将匹配到的分子和分母映射到 AST 节点
        .map(|(num, den)| MathNode::Fraction(Box::new(num), Box::new(den))),
    )
    .parse_next(input)
}

/// 解析单个节点 (尝试匹配分数、字母或数字)
fn parse_node<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace(
        "parse_node",
        alt((
            parse_fraction,
            parse_ident,
            parse_number,
        )),
    )
    .parse_next(input)
}

/// 解析一行 (Row): 核心入口，处理并列的多个元素
fn parse_row<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace(
        "parse_row",
        // 重复匹配 1 次或多次节点，忽略前导空格
        repeat(1.., preceded(space0, parse_node)).map(|nodes: Vec<MathNode>| {
            // 如果只有一个节点，就不需要 <mrow> 包裹，直接返回该节点
            if nodes.len() == 1 {
                nodes.into_iter().next().unwrap()
            } else {
                MathNode::Row(nodes)
            }
        }),
    )
    .parse_next(input)
}

// ==========================================
// 3. 代码生成器 (AST -> MathML)
// ==========================================
pub fn generate_mathml(node: &MathNode) -> String {
    match node {
        MathNode::Number(n) => format!("<mn>{}</mn>", n),
        MathNode::Identifier(i) => format!("<mi>{}</mi>", i),
        MathNode::Fraction(num, den) => {
            format!(
                "<mfrac>{}{}</mfrac>",
                generate_mathml(num),
                generate_mathml(den)
            )
        }
        MathNode::Row(nodes) => {
            let inner: String = nodes.iter().map(generate_mathml).collect();
            format!("<mrow>{}</mrow>", inner)
        }
    }
}


// ==========================================

#[cfg(test)]
mod tests;
