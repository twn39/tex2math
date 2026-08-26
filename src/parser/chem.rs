//! Chemistry parsing for `\ce{...}` and `\pu{...}` (mhchem core subset).
//!
//! Direct streaming parsing into zero-copy `MathNode` AST without intermediate
//! string re-expansion.

use std::borrow::Cow;
use winnow::ascii::multispace0 as space0;
use winnow::combinator::opt;
use winnow::prelude::*;

use crate::ast::*;
use crate::parser::atoms::{take_balanced_braces, take_balanced_brackets};
use crate::parser::parse_math;

/// Parses `\ce{...}` command.
pub fn parse_ce_cmd<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let (inner, _) = take_balanced_braces(input)?;
    let mut cursor = inner;
    let node = parse_chem_formula(&mut cursor)?;
    Ok(node)
}

/// Parses the entire contents of a chemical formula.
pub fn parse_chem_formula<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let mut nodes = Vec::new();

    while !input.is_empty() {
        let _ = space0.parse_next(input)?;
        if input.is_empty() {
            break;
        }

        // 1. Embedded math: `$ ... $`
        if input.starts_with('$') {
            *input = &input[1..];
            if let Some(end_idx) = input.find('$') {
                let mut math_str = &input[..end_idx];
                *input = &input[end_idx + 1..];
                let math_node = parse_math.parse_next(&mut math_str)?;
                nodes.push(math_node);
                continue;
            }
        }

        // 2. Embedded LaTeX command: e.g. `\text{...}`, `\alpha`, `\Delta`
        if input.starts_with('\\') {
            let math_node = parse_math.parse_next(input)?;
            nodes.push(math_node);
            continue;
        }

        // 3. Reaction Arrows
        if let Some(arrow_node) = try_parse_reaction_arrow(input)? {
            nodes.push(arrow_node);
            continue;
        }

        // 4. Precipitate / gas arrows: (v), v, (^), ^
        if input.starts_with("(v)") || (input.starts_with('v') && is_delim_or_space(input, 1)) {
            let len = if input.starts_with("(v)") { 3 } else { 1 };
            *input = &input[len..];
            nodes.push(MathNode::Operator(Cow::Borrowed("\u{2193}"))); // ↓
            continue;
        }
        if input.starts_with("(^)") {
            *input = &input[3..];
            nodes.push(MathNode::Operator(Cow::Borrowed("\u{2191}"))); // ↑
            continue;
        }

        // 5. Addition dot (e.g. CuSO4 . 5H2O)
        if (input.starts_with('.') || input.starts_with('*')) && is_delim_or_space(input, 1) {
            *input = &input[1..];
            nodes.push(MathNode::Operator(Cow::Borrowed("\u{22C5}"))); // ⋅
            continue;
        }

        // 6. Chemical bonds: # (triple), = (double)
        if input.starts_with('#') {
            *input = &input[1..];
            nodes.push(MathNode::Operator(Cow::Borrowed("\u{2261}"))); // ≡
            continue;
        }

        // 7. Plus operator between reactants: `+`
        if input.starts_with('+') && is_delim_or_space(input, 1) {
            *input = &input[1..];
            nodes.push(MathNode::Operator(Cow::Borrowed("+")));
            continue;
        }

        // 8. Stoichiometric number / fraction at term level: e.g. `2H2` or `1/2O2`
        if let Some(stoich_node) = try_parse_stoichiometric(input)? {
            nodes.push(stoich_node);
            continue;
        }

        // 9. Isotopes / pre-scripts: e.g. `^{238}_{92}U` or `^238_92U` or `^14C`
        if input.starts_with('^') || input.starts_with('_') {
            if let Some(iso_node) = try_parse_isotope(input)? {
                nodes.push(iso_node);
                continue;
            }
        }

        // 10. Parentheses: (OH)2 or [Fe(CN)6]3-
        if input.starts_with('(') || input.starts_with('[') {
            let is_bracket = input.starts_with('[');
            let open_char = if is_bracket { '[' } else { '(' };
            let close_char = if is_bracket { ']' } else { ')' };
            *input = &input[1..];

            // Find matching close
            let mut depth = 1usize;
            let mut end_idx = None;
            for (idx, c) in input.char_indices() {
                if c == open_char {
                    depth += 1;
                } else if c == close_char {
                    depth -= 1;
                    if depth == 0 {
                        end_idx = Some(idx);
                        break;
                    }
                }
            }

            if let Some(idx) = end_idx {
                let mut inner_str = &input[..idx];
                *input = &input[idx + 1..];
                let inner_node = parse_chem_formula(&mut inner_str)?;
                let fenced = MathNode::Fenced {
                    open: Cow::Borrowed(if is_bracket { "[" } else { "(" }),
                    content: Box::new(inner_node),
                    close: Cow::Borrowed(if is_bracket { "]" } else { ")" }),
                };

                // Check for subscripts/superscripts on the fenced group
                let (sub, sup) = parse_chem_scripts(input)?;
                if sub.is_some() || sup.is_some() {
                    nodes.push(MathNode::Scripts {
                        base: Box::new(fenced),
                        sub,
                        sup,
                        pre_sub: None,
                        pre_sup: None,
                        behavior: LimitBehavior::Default,
                    });
                } else {
                    nodes.push(fenced);
                }
                continue;
            }
        }

        // 11. Chemical Elements (e.g. `H`, `He`, `Ca`, `Fe`, `Na`, `Cl`, `O`)
        if let Some(first_char) = input.chars().next() {
            if first_char.is_ascii_uppercase() {
                let mut elem_len = 1;
                let rest = &input[1..];
                if let Some(second_char) = rest.chars().next() {
                    if second_char.is_ascii_lowercase() {
                        elem_len = 2;
                    }
                }
                let elem_str = &input[..elem_len];
                *input = &input[elem_len..];

                let elem_base = MathNode::Style {
                    variant: Cow::Borrowed("normal"),
                    content: Box::new(MathNode::Identifier(Cow::Borrowed(elem_str))),
                };

                let (sub, sup) = parse_chem_scripts(input)?;
                if sub.is_some() || sup.is_some() {
                    nodes.push(MathNode::Scripts {
                        base: Box::new(elem_base),
                        sub,
                        sup,
                        pre_sub: None,
                        pre_sup: None,
                        behavior: LimitBehavior::Default,
                    });
                } else {
                    nodes.push(elem_base);
                }
                continue;
            }
        }

        // 12. Fallback: numbers or identifiers
        if let Some(c) = input.chars().next() {
            if c.is_ascii_digit() {
                let mut digit_len = 0;
                for ch in input.chars() {
                    if ch.is_ascii_digit() {
                        digit_len += ch.len_utf8();
                    } else {
                        break;
                    }
                }
                let s = &input[..digit_len];
                *input = &input[digit_len..];
                nodes.push(MathNode::Number(Cow::Borrowed(s)));
            } else {
                let c_len = c.len_utf8();
                let s = &input[..c_len];
                *input = &input[c_len..];
                nodes.push(MathNode::Identifier(Cow::Borrowed(s)));
            }
        }
    }

    if nodes.len() == 1 {
        Ok(nodes.remove(0))
    } else {
        Ok(MathNode::Row(nodes))
    }
}

