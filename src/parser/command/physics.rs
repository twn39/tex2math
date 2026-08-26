//! Physics macros parsing (\bra, \ket, \braket, \ketbra, \mel, \dd, \dv, \pdv, etc.)

use std::borrow::Cow;
use winnow::combinator::opt;
use winnow::prelude::*;

use crate::ast::*;
use crate::parser::atoms::{take_balanced_braces, take_balanced_brackets};
use crate::parser::parse_math;

/// Parses Dirac Bra notation: `\bra{ψ}` -> `⟨ ψ |`
pub fn parse_bra_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let (inner, _) = take_balanced_braces(input)?;
    let mut cursor = inner;
    let content = parse_math.parse_next(&mut cursor)?;
    Ok(MathNode::Fenced {
        open: Cow::Borrowed("\u{27E8}"), // ⟨ (U+27E8)
        content: Box::new(content),
        close: Cow::Borrowed("|"),
    })
}

/// Parses Dirac Ket notation: `\ket{ψ}` -> `| ψ ⟩`
pub fn parse_ket_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let (inner, _) = take_balanced_braces(input)?;
    let mut cursor = inner;
    let content = parse_math.parse_next(&mut cursor)?;
    Ok(MathNode::Fenced {
        open: Cow::Borrowed("|"),
        content: Box::new(content),
        close: Cow::Borrowed("\u{27E9}"), // ⟩ (U+27E9)
    })
}

/// Parses Dirac Bra-Ket (inner product) notation:
/// Supports both `\braket{a | b}` and `\braket{a}{b}`
pub fn parse_braket_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let (first, _) = take_balanced_braces(input)?;
    let second = opt(take_balanced_braces).parse_next(input)?;

    if let Some((sec, _)) = second {
        // Two argument form: \braket{a}{b} -> ⟨ a | b ⟩
        let mut c1 = first;
        let n1 = parse_math.parse_next(&mut c1)?;
        let mut c2 = sec;
        let n2 = parse_math.parse_next(&mut c2)?;
        Ok(MathNode::Fenced {
            open: Cow::Borrowed("\u{27E8}"),
            content: Box::new(MathNode::Row(vec![
                n1,
                MathNode::Middle(Cow::Borrowed("|")),
                n2,
            ])),
            close: Cow::Borrowed("\u{27E9}"),
        })
    } else {
        // Single argument form: \braket{a | b} or \braket{a}
        // If it contains `|`, split on `|`
        if first.contains('|') {
            let parts: Vec<&str> = first.split('|').collect();
            let mut nodes = Vec::new();
            for (i, part) in parts.iter().enumerate() {
                if i > 0 {
                    nodes.push(MathNode::Middle(Cow::Borrowed("|")));
                }
                let mut c = *part;
                nodes.push(parse_math.parse_next(&mut c)?);
            }
            Ok(MathNode::Fenced {
                open: Cow::Borrowed("\u{27E8}"),
                content: Box::new(MathNode::Row(nodes)),
                close: Cow::Borrowed("\u{27E9}"),
            })
        } else {
            let mut cursor = first;
            let content = parse_math.parse_next(&mut cursor)?;
            Ok(MathNode::Fenced {
                open: Cow::Borrowed("\u{27E8}"),
                content: Box::new(content),
                close: Cow::Borrowed("\u{27E9}"),
            })
        }
    }
}

/// Parses `\ketbra{a}{b}` or `\ketbra{a}` -> `| a ⟩⟨ b |`
pub fn parse_ketbra_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let (first, _) = take_balanced_braces(input)?;
    let second = opt(take_balanced_braces).parse_next(input)?;

    let mut c1 = first;
    let n1 = parse_math.parse_next(&mut c1)?;
    let bra_content = if let Some((sec, _)) = second {
        let mut c2 = sec;
        parse_math.parse_next(&mut c2)?
    } else {
        n1.clone()
    };

    let ket = MathNode::Fenced {
        open: Cow::Borrowed("|"),
        content: Box::new(n1),
        close: Cow::Borrowed("\u{27E9}"),
    };
    let bra = MathNode::Fenced {
        open: Cow::Borrowed("\u{27E8}"),
        content: Box::new(bra_content),
        close: Cow::Borrowed("|"),
    };
    Ok(MathNode::Row(vec![ket, bra]))
}

