//! Semantic analysis / AST folding passes (tex2math 2.x).
//!
//! Parser builds a mostly-syntactic tree. This module applies structural
//! rewrites that are easier to test and evolve independently of winnow combinators.
//!
//! ## Ordered passes (do not reorder casually)
//!
//! [`analyze`] is a **single recursive walk** (`normalize_tree`) that, at each
//! node, applies the same local rules. Row-level folding uses a fixed order:
//!
//! 1. **Prime fold** (`fold_primes_in_node` / `fold_prime_node`) — collapse
//!    prime-only script bodies (`^{\prime\prime}` → ″) before other rewrites.
//! 2. **Prescript fold** (`fold_prescripts`) — `{}_a^b X` empty-base scripts
//!    merge into `mmultiscripts` / pre-script fields.
//! 3. **Choose fold** (`fold_choose`) — infix `\choose` marker → [`MathNode::Binom`].
//! 4. **Tree normalize** — collapse trivial single-child rows; recurse into
//!    children so nested rows see the same pipeline.
//!
//! Row assembly in the parser calls [`fold_row`] with the same 1→2→3 order so
//! mid-parse and post-parse trees stay consistent.
//!
//! When adding a new rewrite: prefer a named `fold_*` function, call it from
//! [`fold_row`] / `normalize_tree` in an explicit position, and document it here.

use crate::ast::{LimitBehavior, MathNode};
use std::borrow::Cow;

/// Run all semantic passes on a parsed tree (see module-level pass order).
#[inline]
pub fn analyze<'s>(node: MathNode<'s>) -> MathNode<'s> {
    // Entry: full recursive normalize (includes primes / prescripts / choose on rows).
    normalize_tree(node)
}

/// Fold a flat list of nodes in fixed order: primes → prescripts → `\choose`.
#[inline]
pub fn fold_row<'s>(nodes: Vec<MathNode<'s>>) -> MathNode<'s> {
    // Order: (1) primes (2) prescripts (3) choose — see module docs.
    fold_choose(flatten_row_nodes(fold_prescripts(
        nodes.into_iter().map(fold_primes_in_node).collect(),
    )))
}

/// Weight of a prime glyph (`′` = 1, `″` = 2, …).
fn prime_weight(s: &str) -> Option<usize> {
    match s {
        "\u{2032}" | "'" => Some(1),
        "\u{2033}" => Some(2),
        "\u{2034}" => Some(3),
        "\u{2057}" => Some(4),
        _ => None,
    }
}

fn prime_glyph(count: usize) -> &'static str {
    match count {
        0 => "",
        1 => "\u{2032}",
        2 => "\u{2033}",
        3 => "\u{2034}",
        _ => "\u{2057}",
    }
}

/// Collapse a node that is only primes (or a row of primes) into one prime identifier.
pub fn fold_prime_node<'s>(node: MathNode<'s>) -> MathNode<'s> {
    match node {
        MathNode::Row(nodes) => {
            let mut total = 0usize;
            let mut all_primes = !nodes.is_empty();
            for n in &nodes {
                match n {
                    MathNode::Identifier(s) | MathNode::Operator(s) => {
                        if let Some(w) = prime_weight(s.as_ref()) {
                            total = total.saturating_add(w);
                        } else {
                            all_primes = false;
                            break;
                        }
                    }
                    _ => {
                        all_primes = false;
                        break;
                    }
                }
            }
            if all_primes && total > 0 {
                MathNode::Identifier(Cow::Borrowed(prime_glyph(total.min(4))))
            } else {
                MathNode::Row(nodes.into_iter().map(fold_prime_node).collect())
            }
        }
        MathNode::Identifier(s) | MathNode::Operator(s) if prime_weight(s.as_ref()).is_some() => {
            let w = prime_weight(s.as_ref()).unwrap_or(1);
            MathNode::Identifier(Cow::Borrowed(prime_glyph(w.min(4))))
        }
        other => other,
    }
}

