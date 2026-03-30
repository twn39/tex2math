#![allow(clippy::needless_lifetimes)]
#![allow(clippy::new_without_default)]
#![allow(clippy::redundant_pattern_matching)]
use winnow::ascii::{alpha1, digit1, space0};
use winnow::combinator::{alt, delimited, opt, preceded, repeat, separated, trace};
use winnow::prelude::*;
use winnow::token::{literal, one_of, take_till, take_until};

mod symbols;

// ==========================================
// 1. AST (抽象语法树) 定义
// ==========================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderMode {
    Inline,
    Display,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LimitBehavior {
    Default,
    Limits,   // 强制 \limits (总是生成 munderover)
    NoLimits, // 强制 \nolimits (总是生成 msubsup)
}

#[derive(Debug, Clone, PartialEq)]
pub enum MathNode {
    Number(String),
    Identifier(String),
    Operator(String),
    Fraction(Box<MathNode>, Box<MathNode>),

    // 我们将所有上下标和上下界合并成一个更智能、更通用的统一节点
    // 之前我们分为 SubSup 和 UnderOver，现在我们在生成时动态决定它们！
    Scripts {
        base: Box<MathNode>,
        sub: Option<Box<MathNode>>,
        sup: Option<Box<MathNode>>,
        // 新增：用于张量和前置角标
        pre_sub: Option<Box<MathNode>>,
        pre_sup: Option<Box<MathNode>>,
        behavior: LimitBehavior,
        is_large_op: bool,
    },

    Row(Vec<MathNode>),
    Sqrt(Box<MathNode>),
    Root {
        index: Box<MathNode>,
        content: Box<MathNode>,
    },
    Fenced {
        open: String,
        content: Box<MathNode>,
        close: String,
    },
    Environment {
        name: String,
        format: Option<String>,
        rows: Vec<(Vec<MathNode>, Option<String>)>,
    },
    Text(String),
    Style {
        variant: String,
        content: Box<MathNode>,
    },
    Accent {
        mark: String,
        content: Box<MathNode>,
    },
    Function(String),
    Space(String),

    // == 新增：高级文本处理与颜色系统 ==
    Color {
        color: String,
        content: Box<MathNode>,
    },
    ColorBox {
        bg_color: String,
        content: Box<MathNode>,
    },
    Boxed(Box<MathNode>), // 边框

    // == 新增：隐形占位符与约分划线 ==
    Phantom(Box<MathNode>),
    Cancel {
        mode: String, // 对应 notation 的属性值
        content: Box<MathNode>,
    },

    // == 新增：可拉伸跨度修饰符 ==
    StretchOp {
        op: String,
        is_over: bool,
        content: Box<MathNode>,
    },

    Error(String),
}
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
        winnow::token::one_of(|c: char| c.is_ascii_alphabetic())
            .map(|c: char| MathNode::Identifier(c.to_string())),
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
    trace("parse_fraction", |input: &mut &'s str| {
        let _ = literal("\\frac").parse_next(input)?;

        // 辅助函数：解析一个 { ... } 块。如果没有右括号，不报错，而是吸收剩余所有字符并报错。
        let mut parse_block = |inp: &mut &'s str| -> ModalResult<MathNode> {
            let _ = preceded(space0, literal("{")).parse_next(inp)?;
            let content = parse_row.parse_next(inp)?;

            if opt(preceded(space0, literal("}")))
                .parse_next(inp)?
                .is_some()
            {
                Ok(content)
            } else {
                let remaining = winnow::token::rest.parse_next(inp)?;
                Ok(MathNode::Row(vec![
                    content,
                    MathNode::Error(format!("Missing '}}' in fraction, found: '{}'", remaining)),
                ]))
            }
        };

        // 第一个块：分子 (若匹配不到左括号，直接失败，因为这不符合 \frac 的特征)
        let num = parse_block.parse_next(input)?;

        // 第二个块：分母 (如果连左括号都没有，那说明整个格式残缺，我们将分母作为一个空的错误)
        let den = if let Ok(d) = parse_block.parse_next(input) {
            d
        } else {
            MathNode::Error("Missing denominator block".to_string())
        };

        Ok(MathNode::Fraction(Box::new(num), Box::new(den)))
    })
    .parse_next(input)
}