fn is_delim_or_space(input: &str, offset: usize) -> bool {
    if offset >= input.len() {
        return true;
    }
    let next_char = input[offset..].chars().next();
    matches!(
        next_char,
        Some(' ' | '\t' | '\r' | '\n' | '+' | '-' | '<' | '>' | '$' | '(' | '[' | '{')
    )
}

/// Parses stoichiometric prefix: e.g. `2`, `3`, `1/2`, `3/2`
fn try_parse_stoichiometric<'s>(input: &mut &'s str) -> ModalResult<Option<MathNode<'s>>> {
    let s = *input;
    let mut digit_len = 0;
    for c in s.chars() {
        if c.is_ascii_digit() {
            digit_len += 1;
        } else {
            break;
        }
    }

    if digit_len == 0 {
        return Ok(None);
    }

    let after_digits = &s[digit_len..];
    // Check for fraction like 1/2
    if let Some(after_slash) = after_digits.strip_prefix('/') {
        let mut den_len = 0;
        for c in after_slash.chars() {
            if c.is_ascii_digit() {
                den_len += 1;
            } else {
                break;
            }
        }
        if den_len > 0 {
            let num_str = &s[..digit_len];
            let den_str = &after_slash[..den_len];
            *input = &after_slash[den_len..];
            return Ok(Some(MathNode::Fraction(
                Box::new(MathNode::Number(Cow::Borrowed(num_str))),
                Box::new(MathNode::Number(Cow::Borrowed(den_str))),
            )));
        }
    }

    // A digit is stoichiometric if followed by an uppercase letter or `(` or `[`
    if after_digits.starts_with(|c: char| c.is_ascii_uppercase() || c == '(' || c == '[') {
        let num_str = &s[..digit_len];
        *input = after_digits;
        return Ok(Some(MathNode::Number(Cow::Borrowed(num_str))));
    }

    Ok(None)
}