fn fold_primes_in_node<'s>(node: MathNode<'s>) -> MathNode<'s> {
    match node {
        MathNode::Scripts {
            base,
            sub,
            sup,
            pre_sub,
            pre_sup,
            behavior,
        } => MathNode::Scripts {
            base: Box::new(fold_primes_in_node(*base)),
            sub: sub.map(|n| Box::new(fold_prime_node(fold_primes_in_node(*n)))),
            sup: sup.map(|n| Box::new(fold_prime_node(fold_primes_in_node(*n)))),
            pre_sub: pre_sub.map(|n| Box::new(fold_prime_node(fold_primes_in_node(*n)))),
            pre_sup: pre_sup.map(|n| Box::new(fold_prime_node(fold_primes_in_node(*n)))),
            behavior,
        },
        MathNode::Row(nodes) => MathNode::Row(nodes.into_iter().map(fold_primes_in_node).collect()),
        other => other,
    }
}

fn flatten_row_nodes<'s>(node: MathNode<'s>) -> Vec<MathNode<'s>> {
    match node {
        MathNode::Row(nodes) => nodes,
        other => vec![other],
    }
}

/// Fold `a \choose b` (and multi-token sides) into [`MathNode::Binom`].
pub fn fold_choose<'s>(nodes: Vec<MathNode<'s>>) -> MathNode<'s> {
    if let Some(idx) = nodes
        .iter()
        .position(|n| matches!(n, MathNode::ChooseMarker))
    {
        let mut left: Vec<MathNode<'s>> = nodes.into_iter().collect();
        let right = left.split_off(idx + 1);
        left.pop(); // remove ChooseMarker
        let upper = match left.len() {
            0 => MathNode::Row(vec![]),
            1 => left.into_iter().next().unwrap(),
            _ => MathNode::Row(left),
        };
        let lower = match right.len() {
            0 => MathNode::Row(vec![]),
            1 => right.into_iter().next().unwrap(),
            _ => MathNode::Row(right),
        };
        return MathNode::Binom(Box::new(upper), Box::new(lower));
    }
    if nodes.len() == 1 {
        nodes.into_iter().next().unwrap()
    } else {
        MathNode::Row(nodes)
    }
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
            fold_row(mapped)
        }
        MathNode::Fraction(a, b) => {
            MathNode::Fraction(Box::new(normalize_tree(*a)), Box::new(normalize_tree(*b)))
        }
        MathNode::Binom(a, b) => {
            MathNode::Binom(Box::new(normalize_tree(*a)), Box::new(normalize_tree(*b)))
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
            sub: sub.map(|n| Box::new(fold_prime_node(normalize_tree(*n)))),
            sup: sup.map(|n| Box::new(fold_prime_node(normalize_tree(*n)))),
            pre_sub: pre_sub.map(|n| Box::new(fold_prime_node(normalize_tree(*n)))),
            pre_sup: pre_sup.map(|n| Box::new(fold_prime_node(normalize_tree(*n)))),
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
        MathNode::MathClass { class, content } => MathNode::MathClass {
            class,
            content: Box::new(normalize_tree(*content)),
        },
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

    #[test]
    fn fold_double_prime_row() {
        let node = MathNode::Scripts {
            base: Box::new(MathNode::Identifier(Cow::Borrowed("x"))),
            sub: None,
            sup: Some(Box::new(MathNode::Row(vec![
                MathNode::Identifier(Cow::Borrowed("\u{2032}")),
                MathNode::Identifier(Cow::Borrowed("\u{2032}")),
            ]))),
            pre_sub: None,
            pre_sup: None,
            behavior: LimitBehavior::Default,
        };
        match normalize_tree(node) {
            MathNode::Scripts { sup: Some(s), .. } => {
                assert_eq!(*s, MathNode::Identifier(Cow::Borrowed("\u{2033}")))
            }
            other => panic!("unexpected: {other:?}"),
        }
    }
}