/// Parses Matrix Element `\mel{a}{O}{b}` or `\matrixel{a}{O}{b}` -> `⟨ a | O | b ⟩`
pub fn parse_mel_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let (first, _) = take_balanced_braces(input)?;
    let (op, _) = take_balanced_braces(input)?;
    let (third, _) = take_balanced_braces(input)?;

    let mut c1 = first;
    let n1 = parse_math.parse_next(&mut c1)?;
    let mut c2 = op;
    let n2 = parse_math.parse_next(&mut c2)?;
    let mut c3 = third;
    let n3 = parse_math.parse_next(&mut c3)?;

    Ok(MathNode::Fenced {
        open: Cow::Borrowed("\u{27E8}"),
        content: Box::new(MathNode::Row(vec![
            n1,
            MathNode::Middle(Cow::Borrowed("|")),
            n2,
            MathNode::Middle(Cow::Borrowed("|")),
            n3,
        ])),
        close: Cow::Borrowed("\u{27E9}"),
    })
}

/// Parses Expectation Value `\ev{A}` or `\expval{A}` or `\ev{A}{ψ}` -> `⟨ A ⟩` or `⟨ ψ | A | ψ ⟩`
pub fn parse_expval_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let (first, _) = take_balanced_braces(input)?;
    let second = opt(take_balanced_braces).parse_next(input)?;

    let mut c1 = first;
    let n1 = parse_math.parse_next(&mut c1)?;

    if let Some((sec, _)) = second {
        let mut c2 = sec;
        let psi = parse_math.parse_next(&mut c2)?;
        Ok(MathNode::Fenced {
            open: Cow::Borrowed("\u{27E8}"),
            content: Box::new(MathNode::Row(vec![
                psi.clone(),
                MathNode::Middle(Cow::Borrowed("|")),
                n1,
                MathNode::Middle(Cow::Borrowed("|")),
                psi,
            ])),
            close: Cow::Borrowed("\u{27E9}"),
        })
    } else {
        Ok(MathNode::Fenced {
            open: Cow::Borrowed("\u{27E8}"),
            content: Box::new(n1),
            close: Cow::Borrowed("\u{27E9}"),
        })
    }
}

/// Parses Differential `\dd[n]{x}` or `\dd{x}` -> `\mathrm{d}^n x`
pub fn parse_dd_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let order_opt = opt(take_balanced_brackets).parse_next(input)?;
    let (var, _) = take_balanced_braces(input)?;

    let d_base = MathNode::Style {
        variant: Cow::Borrowed("normal"),
        content: Box::new(MathNode::Identifier(Cow::Borrowed("d"))),
    };

    let d_op = if let Some((order_str, _)) = order_opt {
        let mut c = order_str;
        let order_node = parse_math.parse_next(&mut c)?;
        MathNode::Scripts {
            base: Box::new(d_base),
            sub: None,
            sup: Some(Box::new(order_node)),
            pre_sub: None,
            pre_sup: None,
            behavior: LimitBehavior::Default,
        }
    } else {
        d_base
    };

    let mut c = var;
    let var_node = parse_math.parse_next(&mut c)?;

    Ok(MathNode::Row(vec![d_op, var_node]))
}

