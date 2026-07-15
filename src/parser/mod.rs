use std::borrow::Cow;
use winnow::ascii::{alpha1, multispace0 as space0};
use winnow::combinator::{alt, delimited, opt, preceded, repeat, trace};
use winnow::prelude::*;
use winnow::token::{literal, one_of};

use crate::ast::*;

pub mod atoms;
pub mod command;
pub mod environment;

pub use atoms::{parse_ident, parse_number, parse_operator};
pub use command::parse_command;
pub use environment::parse_environment;

/// Parses LaTeX fractions like `\frac{num}{den}`, `\dfrac{num}{den}`, and `\tfrac{num}{den}`.
pub fn parse_fraction<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    trace("parse_fraction", |input: &mut &'s str| {
        let _ = literal("\\frac").parse_next(input)?;

        // 辅助函数：解析一个 { ... } 块。如果没有右括号，不报错，而是吸收剩余所有字符并报错。
        let mut parse_block = |inp: &mut &'s str| -> ModalResult<MathNode<'s>> {
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
                    MathNode::Error(Cow::Owned(format!(
                        "Missing '}}' in fraction, found: '{}'",
                        remaining
                    ))),
                ]))
            }
        };

        // 第一个块：分子 (若匹配不到左括号，直接失败，因为这不符合 \frac 的特征)
        let num = parse_block.parse_next(input)?;

        // 第二个块：分母 (如果连左括号都没有，那说明整个格式残缺，我们将分母作为一个空的错误)
        let den = if let Ok(d) = parse_block.parse_next(input) {
            d
        } else {
            MathNode::Error(Cow::Borrowed("Missing denominator block"))
        };

        Ok(MathNode::Fraction(Box::new(num), Box::new(den)))
    })
    .parse_next(input)
}

/// Parses a square root or nth root like `\sqrt{x}` or `\sqrt[n]{x}`.
pub fn parse_sqrt<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    trace("parse_sqrt", |input: &mut &'s str| {
        let _ = literal("\\sqrt").parse_next(input)?;
        let _ = space0.parse_next(input)?;

        let mut index_node_opt = None;

        if opt(literal::<&str, &str, winnow::error::ContextError>("["))
            .parse_next(input)?
            .is_some()
        {
            let mut index_nodes = Vec::new();
            loop {
                let _ = space0.parse_next(input)?;
                if input.is_empty() || input.starts_with(']') {
                    break;
                }

                let progress = input.len();
                if let Ok(node) = parse_node.parse_next(input) {
                    index_nodes.push(node);
                } else {
                    break;
                }
                if input.len() == progress {
                    break;
                }
            }

            // 消耗右侧的 ']'
            let _ =
                opt(literal::<&str, &str, winnow::error::ContextError>("]")).parse_next(input)?;

            // 使用复用的 AST 折叠逻辑
            index_node_opt = Some(fold_row_nodes(index_nodes));
        }

        let content = delimited((space0, '{'), parse_row, (space0, '}')).parse_next(input)?;

        if let Some(index_node) = index_node_opt {
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

// 解析 \left / \right 后跟随的定界符
// 支持单字符（(, ), [, ], |, .）和命令符号（\langle, \lfloor, \lceil, \lVert 等）
pub fn parse_fence_delim<'s>(input: &mut &'s str) -> ModalResult<String> {
    preceded(
        space0,
        alt((
            // 隐形定界符
            literal(".").map(|_: &str| ".".to_string()),
            // 转义的单字符定界符：\{, \}, \| 等
            preceded('\\', one_of(['{', '}', '|', '[', ']'])).map(|c: char| c.to_string()),
            // 命令式定界符：\langle, \rangle, \lfloor, \lceil, \lVert 等
            preceded('\\', alpha1).map(|cmd: &str| {
                match cmd {
                    "langle" | "lang" => "\u{27E8}", // ⟨
                    "rangle" | "rang" => "\u{27E9}", // ⟩
                    "lfloor" => "\u{230A}",          // ⌊
                    "rfloor" => "\u{230B}",          // ⌋
                    "lceil" => "\u{2308}",           // ⌈
                    "rceil" => "\u{2309}",           // ⌉
                    "lbrace" => "{",
                    "rbrace" => "}",
                    "lbrack" => "[",
                    "rbrack" => "]",
                    "vert" | "lvert" | "rvert" => "|",
                    "Vert" | "lVert" | "rVert" => "∥", // ∥
                    "uparrow" => "\u{2191}",           // ↑
                    "downarrow" => "\u{2193}",         // ↓
                    "Uparrow" => "\u{21D1}",           // ⇑
                    "Downarrow" => "\u{21D3}",         // ⇓
                    "updownarrow" => "\u{2195}",       // ↕
                    "Updownarrow" => "\u{21D5}",       // ⇕
                    _ => cmd,
                }
                .to_string()
            }),
            // 单字符定界符
            one_of(['(', ')', '[', ']', '{', '}', '|', '<', '>']).map(|c: char| {
                match c {
                    '<' => "\u{27E8}".to_string(), // ⟨
                    '>' => "\u{27E9}".to_string(), // ⟩
                    _ => c.to_string(),
                }
            }),
        )),
    )
    .parse_next(input)
}

/// Parses dynamically sized fences like `\left( ... \right)`.
pub fn parse_left_right<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    trace("parse_left_right", |input: &mut &'s str| {
        let _ = literal("\\left").parse_next(input)?;
        let open = parse_fence_delim.parse_next(input)?;
        let content = delimited(space0, parse_row, space0).parse_next(input)?;
        // Fix 3: graceful recovery when \right is missing
        // (e.g. caused by a mis-nested \begin/\end truncating the environment body).
        // Instead of backtracking and leaving \left as an unresolvable stuck atom,
        // we emit a Fenced node with an implicit empty close delimiter '.'.
        let close = if literal::<&str, &str, winnow::error::ContextError>("\\right")
            .parse_next(input)
            .is_ok()
        {
            parse_fence_delim.parse_next(input)?
        } else {
            ".".to_string() // implicit empty close, same as \left. \right.
        };
        Ok(MathNode::Fenced {
            open: Cow::Owned(open),
            content: Box::new(content),
            close: Cow::Owned(close),
        })
    })
    .parse_next(input)
}

