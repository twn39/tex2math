use winnow::ascii::{alpha1, digit1, space0};
use winnow::combinator::{alt, delimited, opt, preceded, repeat, trace};
use winnow::prelude::*;
use winnow::token::{literal, one_of, take_till};

// ==========================================
// 1. AST (抽象语法树) 定义
// ==========================================
#[derive(Debug, Clone, PartialEq)]
pub enum MathNode {
    Number(String),
    Identifier(String),
    Operator(String),
    Fraction(Box<MathNode>, Box<MathNode>),
    Superscript(Box<MathNode>, Box<MathNode>),
    Subscript(Box<MathNode>, Box<MathNode>),
    SubSup {
        base: Box<MathNode>,
        sub: Box<MathNode>,
        sup: Box<MathNode>,
    },
    Row(Vec<MathNode>),
    // == 新增的 AST 节点 ==
    Sqrt(Box<MathNode>),                   // 对应 <msqrt>
    Root {                                 // 对应 <mroot>
        index: Box<MathNode>,
        content: Box<MathNode>,
    },
    Fenced {                               // 对应 \left \right 包裹的结构
        open: String,
        content: Box<MathNode>,
        close: String,
    },
}

// ==========================================
// 2. Winnow 解析器 (Parser)
// ==========================================

pub fn parse_number<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace(
        "parse_number",
        digit1.map(|s: &str| MathNode::Number(s.to_string())),
    )
    .parse_next(input)
}

pub fn parse_ident<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace(
        "parse_ident",
        alpha1.map(|s: &str| MathNode::Identifier(s.to_string())),
    )
    .parse_next(input)
}

pub fn parse_operator<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace(
        "parse_operator",
        one_of(['+', '-', '=', '<', '>', '(', ')', '[', ']', '|', ','])
            .map(|c: char| MathNode::Operator(c.to_string())),
    )
    .parse_next(input)
}

pub fn parse_fraction<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace(
        "parse_fraction",
        preceded(
            literal("\\frac"),
            (
                delimited((space0, '{'), parse_row, (space0, '}')),
                delimited((space0, '{'), parse_row, (space0, '}')),
            ),
        )
        .map(|(num, den)| MathNode::Fraction(Box::new(num), Box::new(den))),
    )
    .parse_next(input)
}

pub fn parse_group<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace(
        "parse_group",
        delimited((literal("{"), space0), parse_row, (space0, literal("}"))),
    )
    .parse_next(input)
}

// == 新增：解析 \sqrt 和 \sqrt[3] ==
pub fn parse_sqrt<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace("parse_sqrt", |input: &mut &'s str| {
        let _ = literal("\\sqrt").parse_next(input)?;
        
        // 提取 [ 和 ] 之间的纯字符串（不让内部的 parse_row 贪婪吃掉外面的 ]）
        let index_str_opt = opt(delimited(
            (space0, '['),
            take_till(0.., |c| c == ']'),
            ']'
        )).parse_next(input)?;
            
        let content = delimited((space0, '{'), parse_row, (space0, '}'))
            .parse_next(input)?;

        if let Some(mut idx_str) = index_str_opt {
            // 对提取出来的字符串进行 AST 解析
            let index_node = parse_row.parse_next(&mut idx_str)?;
            Ok(MathNode::Root {
                index: Box::new(index_node),
                content: Box::new(content),
            })
        } else {
            Ok(MathNode::Sqrt(Box::new(content)))
        }
    })
    .parse_next(input)
}

// == 新增：解析 \left 和 \right ==
pub fn parse_left_right<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace(
        "parse_left_right",
        (
            preceded((literal("\\left"), space0), one_of(['(', '[', '{', '|'])),
            delimited(space0, parse_row, space0),
            preceded((literal("\\right"), space0), one_of([')', ']', '}', '|'])),
        )
        .map(|(open, content, close)| MathNode::Fenced {
            open: open.to_string(),
            content: Box::new(content),
            close: close.to_string(),
        }),
    )
    .parse_next(input)
}

