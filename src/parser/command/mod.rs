//! LaTeX control-sequence parsing.
//!
//! Dispatch is driven by [`crate::registry::command_spec`]:
//! - **Table-driven** families → fixed argument shapes + registry payloads
//! - **Irregular** → hand combinators in this module tree
//! - **Structural** → fail here (owned by outer parser)
//!
//! Submodules mirror MathML `iter/` families for navigation:
//! - [`style`] — fonts, color, accent, cancel, phantom, style switch
//! - [`scripts`] — overset/underset, sideset, stretch, arrows, not, operatorname
//! - [`frac`] — dfrac/tfrac/cfrac, binom, genfrac
//! - [`space`] — hskip/kern/mkern, sized delims
//! - [`misc`] — mod, math-class, substack, middle, tag

mod frac;
mod misc;
mod scripts;
mod space;
mod style;

use std::borrow::Cow;
use winnow::ascii::alpha1;
use winnow::combinator::{alt, preceded, trace};
use winnow::prelude::*;
use winnow::token::one_of;

use crate::ast::*;
use crate::registry::{command_spec, CommandSpec};

use frac::{parse_binom_cmd, parse_frac_style_cmd, parse_genfrac_cmd};
use misc::{
    parse_math_class_cmd, parse_middle_cmd, parse_mod_cmd, parse_substack_cmd, parse_tag_cmd,
};
use scripts::{
    parse_extensible_arrow_cmd, parse_not_modifier_cmd, parse_operatorname_cmd,
    parse_over_under_set_cmd, parse_sideset_cmd, parse_stretch_modifier_cmd,
};
use space::{parse_dim_space_cmd, parse_sized_delimiter_cmd};
use style::{
    parse_accent_cmd, parse_boxed_cmd, parse_cancel_cmd, parse_color_cmd, parse_font_style_cmd,
    parse_phantom_cmd, parse_style_switch_cmd, parse_text_cmd,
};

/// Resolve zero-argument table-driven commands. Returns `None` if `cmd` is not in any table.
fn resolve_zero_arg_cmd<'s>(cmd: &'s str) -> Option<MathNode<'s>> {
    use crate::registry::{
        lookup, BLACKBOARD_LETTERS, IDENT_ALIASES, MATH_FUNCTIONS, SPACING_CMDS, VAR_GREEK,
        VAR_LIM_CMDS,
    };

    if crate::registry::contains_key(MATH_FUNCTIONS, cmd) {
        return Some(MathNode::Function(Cow::Borrowed(cmd)));
    }
    if let Some(width) = lookup(SPACING_CMDS, cmd) {
        return Some(MathNode::Space(Cow::Borrowed(width)));
    }
    if crate::registry::contains_key(BLACKBOARD_LETTERS, cmd) {
        return Some(MathNode::Style {
            variant: Cow::Borrowed("double-struck"),
            content: Box::new(MathNode::Identifier(Cow::Borrowed(cmd))),
        });
    }
    if let Some(ch) = lookup(IDENT_ALIASES, cmd) {
        return Some(MathNode::Identifier(Cow::Borrowed(ch)));
    }
    if let Some(letter) = lookup(VAR_GREEK, cmd) {
        return Some(MathNode::Style {
            variant: Cow::Borrowed("italic"),
            content: Box::new(MathNode::Identifier(Cow::Borrowed(letter))),
        });
    }
    if let Some(arrow) = lookup(VAR_LIM_CMDS, cmd) {
        return Some(MathNode::Scripts {
            base: Box::new(MathNode::Function(Cow::Borrowed("lim"))),
            sub: Some(Box::new(MathNode::Operator(Cow::Borrowed(arrow)))),
            sup: None,
            pre_sub: None,
            pre_sup: None,
            behavior: LimitBehavior::Limits,
        });
    }
    None
}