pub fn parse_group<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace("parse_group", |input: &mut &'s str| {
        // 匹配左大括号
        let _ = preceded(space0, literal("{")).parse_next(input)?;

        // 尝试正常解析一行内容
        let content = parse_row.parse_next(input)?;

        // 尝试匹配右大括号
        if opt(preceded(space0, literal("}")))
            .parse_next(input)?
            .is_some()
        {
            Ok(content)
        } else {
            // == 错误恢复 ==
            // 如果没找到右大括号，说明公式残缺。我们把剩下的内容视为错误节点。
            // 但为了让外层能继续渲染已正确解析的部分，我们返回一个包含了 Error 的 Row。
            let remaining = winnow::token::rest.parse_next(input)?;
            Ok(MathNode::Row(vec![
                content,
                MathNode::Error(format!("Missing '}}', found: '{}'", remaining)),
            ]))
        }
    })
    .parse_next(input)
}

// == 新增：解析 \sqrt 和 \sqrt[3] ==
pub fn parse_sqrt<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace("parse_sqrt", |input: &mut &'s str| {
        let _ = literal("\\sqrt").parse_next(input)?;

        // 提取 [ 和 ] 之间的纯字符串（不让内部的 parse_row 贪婪吃掉外面的 ]）
        let index_str_opt =
            opt(delimited((space0, '['), take_till(0.., |c| c == ']'), ']')).parse_next(input)?;

        let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;

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
            preceded(
                (literal("\\left"), space0),
                one_of(['(', '[', '{', '|', '.']),
            ),
            delimited(space0, parse_row, space0),
            preceded(
                (literal("\\right"), space0),
                one_of([')', ']', '}', '|', '.']),
            ),
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
        // 提取命令名：可以是英文字母组成的词，也可以是特定的单字符标点符号
        let cmd = preceded(
            '\\',
            alt((
                alpha1,
                // 支持 \%, \$, \{ 等特殊单字符命令
                one_of([
                    ',', ';', ':', '!', '%', '$', '#', '&', '_', ' ', '{', '}', '|',
                ])
                .map(|c: char| c.to_string().leak() as &str),
            )),
        )
        .parse_next(input)?;

        // 1. 处理带参数的高级命令 (文本、样式、重音)
        // \text{...} 文本模式：内部必须原样保留空格，不进行数学递归解析
        if cmd == "text" || cmd == "mathrm" {
            let inner_text =
                delimited((space0, '{'), take_until(0.., "}"), '}').parse_next(input)?;
            return Ok(MathNode::Text(inner_text.to_string()));
        }

        // == 新增：颜色与高亮盒子 ==
        if cmd == "textcolor" {
            // \textcolor{red}{content}
            let color = delimited((space0, '{'), take_until(0.., "}"), '}').parse_next(input)?;
            let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
            return Ok(MathNode::Color {
                color: color.to_string(),
                content: Box::new(content),
            });
        }

        if cmd == "colorbox" {
            // \colorbox{#FF0000}{content}
            let bg_color = delimited((space0, '{'), take_until(0.., "}"), '}').parse_next(input)?;
            let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
            return Ok(MathNode::ColorBox {
                bg_color: bg_color.to_string(),
                content: Box::new(content),
            });
        }

        if cmd == "boxed" {
            // \boxed{content}
            let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
            return Ok(MathNode::Boxed(Box::new(content)));
        }

        if cmd == "phantom" {
            // \phantom{content}
            let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
            return Ok(MathNode::Phantom(Box::new(content)));
        }

        let cancel_mode = match cmd {
            "cancel" => Some("updiagonalstrike"),
            "bcancel" => Some("downdiagonalstrike"),
            "xcancel" => Some("updiagonalstrike downdiagonalstrike"),
            _ => None,
        };
        if let Some(mode) = cancel_mode {
            let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
            return Ok(MathNode::Cancel {
                mode: mode.to_string(),
                content: Box::new(content),
            });
        }

        // == 新增：可拉伸的修饰符 ==
        let stretch_info = match cmd {
            "underbrace" => Some(("⏟", false)),
            "overbrace" => Some(("⏞", true)),
            "underline" => Some(("_", false)),
            "overline" => Some(("¯", true)),
            _ => None,
        };
        if let Some((op_str, is_over)) = stretch_info {
            let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
            return Ok(MathNode::StretchOp {
                op: op_str.to_string(),
                is_over,
                content: Box::new(content),
            });
        }

        if cmd == "color" {
            // \color{red} ... 从此处开始吃掉当前级别的剩余所有元素
            let color = delimited((space0, '{'), take_until(0.., "}"), '}').parse_next(input)?;

            // 巧妙的贪婪：读取从这里到作用域结束（如 } 或者 EOF）的所有后续节点
            // 由于 parse_row 是由当前层级调用的，为了不污染外层的 '}' 终止符，
            // 我们不能直接用 parse_row 吃到文件末尾。在真正的 PEG 中，
            // 这是极其依赖上下文状态的（Stateful）。
            // 作为一个轻量级实现，我们可以让它只解析接下来的一个 Atom 或 Group 块：
            // （如果用户写 \color{red} x + y，在我们的极简版里可能需要写成 \color{red}{x + y} 或隔离在括号中）
            // 我们采取折衷：在 tex2math 中，遇到 \color{red} 时，如果它紧跟着括号，
            // 就直接把它当做 textcolor 对待；否则它只影响下一个 Node。
            // 实际上为了通过标准测试 test_parse_color_switch，我们需要让它贪婪消耗当前环境剩余的内容。
            // 为了安全，我们先提取剩下的所有能被 parse_node 消费的东西，这本质上就是剩下的 row：
            let remaining_nodes: Vec<MathNode> =
                repeat(0.., preceded(space0, parse_node)).parse_next(input)?;

            let content = if remaining_nodes.len() == 1 {
                remaining_nodes.into_iter().next().unwrap()
            } else {
                MathNode::Row(remaining_nodes)
            };

            return Ok(MathNode::Color {
                color: color.to_string(),
                content: Box::new(content),
            });
        }

        // 字体样式命令：其内部是一个标准的数学表达式 (Row)
        let style_variant = match cmd {
            "mathbf" => Some("bold"),
            "mathit" => Some("italic"),
            "mathbb" => Some("double-struck"),
            "mathcal" => Some("script"),
            "mathfrak" => Some("fraktur"),
            _ => None,
        };
        if let Some(variant) = style_variant {
            let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;
            return Ok(MathNode::Style {
                variant: variant.to_string(),
                content: Box::new(content),
            });
        }

        // 数学重音修饰符 (Accents)
        let accent_mark = match cmd {
            "hat" | "widehat" => Some("^"),
            "vec" => Some("→"),
            "bar" | "overline" => Some("¯"),
            "dot" => Some("˙"),
            "ddot" => Some("¨"),
            "tilde" | "widetilde" => Some("~"),
            _ => None,
        };
        if let Some(mark) = accent_mark {
            // 重音的参数可以是一个字符，也可以是大括号包裹的表达式
            let content = alt((
                delimited((space0, '{'), parse_row, (space0, '}')),
                // 允许 \hat x (不带括号)
                parse_ident,
            ))
            .parse_next(input)?;

            return Ok(MathNode::Accent {
                mark: mark.to_string(),
                content: Box::new(content),
            });
        }

        // 2. 处理无参数的纯静态字典映射
        match cmd {
            // == 显式排版空格 ==
            "quad" => return Ok(MathNode::Space("1em".to_string())),
            "qquad" => return Ok(MathNode::Space("2em".to_string())),
            "enspace" | "enskip" => return Ok(MathNode::Space("0.5em".to_string())),
            "," | "thinspace" => return Ok(MathNode::Space("0.1667em".to_string())),
            ":" | "medspace" => return Ok(MathNode::Space("0.2222em".to_string())),
            ";" | "thickspace" => return Ok(MathNode::Space("0.2778em".to_string())),
            "!" | "negthinspace" => return Ok(MathNode::Space("-0.1667em".to_string())),

            // == 标准数学函数 ==
            "sin" | "cos" | "tan" | "csc" | "sec" | "cot" | "arcsin" | "arccos" | "arctan"
            | "sinh" | "cosh" | "tanh" | "exp" | "log" | "ln" | "lg" | "lim" | "limsup"
            | "liminf" | "max" | "min" | "sup" | "inf" | "det" | "arg" | "dim" => {
                return Ok(MathNode::Function(cmd.to_string()))
            }

            // 排除专门的结构命令，让它们去各自的解析器里匹配
            "frac" | "sqrt" | "left" | "right" => {
                return winnow::combinator::fail.parse_next(input);
            }
            _ => {}
        }

        // 3. 去外部超级大字典里查找 (希腊字母、特殊符号、箭头、大运算符)
        if let Some(node) = symbols::lookup_symbol(cmd) {
            return Ok(node);
        }

        // 如果哪里都不在，原样保留为未识别命令 (Fallback)
        Ok(MathNode::Identifier(format!("\\{}", cmd)))
    })
    .parse_next(input)
}