// == 新增：命令符号字典映射 ==
// 参考了 KaTeX 和 texmath 的底层字典，将 LaTeX 命令映射为等价的 Unicode 字符
pub fn parse_command<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace("parse_command", |input: &mut &'s str| {
        let cmd = preceded('\\', alpha1).parse_next(input)?;
        match cmd {
            // 希腊字母 (映射为 Identifier)
            "alpha" => Ok(MathNode::Identifier("α".to_string())),
            "beta" => Ok(MathNode::Identifier("β".to_string())),
            "gamma" => Ok(MathNode::Identifier("γ".to_string())),
            "delta" => Ok(MathNode::Identifier("δ".to_string())),
            "epsilon" => Ok(MathNode::Identifier("ε".to_string())),
            "theta" => Ok(MathNode::Identifier("θ".to_string())),
            "lambda" => Ok(MathNode::Identifier("λ".to_string())),
            "mu" => Ok(MathNode::Identifier("μ".to_string())),
            "pi" => Ok(MathNode::Identifier("π".to_string())),
            "sigma" => Ok(MathNode::Identifier("σ".to_string())),
            "phi" => Ok(MathNode::Identifier("φ".to_string())),
            "omega" => Ok(MathNode::Identifier("ω".to_string())),
            "Gamma" => Ok(MathNode::Identifier("Γ".to_string())),
            "Delta" => Ok(MathNode::Identifier("Δ".to_string())),
            "Theta" => Ok(MathNode::Identifier("Θ".to_string())),
            "Lambda" => Ok(MathNode::Identifier("Λ".to_string())),
            "Pi" => Ok(MathNode::Identifier("Π".to_string())),
            "Sigma" => Ok(MathNode::Identifier("Σ".to_string())),
            "Phi" => Ok(MathNode::Identifier("Φ".to_string())),
            "Omega" => Ok(MathNode::Identifier("Ω".to_string())),

            // 常见数学符号 (作为 Identifier)
            "infty" => Ok(MathNode::Identifier("∞".to_string())),
            "partial" => Ok(MathNode::Identifier("∂".to_string())),
            "nabla" => Ok(MathNode::Identifier("∇".to_string())),
            "emptyset" => Ok(MathNode::Identifier("∅".to_string())),

            // 操作符与关系符 (映射为 Operator)
            "pm" => Ok(MathNode::Operator("±".to_string())),
            "mp" => Ok(MathNode::Operator("∓".to_string())),
            "times" => Ok(MathNode::Operator("×".to_string())),
            "div" => Ok(MathNode::Operator("÷".to_string())),
            "cdot" => Ok(MathNode::Operator("·".to_string())),
            "ast" => Ok(MathNode::Operator("*".to_string())),
            "star" => Ok(MathNode::Operator("⋆".to_string())),
            "oplus" => Ok(MathNode::Operator("⊕".to_string())),
            "otimes" => Ok(MathNode::Operator("⊗".to_string())),
            
            // 不等与等价
            "leq" | "le" => Ok(MathNode::Operator("≤".to_string())),
            "geq" | "ge" => Ok(MathNode::Operator("≥".to_string())),
            "neq" | "ne" => Ok(MathNode::Operator("≠".to_string())),
            "approx" => Ok(MathNode::Operator("≈".to_string())),
            "equiv" => Ok(MathNode::Operator("≡".to_string())),
            "sim" => Ok(MathNode::Operator("∼".to_string())),
            "cong" => Ok(MathNode::Operator("≅".to_string())),
            
            // 逻辑与集合
            "forall" => Ok(MathNode::Operator("∀".to_string())),
            "exists" => Ok(MathNode::Operator("∃".to_string())),
            "in" => Ok(MathNode::Operator("∈".to_string())),
            "notin" => Ok(MathNode::Operator("∉".to_string())),
            "subset" => Ok(MathNode::Operator("⊂".to_string())),
            "subseteq" => Ok(MathNode::Operator("⊆".to_string())),
            "cup" => Ok(MathNode::Operator("∪".to_string())),
            "cap" => Ok(MathNode::Operator("∩".to_string())),

            // 箭头
            "leftarrow" | "gets" => Ok(MathNode::Operator("←".to_string())),
            "rightarrow" | "to" => Ok(MathNode::Operator("→".to_string())),
            "leftrightarrow" => Ok(MathNode::Operator("↔".to_string())),
            
            // 大运算符
            "sum" => Ok(MathNode::Operator("∑".to_string())),
            "prod" => Ok(MathNode::Operator("∏".to_string())),
            "coprod" => Ok(MathNode::Operator("∐".to_string())),
            "int" => Ok(MathNode::Operator("∫".to_string())),
            "oint" => Ok(MathNode::Operator("∮".to_string())),
            
            // 排除专门的结构命令，让它们去各自的解析器里匹配
            "frac" | "sqrt" | "left" | "right" => {
                winnow::combinator::fail.parse_next(input)
            }
            // 如果不在字典中，原样保留为未识别命令 (Fallback)
            _ => Ok(MathNode::Identifier(format!("\\{}", cmd))),
        }
    })
    .parse_next(input)
}