/// Parses Derivative `\dv[n]{f}{x}` or `\dv{f}{x}` or `\dv{x}`
pub fn parse_dv_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let order_opt = opt(take_balanced_brackets).parse_next(input)?;
    let (first, _) = take_balanced_braces(input)?;
    let second = opt(take_balanced_braces).parse_next(input)?;

    let d_sym = || MathNode::Style {
        variant: Cow::Borrowed("normal"),
        content: Box::new(MathNode::Identifier(Cow::Borrowed("d"))),
    };

    if let Some((sec, _)) = second {
        // Two arguments: \dv[n]{f}{x} -> (d^n f) / (d x^n)
        let mut c_f = first;
        let node_f = parse_math.parse_next(&mut c_f)?;
        let mut c_x = sec;
        let node_x = parse_math.parse_next(&mut c_x)?;

        let (num, den) = if let Some((order_str, _)) = order_opt {
            let mut c1 = order_str;
            let o1 = parse_math.parse_next(&mut c1)?;
            let num = MathNode::Row(vec![
                MathNode::Scripts {
                    base: Box::new(d_sym()),
                    sub: None,
                    sup: Some(Box::new(o1.clone())),
                    pre_sub: None,
                    pre_sup: None,
                    behavior: LimitBehavior::Default,
                },
                node_f,
            ]);
            let den = MathNode::Row(vec![
                d_sym(),
                MathNode::Scripts {
                    base: Box::new(node_x),
                    sub: None,
                    sup: Some(Box::new(o1)),
                    pre_sub: None,
                    pre_sup: None,
                    behavior: LimitBehavior::Default,
                },
            ]);
            (num, den)
        } else {
            let num = MathNode::Row(vec![d_sym(), node_f]);
            let den = MathNode::Row(vec![d_sym(), node_x]);
            (num, den)
        };
        Ok(MathNode::Fraction(Box::new(num), Box::new(den)))
    } else {
        // Single argument: \dv[n]{x} -> d^n / d x^n
        let mut c_x = first;
        let node_x = parse_math.parse_next(&mut c_x)?;

        let (num, den) = if let Some((order_str, _)) = order_opt {
            let mut c1 = order_str;
            let o1 = parse_math.parse_next(&mut c1)?;
            let num = MathNode::Scripts {
                base: Box::new(d_sym()),
                sub: None,
                sup: Some(Box::new(o1.clone())),
                pre_sub: None,
                pre_sup: None,
                behavior: LimitBehavior::Default,
            };
            let den = MathNode::Row(vec![
                d_sym(),
                MathNode::Scripts {
                    base: Box::new(node_x),
                    sub: None,
                    sup: Some(Box::new(o1)),
                    pre_sub: None,
                    pre_sup: None,
                    behavior: LimitBehavior::Default,
                },
            ]);
            (num, den)
        } else {
            let num = d_sym();
            let den = MathNode::Row(vec![d_sym(), node_x]);
            (num, den)
        };
        Ok(MathNode::Fraction(Box::new(num), Box::new(den)))
    }
}