// == 新增：解析 LaTeX Environment (\begin...\end) ==
pub fn parse_environment<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace("parse_environment", |input: &mut &'s str| {
        let begin_tag = preceded((literal("\\begin"), space0, '{'), alpha1).parse_next(input)?;
        let name = begin_tag.to_string();
        let _ = literal("}").parse_next(input)?;

        let mut format = None;
        if name == "array" {
            // 使用 opt 来利用上下文进行推导，这样编译器就能知道 Error 是 winnow::error::ContextError
            let fmt_opt: Option<&str> =
                opt(delimited((space0, '{'), take_until(0.., "}"), '}')).parse_next(input)?;
            if let Some(fmt_str) = fmt_opt {
                format = Some(fmt_str.to_string());
            }
        }

        let end_pattern = format!("\\end{{{}}}", name);
        let inner_str_result: ModalResult<&str> =
            take_until(0.., end_pattern.as_str()).parse_next(input);

        let (mut inner_str, is_closed) = match inner_str_result {
            Ok(s) => {
                let _ = literal(end_pattern.as_str()).parse_next(input)?;
                (s, true)
            }
            Err(_) => {
                let s = winnow::token::rest.parse_next(input)?;
                (s, false)
            }
        };

        let mut parse_cells_in_row = |row_input: &mut &str| -> ModalResult<Vec<MathNode>> {
            separated(
                0..,
                delimited(space0, parse_row, space0),
                (space0, '&', space0),
            )
            .parse_next(row_input)
        };

        // 指定类型为 ModalResult<Option<&str>>
        let mut parse_newline_opt = |input: &mut &'s str| -> ModalResult<Option<&str>> {
            preceded(
                literal("\\\\"),
                opt(delimited((space0, '['), take_until(0.., "]"), ']')),
            )
            .parse_next(input)
        };

        let mut rows: Vec<(Vec<MathNode>, Option<String>)> = Vec::new();

        loop {
            let row_content_res: ModalResult<&str> =
                take_until(0.., "\\\\").parse_next(&mut inner_str);
            let mut row_content: &str = if let Ok(content) = row_content_res {
                content
            } else {
                winnow::token::rest.parse_next(&mut inner_str)?
            };

            if let Ok(cells) = parse_cells_in_row.parse_next(&mut row_content) {
                let spacing = if let Ok(opt_spacing) = parse_newline_opt.parse_next(&mut inner_str)
                {
                    opt_spacing.map(|s: &str| s.to_string())
                } else {
                    None
                };

                if !cells.is_empty() || spacing.is_some() || rows.is_empty() {
                    rows.push((cells, spacing));
                }
            }

            if inner_str.is_empty() {
                break;
            }
        }

        let env_node = MathNode::Environment {
            name: name.clone(),
            format,
            rows,
        };

        if !is_closed {
            Ok(MathNode::Row(vec![
                env_node,
                MathNode::Error(format!("Missing \\end{{{}}}", name)),
            ]))
        } else {
            Ok(env_node)
        }
    })
    .parse_next(input)
}