/// Parses an atomic mathematical element, which can be a number, identifier, operator, group, or command.
pub fn parse_atom<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    // Cap nesting so pathological input cannot blow the native stack.
    let _depth_guard = match crate::depth::DepthGuard::enter_parse() {
        Ok(g) => g,
        Err(()) => {
            return Err(winnow::error::ErrMode::Cut(
                winnow::error::ContextError::new(),
            ));
        }
    };

    trace(
        "parse_atom",
        alt((
            parse_environment, // 环境优先级最高
            parse_left_right,
            parse_fraction,
            parse_sqrt,
            atoms::parse_group,
            parse_command, // 将通用命令解析器加入！
            parse_ident,
            parse_number,
            parse_operator, // 允许单字符操作符作为 atom（例如为它添加上下标 V^* 或 \lim_{x \to 0}^+）
            atoms::parse_fallback_char, // 允许原生 Unicode 字符（如 • 或 α）
        )),
    )
    .parse_next(input)
}

/// Parses subscripts (`_`) and superscripts (`^`) attached to a base mathematical element.
pub fn parse_script<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    trace("parse_script", |input: &mut &'s str| {
        let base = match parse_atom.parse_next(input) {
            Ok(b) => b,
            Err(e) => {
                let next_char = input.chars().next();
                if next_char == Some('_') || next_char == Some('^') || next_char == Some('\'') {
                    MathNode::Row(vec![])
                } else {
                    return Err(e);
                }
            }
        };

        // 探测 base 之后是否紧跟 \limits 或 \nolimits (这些通常用于覆盖默认的上下标排版)
        let behavior = if literal::<&str, &str, winnow::error::ContextError>("\\limits")
            .parse_next(input)
            .is_ok()
        {
            LimitBehavior::Limits
        } else if literal::<&str, &str, winnow::error::ContextError>("\\nolimits")
            .parse_next(input)
            .is_ok()
        {
            LimitBehavior::NoLimits
        } else {
            LimitBehavior::Default
        };

        let mut sub = None;
        let mut sup = None;

        // == 新增：撇号（prime）支持 ==
        // x' 和 x^{\prime} 等价；x'' 对应 x^{\prime\prime}（即双撇线 ″）
        let mut prime_count = 0usize;
        while opt(one_of::<_, _, winnow::error::ContextError>('\''))
            .parse_next(input)?
            .is_some()
        {
            prime_count += 1;
        }
        if prime_count > 0 {
            let prime_char = match prime_count {
                1 => "\u{2032}", // ′
                2 => "\u{2033}", // ″
                3 => "\u{2034}", // ‴
                _ => "\u{2057}", // ⁗ (4重撇及以上)
            };
            sup = Some(MathNode::Identifier(Cow::Borrowed(prime_char)));
        }

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

        // 如果没有显式指定 \limits，且它是积分符号，我们覆盖为 NoLimits 行为 (右下/右上角标)
        // 除非用户显式写了 \limits，则保留 LimitBehavior::Limits。
        let final_behavior = if behavior == LimitBehavior::Default {
            match &base {
                MathNode::Operator(op) if crate::symbols::is_integral_symbol(op) => {
                    LimitBehavior::NoLimits
                }
                _ => behavior,
            }
        } else {
            behavior
        };

        if sub.is_none() && sup.is_none() && final_behavior == LimitBehavior::Default {
            return Ok(base);
        }

        Ok(MathNode::Scripts {
            base: Box::new(base),
            sub: sub.map(Box::new),
            sup: sup.map(Box::new),
            behavior: final_behavior,
            pre_sub: None,
            pre_sup: None,
        })
    })
    .parse_next(input)
}