pub fn parse_atom<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace(
        "parse_atom",
        alt((
            parse_left_right, 
            parse_fraction, 
            parse_sqrt,       
            parse_group, 
            parse_command,    // 将通用命令解析器加入！
            parse_ident, 
            parse_number
        )),
    )
    .parse_next(input)
}

pub fn parse_script<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace("parse_script", |input: &mut &'s str| {
        let base = parse_atom.parse_next(input)?;

        let mut sub = None;
        let mut sup = None;

        loop {
            if sup.is_none() {
                if let Some(s) = opt(preceded((space0, '^', space0), parse_atom)).parse_next(input)? {
                    sup = Some(s);
                    continue;
                }
            }
            if sub.is_none() {
                if let Some(s) = opt(preceded((space0, '_', space0), parse_atom)).parse_next(input)? {
                    sub = Some(s);
                    continue;
                }
            }
            break;
        }

        match (sub, sup) {
            (Some(sub), Some(sup)) => Ok(MathNode::SubSup {
                base: Box::new(base),
                sub: Box::new(sub),
                sup: Box::new(sup),
            }),
            (Some(sub), None) => Ok(MathNode::Subscript(Box::new(base), Box::new(sub))),
            (None, Some(sup)) => Ok(MathNode::Superscript(Box::new(base), Box::new(sup))),
            (None, None) => Ok(base),
        }
    })
    .parse_next(input)
}

pub fn parse_node<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace(
        "parse_node",
        alt((parse_script, parse_operator)),
    )
    .parse_next(input)
}

pub fn parse_row<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace(
        "parse_row",
        repeat(1.., preceded(space0, parse_node)).map(|nodes: Vec<MathNode>| {
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
        MathNode::Operator(o) => format!("<mo>{}</mo>", o),
        MathNode::Fraction(num, den) => {
            format!("<mfrac>{}{}</mfrac>", generate_mathml(num), generate_mathml(den))
        }
        MathNode::Superscript(base, sup) => {
            format!("<msup>{}{}</msup>", generate_mathml(base), generate_mathml(sup))
        }
        MathNode::Subscript(base, sub) => {
            format!("<msub>{}{}</msub>", generate_mathml(base), generate_mathml(sub))
        }
        MathNode::SubSup { base, sub, sup } => {
            format!(
                "<msubsup>{}{}{}</msubsup>",
                generate_mathml(base), generate_mathml(sub), generate_mathml(sup)
            )
        }
        MathNode::Row(nodes) => {
            let inner: String = nodes.iter().map(generate_mathml).collect();
            format!("<mrow>{}</mrow>", inner)
        }
        // == 新增的生成逻辑 ==
        MathNode::Sqrt(content) => {
            format!("<msqrt>{}</msqrt>", generate_mathml(content))
        }
        MathNode::Root { index, content } => {
            // 注意 MathML <mroot> 的顺序是：先内容，后指数！
            format!("<mroot>{}{}</mroot>", generate_mathml(content), generate_mathml(index))
        }
        MathNode::Fenced { open, content, close } => {
            format!(
                "<mrow><mo stretchy=\"true\">{}</mo>{}<mo stretchy=\"true\">{}</mo></mrow>",
                open, generate_mathml(content), close
            )
        }
    }
}

// ==========================================

#[cfg(test)]
mod tests;