pub fn parse_atom<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace(
        "parse_atom",
        alt((
            parse_environment, // 环境优先级最高
            parse_left_right,
            parse_fraction,
            parse_sqrt,
            parse_group,
            parse_command, // 将通用命令解析器加入！
            parse_ident,
            parse_number,
        )),
    )
    .parse_next(input)
}

pub fn parse_script<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace("parse_script", |input: &mut &'s str| {
        let base = parse_atom.parse_next(input)?;

        // 探测 base 之后是否紧跟 \limits 或 \nolimits (这些通常用于覆盖默认的上下标排版)
        let behavior = if let Ok(_) =
            literal::<&str, &str, winnow::error::ContextError>("\\limits").parse_next(input)
        {
            LimitBehavior::Limits
        } else if let Ok(_) =
            literal::<&str, &str, winnow::error::ContextError>("\\nolimits").parse_next(input)
        {
            LimitBehavior::NoLimits
        } else {
            LimitBehavior::Default
        };

        let mut sub = None;
        let mut sup = None;

        loop {
            if sup.is_none() {
                if let Some(s) =
                    opt(preceded((space0, '^', space0), parse_atom)).parse_next(input)?
                {
                    sup = Some(s);
                    continue;
                }
            }
            if sub.is_none() {
                if let Some(s) =
                    opt(preceded((space0, '_', space0), parse_atom)).parse_next(input)?
                {
                    sub = Some(s);
                    continue;
                }
            }
            break;
        }

        // 判断 base 是否是要求使用 limits 渲染的大运算符或极限函数
        let is_large_operator = match &base {
            MathNode::Operator(op) => ["∑", "∏", "∐", "∫", "∮"].contains(&op.as_str()),
            MathNode::Function(f) => {
                ["lim", "limsup", "liminf", "max", "min", "sup", "inf"].contains(&f.as_str())
            }
            MathNode::StretchOp { .. } => true, // 拉伸括号必须把它后面的附着物当成 limits
            _ => false,
        };

        if sub.is_none() && sup.is_none() && behavior == LimitBehavior::Default {
            return Ok(base);
        }

        Ok(MathNode::Scripts {
            base: Box::new(base),
            sub: sub.map(Box::new),
            sup: sup.map(Box::new),
            behavior,
            is_large_op: is_large_operator,
            pre_sub: None,
            pre_sup: None,
        })
    })
    .parse_next(input)
}