/// Parses chemical reaction arrows: `->`, `<=>`, `<-`, `<->` with optional `[top][bottom]`
fn try_parse_reaction_arrow<'s>(input: &mut &'s str) -> ModalResult<Option<MathNode<'s>>> {
    let s = *input;
    let (arrow_str, op_glyph) = if s.starts_with("<=>") {
        ("<=>", "\u{21CC}") // ⇌ (rightleftharpoons)
    } else if s.starts_with("<->") {
        ("<->", "\u{2194}") // ↔
    } else if s.starts_with("->") {
        ("->", "\u{2192}") // →
    } else if s.starts_with("<-") {
        ("<-", "\u{2190}") // ←
    } else {
        return Ok(None);
    };

    *input = &s[arrow_str.len()..];

    // Optional annotations: [top][bottom]
    let top_opt = opt(take_balanced_brackets).parse_next(input)?;
    let bottom_opt = opt(take_balanced_brackets).parse_next(input)?;

    if top_opt.is_some() || bottom_opt.is_some() {
        let top_node = if let Some((top_str, _)) = top_opt {
            let mut c = top_str;
            Some(parse_chem_formula(&mut c)?)
        } else {
            None
        };

        let bottom_node = if let Some((bot_str, _)) = bottom_opt {
            let mut c = bot_str;
            Some(parse_chem_formula(&mut c)?)
        } else {
            None
        };

        let base_arrow = MathNode::Operator(Cow::Borrowed(op_glyph));
        let node = match (top_node, bottom_node) {
            (Some(top), Some(bot)) => MathNode::Scripts {
                base: Box::new(base_arrow),
                sub: Some(Box::new(bot)),
                sup: Some(Box::new(top)),
                pre_sub: None,
                pre_sup: None,
                behavior: LimitBehavior::Limits,
            },
            (Some(top), None) => MathNode::Scripts {
                base: Box::new(base_arrow),
                sub: None,
                sup: Some(Box::new(top)),
                pre_sub: None,
                pre_sup: None,
                behavior: LimitBehavior::Limits,
            },
            (None, Some(bot)) => MathNode::Scripts {
                base: Box::new(base_arrow),
                sub: Some(Box::new(bot)),
                sup: None,
                pre_sub: None,
                pre_sup: None,
                behavior: LimitBehavior::Limits,
            },
            (None, None) => base_arrow,
        };
        Ok(Some(node))
    } else {
        Ok(Some(MathNode::Operator(Cow::Borrowed(op_glyph))))
    }
}

/// Parses isotopes: `^{238}_{92}U` or `^238_92U` or `^14C`
fn try_parse_isotope<'s>(input: &mut &'s str) -> ModalResult<Option<MathNode<'s>>> {
    let orig = *input;
    let mut pre_sup = None;
    let mut pre_sub = None;

    while input.starts_with('^') || input.starts_with('_') {
        let is_sup = input.starts_with('^');
        *input = &input[1..];

        let content_node = if input.starts_with('{') {
            let (b, _) = take_balanced_braces(input)?;
            let mut c = b;
            parse_chem_formula(&mut c)?
        } else {
            let mut digit_len = 0;
            for c in input.chars() {
                if c.is_ascii_digit() {
                    digit_len += 1;
                } else {
                    break;
                }
            }
            if digit_len > 0 {
                let d_str = &input[..digit_len];
                *input = &input[digit_len..];
                MathNode::Number(Cow::Borrowed(d_str))
            } else if let Some(c) = input.chars().next() {
                let c_len = c.len_utf8();
                let s = &input[..c_len];
                *input = &input[c_len..];
                MathNode::Identifier(Cow::Borrowed(s))
            } else {
                *input = orig;
                return Ok(None);
            }
        };

        if is_sup {
            pre_sup = Some(Box::new(content_node));
        } else {
            pre_sub = Some(Box::new(content_node));
        }
    }

    // Next must be element symbol
    let _ = space0.parse_next(input)?;
    if let Some(first_char) = input.chars().next() {
        if first_char.is_ascii_uppercase() {
            let mut elem_len = 1;
            let rest = &input[1..];
            if let Some(second_char) = rest.chars().next() {
                if second_char.is_ascii_lowercase() {
                    elem_len = 2;
                }
            }
            let elem_str = &input[..elem_len];
            *input = &input[elem_len..];

            let elem_base = MathNode::Style {
                variant: Cow::Borrowed("normal"),
                content: Box::new(MathNode::Identifier(Cow::Borrowed(elem_str))),
            };

            let (post_sub, post_sup) = parse_chem_scripts(input)?;

            return Ok(Some(MathNode::Scripts {
                base: Box::new(elem_base),
                sub: post_sub,
                sup: post_sup,
                pre_sub,
                pre_sup,
                behavior: LimitBehavior::Default,
            }));
        }
    }

    *input = orig;
    Ok(None)
}