fn unknown_command_node<'s>(cmd: &str) -> MathNode<'s> {
    use crate::ast::UnknownCommandPolicy;
    match crate::depth::unknown_command_policy() {
        UnknownCommandPolicy::Identifier => MathNode::Identifier(Cow::Owned(format!("\\{cmd}"))),
        UnknownCommandPolicy::Error => {
            MathNode::Error(Cow::Owned(format!("Unknown command \\{cmd}")))
        }
    }
}

fn parse_irregular_cmd<'s>(cmd: &str, input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    match cmd {
        "text" => parse_text_cmd(input),
        "color" | "textcolor" | "colorbox" => parse_color_cmd(cmd, input),
        "boxed" => parse_boxed_cmd(input),
        "overset" | "underset" | "stackrel" => {
            parse_over_under_set_cmd(if cmd == "stackrel" { "overset" } else { cmd }, input)
        }
        "sideset" => parse_sideset_cmd(input),
        "operatorname" | "operatorname*" => parse_operatorname_cmd(cmd, input),
        "not" => parse_not_modifier_cmd(input),
        "choose" => Ok(MathNode::ChooseMarker),
        "genfrac" => parse_genfrac_cmd(input),
        "substack" => parse_substack_cmd(input),
        "middle" => parse_middle_cmd(input),
        "tag" => parse_tag_cmd(input),
        "notag" => Ok(MathNode::Row(vec![])),
        _ => unreachable!("irregular pre-filtered via CommandSpec"),
    }
}

pub fn parse_command<'s>(input: &mut &'s str) -> ModalResult<MathNode<'s>> {
    trace("parse_command", |input: &mut &'s str| {
        let cmd = preceded(
            '\\',
            alt((
                alpha1,
                one_of([
                    ',', ';', ':', '!', '%', '$', '#', '&', '_', ' ', '{', '}', '|',
                ])
                .take(),
            )),
        )
        .parse_next(input)?;

        // Fast path: pure zero-arg macros from static tables.
        if let Some(node) = resolve_zero_arg_cmd(cmd) {
            return Ok(node);
        }

        // Single-source dispatch (registry tables + irregular list).
        match command_spec(cmd) {
            Some(CommandSpec::FontStyle) => parse_font_style_cmd(cmd, input),
            Some(CommandSpec::FracStyle) => parse_frac_style_cmd(cmd, input),
            Some(CommandSpec::Binom) => parse_binom_cmd(cmd, input),
            Some(CommandSpec::ExtensibleArrow) => parse_extensible_arrow_cmd(cmd, input),
            Some(CommandSpec::StretchOp) => parse_stretch_modifier_cmd(cmd, input),
            Some(CommandSpec::Accent) => parse_accent_cmd(cmd, input),
            Some(CommandSpec::Cancel) => parse_cancel_cmd(cmd, input),
            Some(CommandSpec::Phantom) => parse_phantom_cmd(cmd, input),
            Some(CommandSpec::SizedDelim) => parse_sized_delimiter_cmd(cmd, input),
            Some(CommandSpec::Mod) => parse_mod_cmd(cmd, input),
            Some(CommandSpec::MathClass) => parse_math_class_cmd(cmd, input),
            Some(CommandSpec::StyleSwitch) => {
                let ds = crate::registry::lookup_bool(crate::registry::STYLE_SWITCH_CMDS, cmd)
                    .expect("style switch pre-filtered");
                parse_style_switch_cmd(ds, input)
            }
            Some(CommandSpec::DimSpace) => parse_dim_space_cmd(cmd, input),
            Some(CommandSpec::Structural) => {
                // Handled by outer parser (`frac` / `sqrt` / `left` / `right`).
                winnow::combinator::fail.parse_next(input)
            }
            Some(CommandSpec::Irregular) => parse_irregular_cmd(cmd, input),
            None => {
                if let Some(node) = crate::symbols::lookup_symbol(cmd) {
                    Ok(node)
                } else {
                    Ok(unknown_command_node(cmd))
                }
            }
        }
    })
    .parse_next(input)
}