pub fn parse_node<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace("parse_node", alt((parse_script, parse_operator))).parse_next(input)
}

pub fn parse_row<'s>(input: &mut &'s str) -> ModalResult<MathNode> {
    trace(
        "parse_row",
        repeat(0.., preceded(space0, parse_node)).map(|nodes: Vec<MathNode>| {
            // == AST 智能折叠 Pass: 张量与前置角标 ==
            // 扫描平铺的数组，寻找: [Scripts(base: 空Row, sub: A, sup: B), Identifier(X)]
            // 并将它们合并为: Scripts(base: X, pre_sub: A, pre_sup: B)
            let mut folded_nodes: Vec<MathNode> = Vec::with_capacity(nodes.len());
            let mut i = 0;

            while i < nodes.len() {
                if i + 1 < nodes.len() {
                    // 检查当前节点是否是一个空基底的角标
                    if let MathNode::Scripts {
                        base,
                        sub,
                        sup,
                        pre_sub: None,
                        pre_sup: None,
                        behavior: LimitBehavior::Default,
                        ..
                    } = &nodes[i]
                    {
                        if let MathNode::Row(inner) = &**base {
                            if inner.is_empty() {
                                // 发现了一个完美的前置角标载体！
                                let next_node = nodes[i + 1].clone();

                                // 取出下一个节点。如果下一个节点本身也是一个 Scripts 节点，
                                // 我们就把前置角标合并进它的 pre_sub/pre_sup 里！
                                let merged_node = match next_node {
                                    MathNode::Scripts {
                                        base: next_base,
                                        sub: next_sub,
                                        sup: next_sup,
                                        behavior,
                                        is_large_op,
                                        ..
                                    } => MathNode::Scripts {
                                        base: next_base,
                                        sub: next_sub,
                                        sup: next_sup,
                                        pre_sub: sub.clone(),
                                        pre_sup: sup.clone(),
                                        behavior,
                                        is_large_op,
                                    },
                                    // 如果下一个只是个普通的原子 (比如 Identifier X)，包装它！
                                    _ => MathNode::Scripts {
                                        base: Box::new(next_node),
                                        sub: None,
                                        sup: None,
                                        pre_sub: sub.clone(),
                                        pre_sup: sup.clone(),
                                        behavior: LimitBehavior::Default,
                                        is_large_op: false,
                                    },
                                };

                                folded_nodes.push(merged_node);
                                i += 2; // 跳过这两个被合并的节点
                                continue;
                            }
                        }
                    }
                }
                folded_nodes.push(nodes[i].clone());
                i += 1;
            }

            if folded_nodes.len() == 1 {
                folded_nodes.into_iter().next().unwrap()
            } else {
                MathNode::Row(folded_nodes)
            }
        }),
    )
    .parse_next(input)
}

