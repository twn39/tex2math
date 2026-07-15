use std::borrow::Cow;
use winnow::ascii::{alpha1, multispace0 as space0};
use winnow::combinator::{delimited, opt, preceded, separated, trace};
use winnow::prelude::*;
use winnow::token::literal;

use super::atoms::{take_balanced_braces, take_balanced_brackets};
use super::parse_row;
use crate::ast::*;

/// 解析一行中由 `&` 分隔的多个单元格 (Cell)
pub fn parse_cells_in_row<'s>(input: &mut &'s str) -> ModalResult<Vec<MathNode<'s>>> {
    separated(
        0..,
        delimited(space0, parse_row, space0),
        (space0, '&', space0),
    )
    .parse_next(input)
}

/// 解析换行符 `\\` 及其可选的垂直间距参数，例如 `\\[2mm]`
pub fn parse_newline_opt<'s>(input: &mut &'s str) -> ModalResult<Option<&'s str>> {
    preceded(
        literal("\\\\"),
        opt(take_balanced_brackets.map(|(content, _)| content)),
    )
    .parse_next(input)
}

pub fn parse_environment<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    trace("parse_environment", |input: &mut &'s str| {
        let begin_tag = preceded((literal("\\begin"), space0, '{'), alpha1).parse_next(input)?;
        let name: Cow<'s, str> = Cow::Borrowed(begin_tag);
        let _ = literal("}").parse_next(input)?;

        let mut format: Option<Cow<'s, str>> = None;
        if name.as_ref() == "array" {
            let fmt_opt: Option<(&str, bool)> = opt(take_balanced_braces).parse_next(input)?;
            if let Some((fmt_str, _)) = fmt_opt {
                format = Some(Cow::Borrowed(fmt_str));
            }
        }

        let begin_pattern = format!("\\begin{{{}}}", name);
        let end_pattern = format!("\\end{{{}}}", name);

        // Fix 1 (root cause): nesting-aware body extraction.
        let (mut inner_str, is_closed) = {
            let full = *input;
            let mut depth = 1usize;
            let mut scanned = 0usize;
            let mut remaining = full;
            let mut matched_end_pos: Option<usize> = None;

            'scan: loop {
                let next_begin = remaining.find(begin_pattern.as_str());
                let next_end = remaining.find(end_pattern.as_str());

                match (next_begin, next_end) {
                    (_, None) => {
                        // No \end{name} found anywhere: unclosed environment
                        break 'scan;
                    }
                    (Some(b), Some(e)) if b < e => {
                        // A nested \begin{name} appears before the next \end{name}: push depth
                        depth += 1;
                        let skip = b + begin_pattern.len();
                        scanned += skip;
                        remaining = &remaining[skip..];
                    }
                    (_, Some(e)) => {
                        // \end{name} comes next (or no \begin{name} left): pop depth
                        depth -= 1;
                        if depth == 0 {
                            // Found the truly matching \end{name}
                            matched_end_pos = Some(scanned + e);
                            break 'scan;
                        }
                        let skip = e + end_pattern.len();
                        scanned += skip;
                        remaining = &remaining[skip..];
                    }
                }
            }

            if let Some(end_pos) = matched_end_pos {
                let inner = &full[..end_pos];
                *input = &full[end_pos + end_pattern.len()..];
                (inner, true)
            } else {
                // Unclosed: consume all remaining input
                let inner = full;
                *input = &full[full.len()..];
                (inner, false)
            }
        };

        let mut parse_cells_in_row = |row_input: &mut &'s str| -> ModalResult<Vec<MathNode<'s>>> {
            separated(
                0..,
                delimited(space0, parse_row, space0),
                (space0, '&', space0),
            )
            .parse_next(row_input)
        };

        let mut parse_newline_opt = |input: &mut &'s str| -> ModalResult<Option<&str>> {
            preceded(
                literal("\\\\"),
                opt(take_balanced_brackets.map(|(content, _)| content)),
            )
            .parse_next(input)
        };

        let mut rows: Vec<(Vec<MathNode<'s>>, Option<Cow<'s, str>>)> = Vec::new();

        loop {
            let _ = space0.parse_next(&mut inner_str)?;
            if inner_str.is_empty() {
                break;
            }
            // Fix 2: zero-progress guard — record position after consuming leading whitespace.
            let progress_mark = inner_str.len();

            if let Ok(cells) = parse_cells_in_row.parse_next(&mut inner_str) {
                let spacing = if let Ok(opt_spacing) = parse_newline_opt.parse_next(&mut inner_str)
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

                // Ignore trailing empty rows caused by a final `\\` before `\end`
                if is_empty_row
                    && spacing.is_none()
                    && inner_str.trim().is_empty()
                    && !rows.is_empty()
                {
                    // Do not push this empty row
                } else {
                    rows.push((cells, spacing));
                }
            } else {
                // Should not happen as separated(0..) always matches something, but fallback
                break;
            }

            if inner_str.is_empty() {
                break;
            }
            // Zero-progress guard: if neither parse_cells_in_row nor parse_newline_opt
            // consumed any input, there is an irrecoverable stuck token — exit cleanly.
            if inner_str.len() == progress_mark {
                break;
            }
        }

        let env_node = MathNode::Environment {
            name: name.clone(),
            format,
            rows,
        };

        let unknown = !crate::registry::is_known_environment(name.as_ref());
        let strict = matches!(
            crate::depth::recovery_mode(),
            crate::ast::RecoveryMode::Strict
        );

        // Strict + unknown: surface only `<merror>` (no salvage table).
        if unknown && strict {
            let mut parts = vec![MathNode::Error(Cow::Owned(format!(
                "Unknown environment '{}'",
                name.as_ref()
            )))];
            if !is_closed {
                parts.push(MathNode::Error(Cow::Owned(format!(
                    "Missing \\end{{{}}}",
                    name.as_ref()
                ))));
            }
            return Ok(if parts.len() == 1 {
                parts.into_iter().next().unwrap()
            } else {
                MathNode::Row(parts)
            });
        }

        let mut parts = vec![env_node];
        if unknown {
            parts.push(MathNode::Error(Cow::Owned(format!(
                "Unknown environment '{}'",
                name.as_ref()
            ))));
        }
        if !is_closed {
            parts.push(MathNode::Error(Cow::Owned(format!(
                "Missing \\end{{{}}}",
                name.as_ref()
            ))));
        }

        if parts.len() == 1 {
            Ok(parts.into_iter().next().unwrap())
        } else {
            Ok(MathNode::Row(parts))
        }
    })
    .parse_next(input)
}
