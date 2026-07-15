//! Semantic analysis / AST folding passes (tex2math 2.x).
//!
//! Parser builds a mostly-syntactic tree. This module applies structural
//! rewrites that are easier to test and evolve independently of winnow combinators.
//!
//! Current passes:
//! 1. **Prescript fold** — `{}_a^b X` empty-base scripts merge into `mmultiscripts` shape.
//! 2. **Tree normalize** — collapse trivial single-child rows; recursive walk.

use crate::ast::{LimitBehavior, MathNode};

/// Run all semantic passes on a parsed tree.
#[inline]
pub fn analyze<'s>(node: MathNode<'s>) -> MathNode<'s> {
    normalize_tree(node)
}

/// Fold adjacent empty-base scripts into tensor / prescript form.
///
/// Used while assembling rows (`parse_row`) and as part of full-tree normalize.
pub fn fold_prescripts<'s>(nodes: Vec<MathNode<'s>>) -> MathNode<'s> {
    let mut folded: Vec<MathNode<'s>> = Vec::with_capacity(nodes.len());
    let mut i = 0;

    while i < nodes.len() {
        if i + 1 < nodes.len() {
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
                        let next_node = nodes[i + 1].clone();
                        let merged = match next_node {
                            MathNode::Scripts {
                                base: next_base,
                                sub: next_sub,
                                sup: next_sup,
                                behavior,
                                ..
                            } => MathNode::Scripts {
                                base: next_base,
                                sub: next_sub,
                                sup: next_sup,
                                pre_sub: sub.clone(),
                                pre_sup: sup.clone(),
                                behavior,
                            },
                            other => MathNode::Scripts {
                                base: Box::new(other),
                                sub: None,
                                sup: None,
                                pre_sub: sub.clone(),
                                pre_sup: sup.clone(),
                                behavior: LimitBehavior::Default,
                            },
                        };
                        folded.push(merged);
                        i += 2;
                        continue;
                    }
                }
            }
        }
        folded.push(nodes[i].clone());
        i += 1;
    }

    if folded.len() == 1 {
        folded.into_iter().next().unwrap()
    } else {
        MathNode::Row(folded)
    }
}

/// Recursively normalize the tree (collapse trivial rows; re-fold row children).
pub fn normalize_tree<'s>(node: MathNode<'s>) -> MathNode<'s> {
    match node {
        MathNode::Row(nodes) => {
            let mapped: Vec<_> = nodes.into_iter().map(normalize_tree).collect();
            fold_prescripts(mapped)
        }
        MathNode::Fraction(a, b) => {
            MathNode::Fraction(Box::new(normalize_tree(*a)), Box::new(normalize_tree(*b)))
        }
        MathNode::Scripts {
            base,
            sub,
            sup,
            pre_sub,
            pre_sup,
            behavior,
        } => MathNode::Scripts {
            base: Box::new(normalize_tree(*base)),
            sub: sub.map(|n| Box::new(normalize_tree(*n))),
            sup: sup.map(|n| Box::new(normalize_tree(*n))),
            pre_sub: pre_sub.map(|n| Box::new(normalize_tree(*n))),
            pre_sup: pre_sup.map(|n| Box::new(normalize_tree(*n))),
            behavior,
        },
        MathNode::Sqrt(c) => MathNode::Sqrt(Box::new(normalize_tree(*c))),
        MathNode::Root { index, content } => MathNode::Root {
            index: Box::new(normalize_tree(*index)),
            content: Box::new(normalize_tree(*content)),
        },
        MathNode::Fenced {
            open,
            content,
            close,
        } => MathNode::Fenced {
            open,
            content: Box::new(normalize_tree(*content)),
            close,
        },
        MathNode::Environment { name, format, rows } => MathNode::Environment {
            name,
            format,
            rows: rows
                .into_iter()
                .map(|(cells, sp)| (cells.into_iter().map(normalize_tree).collect(), sp))
                .collect(),
        },
        MathNode::Style { variant, content } => MathNode::Style {
            variant,
            content: Box::new(normalize_tree(*content)),
        },
        MathNode::Accent { mark, content } => MathNode::Accent {
            mark,
            content: Box::new(normalize_tree(*content)),
        },
        MathNode::OperatorName(c) => MathNode::OperatorName(Box::new(normalize_tree(*c))),
        MathNode::Color { color, content } => MathNode::Color {
            color,
            content: Box::new(normalize_tree(*content)),
        },
        MathNode::ColorBox { bg_color, content } => MathNode::ColorBox {
            bg_color,
            content: Box::new(normalize_tree(*content)),
        },
        MathNode::Boxed(c) => MathNode::Boxed(Box::new(normalize_tree(*c))),
        MathNode::Phantom { kind, content } => MathNode::Phantom {
            kind,
            content: Box::new(normalize_tree(*content)),
        },
        MathNode::Cancel { mode, content } => MathNode::Cancel {
            mode,
            content: Box::new(normalize_tree(*content)),
        },
        MathNode::StretchOp {
            op,
            is_over,
            content,
        } => MathNode::StretchOp {
            op,
            is_over,
            content: Box::new(normalize_tree(*content)),
        },
        MathNode::StyledMath {
            displaystyle,
            content,
        } => MathNode::StyledMath {
            displaystyle,
            content: Box::new(normalize_tree(*content)),
        },
        leaf => leaf,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    #[test]
    fn fold_empty_base_scripts_into_prescripts() {
        let nodes = vec![
            MathNode::Scripts {
                base: Box::new(MathNode::Row(vec![])),
                sub: Some(Box::new(MathNode::Identifier(Cow::Borrowed("a")))),
                sup: Some(Box::new(MathNode::Identifier(Cow::Borrowed("b")))),
                pre_sub: None,
                pre_sup: None,
                behavior: LimitBehavior::Default,
            },
            MathNode::Identifier(Cow::Borrowed("X")),
        ];
        match fold_prescripts(nodes) {
            MathNode::Scripts {
                base,
                pre_sub: Some(ps),
                pre_sup: Some(pp),
                ..
            } => {
                assert_eq!(*base, MathNode::Identifier(Cow::Borrowed("X")));
                assert_eq!(*ps, MathNode::Identifier(Cow::Borrowed("a")));
                assert_eq!(*pp, MathNode::Identifier(Cow::Borrowed("b")));
            }
            other => panic!("unexpected: {other:?}"),
        }
    }
}