// ==========================================
// 3. 代码生成器 (AST -> MathML)
// ==========================================

fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

// ==========================================
// 3. 抽象渲染后端 (Pluggable Backends)
// ==========================================

/// 所有后端渲染器必须实现的抽象接口
pub trait MathRenderer {
    fn render(&self, node: &MathNode, mode: RenderMode) -> String;
}

// ==========================================
// 4. 标准 MathML 渲染器实现
// ==========================================

/// tex2math 官方提供的标准 MathML 渲染后端
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
            MathNode::Operator(o) => format!("<mo>{}</mo>", escape_xml(o)),
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
                format!(
                    "<mrow>{}{}{}</mrow>",
                    mo_open,
                    self.render(content, mode),
                    mo_close
                )
            }
            MathNode::Environment { name, format, rows } => {
                let mut custom_aligns = Vec::new();
                let mut custom_lines = Vec::new();

                if let Some(fmt_str) = format {
                    for c in fmt_str.chars() {
                        match c {
                            'l' => {
                                custom_aligns.push("left");
                                custom_lines.push("none");
                            }
                            'c' => {
                                custom_aligns.push("center");
                                custom_lines.push("none");
                            }
                            'r' => {
                                custom_aligns.push("right");
                                custom_lines.push("none");
                            }
                            '|' => {
                                if let Some(last) = custom_lines.last_mut() {
                                    *last = "solid";
                                }
                            }
                            _ => {}
                        }
                    }
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
                format!(
                    "<mrow mathvariant=\"{}\">{}</mrow>",
                    escape_xml(variant),
                    self.render(content, mode)
                )
            }
            MathNode::Accent { mark, content } => {
                format!(
                    "<mover accent=\"true\">{}<mo>{}</mo></mover>",
                    self.render(content, mode),
                    escape_xml(mark)
                )
            }
            MathNode::Function(f) => format!("<mi mathvariant=\"normal\">{}</mi>", escape_xml(f)),
            MathNode::Space(width) => format!("<mspace width=\"{}\"/>", escape_xml(width)),

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
            MathNode::Phantom(content) => {
                format!("<mphantom>{}</mphantom>", self.render(content, mode))
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

#[cfg(test)]
mod tests;

/// 为了保持向后兼容性和提供一个极其简单易用的接口，
/// 这里提供一个标准的单次调用函数来使用 MathML 渲染器。
pub fn generate_mathml(node: &MathNode, mode: RenderMode) -> String {
    MathMLRenderer::new().render(node, mode)
}