/// Parses Partial Derivative `\pdv[n]{f}{x}`, `\pdv{f}{x}`, `\pdv{f}{x}{y}`, or `\pdv{x}`
pub fn parse_pdv_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let order_opt = opt(take_balanced_brackets).parse_next(input)?;
    let (first, _) = take_balanced_braces(input)?;
    let second = opt(take_balanced_braces).parse_next(input)?;
    let third = opt(take_balanced_braces).parse_next(input)?;

    let p_sym = || MathNode::Operator(Cow::Borrowed("\u{2202}")); // ∂

    if let Some((trd, _)) = third {
        // Three arguments: \pdv{f}{x}{y} -> ∂^2 f / ∂x ∂y
        let mut c_f = first;
        let node_f = parse_math.parse_next(&mut c_f)?;
        let mut c_x = second.unwrap().0;
        let node_x = parse_math.parse_next(&mut c_x)?;
        let mut c_y = trd;
        let node_y = parse_math.parse_next(&mut c_y)?;

        let num = MathNode::Row(vec![
            MathNode::Scripts {
                base: Box::new(p_sym()),
                sub: None,
                sup: Some(Box::new(MathNode::Number(Cow::Borrowed("2")))),
                pre_sub: None,
                pre_sup: None,
                behavior: LimitBehavior::Default,
            },
            node_f,
        ]);
        let den = MathNode::Row(vec![p_sym(), node_x, p_sym(), node_y]);
        Ok(MathNode::Fraction(Box::new(num), Box::new(den)))
    } else if let Some((sec, _)) = second {
        // Two arguments: \pdv[n]{f}{x} -> ∂^n f / ∂x^n
        let mut c_f = first;
        let node_f = parse_math.parse_next(&mut c_f)?;
        let mut c_x = sec;
        let node_x = parse_math.parse_next(&mut c_x)?;

        let (num, den) = if let Some((order_str, _)) = order_opt {
            let mut c1 = order_str;
            let o1 = parse_math.parse_next(&mut c1)?;
            let num = MathNode::Row(vec![
                MathNode::Scripts {
                    base: Box::new(p_sym()),
                    sub: None,
                    sup: Some(Box::new(o1.clone())),
                    pre_sub: None,
                    pre_sup: None,
                    behavior: LimitBehavior::Default,
                },
                node_f,
            ]);
            let den = MathNode::Row(vec![
                p_sym(),
                MathNode::Scripts {
                    base: Box::new(node_x),
                    sub: None,
                    sup: Some(Box::new(o1)),
                    pre_sub: None,
                    pre_sup: None,
                    behavior: LimitBehavior::Default,
                },
            ]);
            (num, den)
        } else {
            let num = MathNode::Row(vec![p_sym(), node_f]);
            let den = MathNode::Row(vec![p_sym(), node_x]);
            (num, den)
        };
        Ok(MathNode::Fraction(Box::new(num), Box::new(den)))
    } else {
        // Single argument: \pdv[n]{x} -> ∂^n / ∂x^n
        let mut c_x = first;
        let node_x = parse_math.parse_next(&mut c_x)?;

        let (num, den) = if let Some((order_str, _)) = order_opt {
            let mut c1 = order_str;
            let o1 = parse_math.parse_next(&mut c1)?;
            let num = MathNode::Scripts {
                base: Box::new(p_sym()),
                sub: None,
                sup: Some(Box::new(o1.clone())),
                pre_sub: None,
                pre_sup: None,
                behavior: LimitBehavior::Default,
            };
            let den = MathNode::Row(vec![
                p_sym(),
                MathNode::Scripts {
                    base: Box::new(node_x),
                    sub: None,
                    sup: Some(Box::new(o1)),
                    pre_sub: None,
                    pre_sup: None,
                    behavior: LimitBehavior::Default,
                },
            ]);
            (num, den)
        } else {
            let num = p_sym();
            let den = MathNode::Row(vec![p_sym(), node_x]);
            (num, den)
        };
        Ok(MathNode::Fraction(Box::new(num), Box::new(den)))
    }
}

/// Vector operators: `\grad`, `\div`, `\curl`, `\rot`, `\laplacian`
pub fn parse_vector_op_cmd<'s>(cmd: &str) -> MathNode<'s> {
    match cmd {
        "grad" => MathNode::Operator(Cow::Borrowed("\u{2207}")), // ∇
        "div" => MathNode::Row(vec![
            MathNode::Operator(Cow::Borrowed("\u{2207}")),
            MathNode::Operator(Cow::Borrowed("\u{22C5}")), // ⋅
        ]),
        "curl" | "rot" => MathNode::Row(vec![
            MathNode::Operator(Cow::Borrowed("\u{2207}")),
            MathNode::Operator(Cow::Borrowed("\u{00D7}")), // ×
        ]),
        "laplacian" => MathNode::Scripts {
            base: Box::new(MathNode::Operator(Cow::Borrowed("\u{2207}"))),
            sub: None,
            sup: Some(Box::new(MathNode::Number(Cow::Borrowed("2")))),
            pre_sub: None,
            pre_sup: None,
            behavior: LimitBehavior::Default,
        },
        _ => unreachable!(),
    }
}

/// Parses unit macros `\unit{...}` or `\pu{...}`
pub fn parse_unit_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let (inner, _) = take_balanced_braces(input)?;
    let mut cursor = inner;
    let content = parse_math.parse_next(&mut cursor)?;
    Ok(MathNode::Style {
        variant: Cow::Borrowed("normal"),
        content: Box::new(content),
    })
}