/// Parses subscripts and charges/superscripts attached to an element or group.
#[allow(clippy::type_complexity)]
fn parse_chem_scripts<'s>(
    input: &mut &'s str,
) -> ModalResult<(Option<Box<MathNode<'s>>>, Option<Box<MathNode<'s>>>)> {
    let mut sub = None;
    let mut sup = None;

    // 1. Explicit subscript `_2` or `_{...}` or trailing digits like `2` in `H2`
    if input.starts_with('_') {
        *input = &input[1..];
        if input.starts_with('{') {
            let (inner, _) = take_balanced_braces(input)?;
            let mut c = inner;
            sub = Some(Box::new(parse_chem_formula(&mut c)?));
        } else if let Some(c) = input.chars().next() {
            let c_len = c.len_utf8();
            let s = &input[..c_len];
            *input = &input[c_len..];
            sub = Some(Box::new(if c.is_ascii_digit() {
                MathNode::Number(Cow::Borrowed(s))
            } else {
                MathNode::Identifier(Cow::Borrowed(s))
            }));
        }
    } else {
        // Auto subscript for immediate digits: e.g. `2` in `H2O`
        let mut digit_len = 0;
        for c in input.chars() {
            if c.is_ascii_digit() {
                digit_len += 1;
            } else {
                break;
            }
        }
        if digit_len > 0 {
            let d_str = &input[..digit_len];
            *input = &input[digit_len..];
            sub = Some(Box::new(MathNode::Number(Cow::Borrowed(d_str))));
        }
    }

    // 2. Superscript or charge: `^2+`, `^{2+}`, `^+`, `^-`, `^3-` or immediate `+`/`-`
    if input.starts_with('^') {
        *input = &input[1..];
        if input.starts_with('{') {
            let (inner, _) = take_balanced_braces(input)?;
            let mut c = inner;
            sup = Some(Box::new(parse_chem_charge_or_sup(&mut c)?));
        } else {
            let mut charge_str = "";
            for len in (1..=3).rev() {
                if input.len() >= len {
                    let candidate = &input[..len];
                    if is_valid_charge(candidate) {
                        charge_str = candidate;
                        break;
                    }
                }
            }
            if !charge_str.is_empty() {
                *input = &input[charge_str.len()..];
                sup = Some(Box::new(format_charge_node(charge_str)));
            } else if let Some(c) = input.chars().next() {
                let c_len = c.len_utf8();
                let s = &input[..c_len];
                *input = &input[c_len..];
                sup = Some(Box::new(MathNode::Identifier(Cow::Borrowed(s))));
            }
        }
    } else {
        // Look for immediate charges like `+`, `-`, `2+`, `3-`
        // Only if followed by space, end, or delimiter
        for len in (1..=3).rev() {
            if input.len() >= len {
                let candidate = &input[..len];
                if is_valid_charge(candidate) && is_delim_or_space(input, len) {
                    *input = &input[len..];
                    sup = Some(Box::new(format_charge_node(candidate)));
                    break;
                }
            }
        }
    }

    Ok((sub, sup))
}

fn is_valid_charge(s: &str) -> bool {
    matches!(
        s,
        "+" | "-" | "2+" | "3+" | "4+" | "2-" | "3-" | "4-" | "+2" | "+3" | "-2" | "-3"
    )
}

fn format_charge_node<'s>(s: &'s str) -> MathNode<'s> {
    if s == "+" {
        MathNode::Operator(Cow::Borrowed("+"))
    } else if s == "-" {
        MathNode::Operator(Cow::Borrowed("\u{2212}")) // −
    } else {
        let mut nodes = Vec::new();
        for c in s.chars() {
            if c.is_ascii_digit() {
                let c_str = match c {
                    '1' => "1",
                    '2' => "2",
                    '3' => "3",
                    '4' => "4",
                    '5' => "5",
                    '6' => "6",
                    '7' => "7",
                    '8' => "8",
                    '9' => "9",
                    _ => "0",
                };
                nodes.push(MathNode::Number(Cow::Borrowed(c_str)));
            } else if c == '+' {
                nodes.push(MathNode::Operator(Cow::Borrowed("+")));
            } else if c == '-' {
                nodes.push(MathNode::Operator(Cow::Borrowed("\u{2212}")));
            }
        }
        if nodes.len() == 1 {
            nodes.remove(0)
        } else {
            MathNode::Row(nodes)
        }
    }
}

fn parse_chem_charge_or_sup<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    let s = *input;
    if is_valid_charge(s) {
        *input = "";
        Ok(format_charge_node(s))
    } else {
        parse_chem_formula(input)
    }
}