/// The main parser for a single mathematical node, handling scripts, atoms, and other constructs.
///
/// Nesting deeper than [`crate::MAX_NESTING_DEPTH`] surfaces as `Err` (even if an inner
/// combinator recovered with `MathNode::Error`), so callers never observe a silently truncated tree.
pub fn parse_math<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    // Clear any stale depth flag from a previous call on this thread.
    let _ = crate::depth::take_parse_depth_exceeded();

    let result = parse_math_inner(input);

    // Inner combinators may recover with MathNode::Error and still return Ok.
    // Elevate a depth-limit hit to a hard Err so callers cannot miss it.
    if crate::depth::parse_depth_exceeded() {
        crate::depth::mark_parse_depth_exceeded();
        return Err(winnow::error::ErrMode::Cut(
            winnow::error::ContextError::new(),
        ));
    }

    result
}

fn parse_math_inner<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let mut rows: Vec<(Vec<MathNode<'s>>, Option<Cow<'s, str>>)> = Vec::new();

    loop {
        let _ = space0.parse_next(input)?;
        if input.is_empty() {
            break;
        }
        // Fix 2 (parse_math): same zero-progress guard as parse_environment.
        let progress_mark = input.len();

        if let Ok(cells) = environment::parse_cells_in_row.parse_next(input) {
            let spacing = if let Ok(opt_spacing) = environment::parse_newline_opt.parse_next(input)
            {
                opt_spacing.map(|s: &str| Cow::Borrowed(s))
            } else {
                None
            };

            let is_empty_row = cells.len() == 1
                && match &cells[0] {
                    MathNode::Row(nodes) => nodes.is_empty(),
                    _ => false,
                };

            if is_empty_row && spacing.is_none() && input.trim().is_empty() && !rows.is_empty() {
                // Ignore trailing empty row
            } else {
                rows.push((cells, spacing));
            }
        } else {
            break;
        }

        if input.is_empty() {
            break;
        }
        // If no input was consumed in this iteration, break to avoid infinite loop
        if input.len() == progress_mark {
            break;
        }
    }

    if rows.len() == 1 && rows[0].0.len() == 1 {
        // Only 1 row and 1 cell
        let (mut row_cells, _) = rows.into_iter().next().unwrap();
        Ok(row_cells.remove(0))
    } else {
        // Multi-line or aligned top-level expression
        Ok(MathNode::Environment {
            name: Cow::Borrowed("align*"), // use align* to support alternating right/left alignments without labels
            format: None,
            rows,
        })
    }
}

pub fn parse_node<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    trace("parse_node", alt((parse_script, parse_operator))).parse_next(input)
}

/// Fold row nodes (prescript / tensor pass). Delegates to [`crate::sema`].
#[inline]
pub fn fold_row_nodes<'s>(nodes: Vec<MathNode<'s>>) -> MathNode<'s> {
    crate::sema::fold_prescripts(nodes)
}

pub fn parse_row<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    trace(
        "parse_row",
        repeat(0.., preceded(space0, parse_node)).map(fold_row_nodes),
    )
    .parse_next(input)
}
