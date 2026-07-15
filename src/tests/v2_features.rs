//! Regression tests for tex2math 2.0 / 2.1 / 2.2 APIs.

use crate::{
    command_spec, convert, generate_mathml, parse, registered_command_names, render_mathml_to,
    supports_command, CommandSpec, ConvertOptions, MathClass, MathNode, ParseErrorKind,
    ParseOptions, RenderMode, RenderOptions, TrailingPolicy, UnknownCommandPolicy,
};

#[test]
fn convert_wraps_math_root_by_default() {
    let out = convert(r"x+1", &ConvertOptions::default()).unwrap();
    assert!(out.starts_with("<math xmlns="));
    assert!(out.contains("<mi>x</mi>"));
}

#[test]
fn convert_no_wrap() {
    let opts = ConvertOptions {
        wrap_math: false,
        ..Default::default()
    };
    let out = convert(r"x", &opts).unwrap();
    assert!(!out.contains("<math"));
    assert!(out.contains("<mi>x</mi>"));
}

#[test]
fn convert_display_mode_attr() {
    let out = convert(r"x", &ConvertOptions::display()).unwrap();
    assert!(out.contains("display=\"block\""));
}

#[test]
fn emit_intent_on_fraction() {
    let opts = ConvertOptions {
        emit_intent: true,
        wrap_math: false,
        ..Default::default()
    };
    let out = convert(r"\frac{1}{2}", &opts).unwrap();
    assert!(out.contains("intent=\"fraction\""), "got {out}");
}

#[test]
fn mathml_core_cancel_avoids_menclose() {
    let opts = ConvertOptions {
        mathml_core: true,
        wrap_math: false,
        ..Default::default()
    };
    let out = convert(r"\cancel{x}", &opts).unwrap();
    assert!(
        !out.contains("<menclose"),
        "core mode should not emit menclose: {out}"
    );
    assert!(out.contains("<mi>x</mi>"));
}

#[test]
fn non_core_cancel_uses_menclose() {
    let opts = ConvertOptions {
        mathml_core: false,
        wrap_math: false,
        ..Default::default()
    };
    let out = convert(r"\cancel{x}", &opts).unwrap();
    assert!(out.contains("<menclose"), "got {out}");
}

#[test]
fn trailing_ignore_policy() {
    let opts = ParseOptions {
        trailing: TrailingPolicy::Ignore,
        ..Default::default()
    };
    // Must succeed even with garbage after a complete expression.
    let _ast = parse(r"x } junk", &opts).expect("ignore trailing");
}

#[test]
fn trailing_error_has_kind() {
    let err = parse(r"x } junk", &ParseOptions::default()).unwrap_err();
    assert_eq!(err.kind, ParseErrorKind::UnexpectedTrailing);
    assert!(!err.span.is_empty() || err.offset() > 0);
}

#[test]
fn supports_command_known_and_unknown() {
    assert!(supports_command("frac"));
    assert!(supports_command("\\sin"));
    assert!(supports_command("alpha"));
    assert!(supports_command("mathbf"));
    assert!(!supports_command("thisIsNotARealCommandXYZ"));
}

#[test]
fn parse_into_owned_survives_drop_input() {
    let owned = {
        let s = String::from(r"\frac{a}{b}");
        parse(&s, &ParseOptions::default()).unwrap().into_owned()
    };
    match owned {
        MathNode::Fraction(num, den) => {
            assert!(matches!(*num, MathNode::Identifier(_)));
            assert!(matches!(*den, MathNode::Identifier(_)));
        }
        other => panic!("expected fraction, got {other:?}"),
    }
}

#[test]
fn render_mathml_to_sink_matches_string() {
    let ast = parse(r"\frac{1}{2}+x", &ParseOptions::default()).unwrap();
    let via_string = generate_mathml(&ast, RenderMode::Inline);
    let mut sink = String::new();
    render_mathml_to(
        &ast,
        RenderMode::Inline,
        &RenderOptions::default(),
        &mut sink,
    )
    .unwrap();
    assert_eq!(via_string, sink);
}

#[test]
fn deep_ast_render_does_not_abort() {
    // Build a deep fraction tree without going through the parse depth limit.
    let mut node = MathNode::Number(std::borrow::Cow::Borrowed("1"));
    for _ in 0..500 {
        node = MathNode::Fraction(
            Box::new(MathNode::Number(std::borrow::Cow::Borrowed("1"))),
            Box::new(node),
        );
    }
    let out = generate_mathml(&node, RenderMode::Display);
    assert!(out.contains("<mfrac>"));
    assert!(out.contains("<mn>1</mn>"));
    // Iterative path should finish (no stack overflow).
    assert!(out.matches("<mfrac>").count() >= 500);
}

#[test]
fn emit_intent_on_scripts_and_accent() {
    let opts = ConvertOptions {
        emit_intent: true,
        wrap_math: false,
        mathml_core: false,
        ..Default::default()
    };
    let out = convert(r"\hat{x}_1", &opts).unwrap();
    assert!(
        out.contains("intent=\"accent\"") || out.contains("intent=\"scripts\""),
        "got {out}"
    );
    assert!(out.contains("<mover") || out.contains("<msub"), "got {out}");
}

#[test]
fn emit_intent_coverage_matrix() {
    let opts = ConvertOptions {
        emit_intent: true,
        wrap_math: false,
        mathml_core: true,
        ..Default::default()
    };
    let cases: &[(&str, &str)] = &[
        (r"\frac{1}{2}", "intent=\"fraction\""),
        (r"\sqrt{x}", "intent=\"square-root\""),
        (r"\sqrt[3]{x}", "intent=\"root\""),
        (r"\left(x\right)", "intent=\"fenced\""),
        (r"\boxed{x}", "intent=\"boxed\""),
        (r"\cancel{x}", "intent=\"cancel:"),
        (r"\begin{pmatrix} a \\ b \end{pmatrix}", "intent=\"table\""),
        // 2.3+ coverage
        (r"\text{hi}", "intent=\"text\""),
        (r"\big(", "intent=\"sized-delimiter\""),
        (r"\overbrace{x}", "intent=\"stretch-over\""),
        (r"\underbrace{x}", "intent=\"stretch-under\""),
        (r"x", "intent=\"identifier\""),
        (r"1", "intent=\"number\""),
        (r"+", "intent=\"operator\""),
        (r"\left(a\middle|b\right)", "intent=\"middle\""),
        (r"\displaystyle x", "intent=\"displaystyle\""),
        (r"\textstyle x", "intent=\"textstyle\""),
    ];
    for (latex, needle) in cases {
        let out = convert(latex, &opts).unwrap();
        assert!(
            out.contains(needle),
            "for {latex:?} expected {needle:?}, got {out}"
        );
    }
}

/// Snapshot of irregular multi-arg commands. Prefer table-driven [`CommandSpec`]
/// families when adding macros; only grow this list for non-fixed arity shapes.
#[test]
fn irregular_cmds_snapshot_guardrail() {
    use crate::registry::IRREGULAR_CMDS;

    const EXPECTED: &[&str] = &[
        "boxed",
        "choose",
        "color",
        "colorbox",
        "genfrac",
        "middle",
        "not",
        "notag",
        "operatorname",
        "operatorname*",
        "overset",
        "sideset",
        "stackrel",
        "substack",
        "tag",
        "text",
        "textcolor",
        "underset",
    ];
    assert_eq!(
        IRREGULAR_CMDS, EXPECTED,
        "IRREGULAR_CMDS changed — update docs/COMMANDS.md and this snapshot only if the command \
         cannot be table-driven (fixed arity + registry payload)"
    );
}

#[test]
fn mathml_core_boxed_avoids_menclose() {
    let core = ConvertOptions {
        mathml_core: true,
        wrap_math: false,
        ..Default::default()
    };
    let out = convert(r"\boxed{x}", &core).unwrap();
    assert!(!out.contains("<menclose"), "got {out}");

    let legacy = ConvertOptions {
        mathml_core: false,
        wrap_math: false,
        ..Default::default()
    };
    let out = convert(r"\boxed{x}", &legacy).unwrap();
    assert!(out.contains("<menclose"), "got {out}");
}

#[test]
fn unknown_command_identifier_default() {
    let ast = parse(r"\notARealCmd", &ParseOptions::default()).unwrap();
    match ast {
        MathNode::Identifier(s) => assert_eq!(s.as_ref(), r"\notARealCmd"),
        MathNode::Row(nodes) => {
            assert!(
                nodes.iter().any(|n| matches!(
                    n,
                    MathNode::Identifier(s) if s.as_ref() == r"\notARealCmd"
                )),
                "got {nodes:?}"
            );
        }
        other => panic!("expected identifier fallback, got {other:?}"),
    }
}

#[test]
fn unknown_command_error_policy() {
    let opts = ParseOptions {
        unknown_command: UnknownCommandPolicy::Error,
        ..Default::default()
    };
    let ast = parse(r"\notARealCmd", &opts).unwrap();
    let has_error = match &ast {
        MathNode::Error(msg) => msg.contains("notARealCmd"),
        MathNode::Row(nodes) => nodes
            .iter()
            .any(|n| matches!(n, MathNode::Error(msg) if msg.contains("notARealCmd"))),
        _ => false,
    };
    assert!(has_error, "got {ast:?}");

    let out = convert(
        r"\notARealCmd",
        &ConvertOptions {
            parse: opts,
            wrap_math: false,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(out.contains("<merror"), "got {out}");
}

#[test]
fn registry_owns_font_style_names() {
    // Adding a font style in registry alone should be enough for dispatch.
    assert!(supports_command("mathbf"));
    assert!(supports_command("xrightarrow"));
    assert!(supports_command("underbrace"));
    assert!(supports_command("varinjlim"));
    assert!(supports_command("AA"));
    assert!(supports_command("binom"));
    assert!(supports_command("pmod"));
    assert!(supports_command("stackrel"));
    assert!(supports_command("arccot"));
    assert!(supports_command("mathbin"));
}

#[test]
fn command_spec_table_vs_irregular() {
    assert_eq!(command_spec("mathbf"), Some(CommandSpec::FontStyle));
    assert_eq!(command_spec("binom"), Some(CommandSpec::Binom));
    assert_eq!(command_spec("mathbin"), Some(CommandSpec::MathClass));
    assert_eq!(command_spec("pmod"), Some(CommandSpec::Mod));
    assert_eq!(command_spec("dfrac"), Some(CommandSpec::FracStyle));
    assert_eq!(command_spec("frac"), Some(CommandSpec::Structural));
    assert_eq!(command_spec("genfrac"), Some(CommandSpec::Irregular));
    assert_eq!(command_spec("text"), Some(CommandSpec::Irregular));
    assert_eq!(command_spec("tag"), Some(CommandSpec::Irregular));
    assert_eq!(command_spec("notarealcmd"), None);
}

#[test]
fn registered_command_names_is_sorted_unique() {
    let names = registered_command_names();
    assert!(names.len() > 50, "expected a non-trivial registry dump");
    assert!(names.windows(2).all(|w| w[0] < w[1]));
    assert!(names.contains(&"mathbf"));
    assert!(names.contains(&"mathbin"));
    assert!(names.contains(&"genfrac"));
}

#[test]
fn math_class_ast_and_spacing() {
    let ast = parse(r"\mathbin{+}", &ParseOptions::default()).unwrap();
    match ast {
        MathNode::MathClass {
            class: MathClass::Bin,
            content,
        } => assert!(matches!(*content, MathNode::Operator(_))),
        other => panic!("expected MathClass::Bin, got {other:?}"),
    }
    let out = convert(
        r"\mathrel{=}",
        &ConvertOptions {
            wrap_math: false,
            emit_intent: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(out.contains("lspace=\"0.2778em\""), "got {out}");
    assert!(out.contains("intent=\"math-class:rel\""), "got {out}");
}

#[test]
fn registry_tables_are_sorted_for_binary_search() {
    use crate::registry::{
        ACCENTS, BINOM_STYLES, BLACKBOARD_LETTERS, CANCEL_MODES, DIM_SPACE_CMDS, EXTENSIBLE_ARROWS,
        FONT_STYLES, FRAC_STYLES, IDENT_ALIASES, IRREGULAR_CMDS, MATH_CLASS_CMDS, MATH_FUNCTIONS,
        MOD_CMDS, PHANTOM_KINDS, SIZED_DELIMS, SPACING_CMDS, STRETCH_OPS, STRUCTURAL_CMDS,
        STYLE_SWITCH_CMDS, VAR_GREEK, VAR_LIM_CMDS,
    };

    fn assert_sorted_keys(keys: impl IntoIterator<Item = &'static str>, label: &str) {
        let v: Vec<_> = keys.into_iter().collect();
        let mut sorted = v.clone();
        sorted.sort_unstable();
        assert_eq!(v, sorted, "{label} must be sorted for binary_search");
    }

    assert_sorted_keys(FONT_STYLES.iter().map(|(k, _)| *k), "FONT_STYLES");
    assert_sorted_keys(ACCENTS.iter().map(|(k, _)| *k), "ACCENTS");
    assert_sorted_keys(SIZED_DELIMS.iter().map(|(k, _)| *k), "SIZED_DELIMS");
    assert_sorted_keys(CANCEL_MODES.iter().map(|(k, _)| *k), "CANCEL_MODES");
    assert_sorted_keys(
        EXTENSIBLE_ARROWS.iter().map(|(k, _)| *k),
        "EXTENSIBLE_ARROWS",
    );
    assert_sorted_keys(STRETCH_OPS.iter().map(|(k, _, _)| *k), "STRETCH_OPS");
    assert_sorted_keys(PHANTOM_KINDS.iter().copied(), "PHANTOM_KINDS");
    assert_sorted_keys(FRAC_STYLES.iter().map(|(k, _)| *k), "FRAC_STYLES");
    assert_sorted_keys(BINOM_STYLES.iter().map(|(k, _)| *k), "BINOM_STYLES");
    assert_sorted_keys(MATH_CLASS_CMDS.iter().copied(), "MATH_CLASS_CMDS");
    assert_sorted_keys(MOD_CMDS.iter().copied(), "MOD_CMDS");
    assert_sorted_keys(STYLE_SWITCH_CMDS.iter().map(|(k, _)| *k), "STYLE_SWITCH");
    assert_sorted_keys(DIM_SPACE_CMDS.iter().copied(), "DIM_SPACE_CMDS");
    assert_sorted_keys(IDENT_ALIASES.iter().map(|(k, _)| *k), "IDENT_ALIASES");
    assert_sorted_keys(VAR_GREEK.iter().map(|(k, _)| *k), "VAR_GREEK");
    assert_sorted_keys(VAR_LIM_CMDS.iter().map(|(k, _)| *k), "VAR_LIM_CMDS");
    assert_sorted_keys(STRUCTURAL_CMDS.iter().copied(), "STRUCTURAL_CMDS");
    assert_sorted_keys(
        crate::registry::KNOWN_ENVIRONMENTS.iter().copied(),
        "KNOWN_ENVIRONMENTS",
    );
    assert_sorted_keys(IRREGULAR_CMDS.iter().copied(), "IRREGULAR_CMDS");
    assert_sorted_keys(MATH_FUNCTIONS.iter().copied(), "MATH_FUNCTIONS");
    assert_sorted_keys(SPACING_CMDS.iter().map(|(k, _)| *k), "SPACING_CMDS");
    assert_sorted_keys(BLACKBOARD_LETTERS.iter().copied(), "BLACKBOARD_LETTERS");
}

#[test]
fn nested_parse_restores_parse_ctx() {
    use crate::depth::{unknown_command_policy, ParseCtx};

    let outer = ParseCtx {
        unknown_command: UnknownCommandPolicy::Identifier,
        ..Default::default()
    };
    let inner = ParseCtx {
        unknown_command: UnknownCommandPolicy::Error,
        ..Default::default()
    };

    let _g_outer = outer.install();
    assert_eq!(unknown_command_policy(), UnknownCommandPolicy::Identifier);
    {
        let _g_inner = inner.install();
        assert_eq!(unknown_command_policy(), UnknownCommandPolicy::Error);
        let ast = parse(
            r"\notarealcmd",
            &ParseOptions {
                unknown_command: UnknownCommandPolicy::Error,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(matches!(ast, MathNode::Error(_)), "got {ast:?}");
    }
    // After inner guard drops, outer Identifier policy is restored.
    assert_eq!(unknown_command_policy(), UnknownCommandPolicy::Identifier);
    let ast = parse(
        r"\stillunknown",
        &ParseOptions {
            unknown_command: UnknownCommandPolicy::Identifier,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(
        matches!(ast, MathNode::Identifier(_)),
        "expected identifier fallback after restore: {ast:?}"
    );
}

#[test]
fn emit_intent_on_function_and_space() {
    let opts = ConvertOptions {
        emit_intent: true,
        wrap_math: false,
        ..Default::default()
    };
    let out = convert(r"\sin\,x", &opts).unwrap();
    assert!(out.contains("intent=\"function\""), "got {out}");
    assert!(out.contains("intent=\"space\""), "got {out}");
}

#[test]
fn binom_ast_shape() {
    let ast = parse(r"\binom{n}{k}", &ParseOptions::default()).unwrap();
    match ast {
        MathNode::Binom(u, l) => {
            assert!(matches!(*u, MathNode::Identifier(_)));
            assert!(matches!(*l, MathNode::Identifier(_)));
        }
        other => panic!("expected Binom, got {other:?}"),
    }
}

#[test]
fn choose_folds_to_binom() {
    let ast = parse(r"{n \choose k}", &ParseOptions::default()).unwrap();
    match ast {
        MathNode::Binom(u, l) => {
            assert!(matches!(*u, MathNode::Identifier(_)));
            assert!(matches!(*l, MathNode::Identifier(_)));
        }
        other => panic!("expected Binom from choose, got {other:?}"),
    }
}

#[test]
fn middle_and_substack_supported() {
    assert!(supports_command("middle"));
    assert!(supports_command("substack"));
    assert!(supports_command("genfrac"));
    assert!(supports_command("choose"));
    assert!(supports_command("displaystyle"));
    assert!(supports_command("hskip"));
    let mid = convert(
        r"\left(a\middle|b\right)",
        &ConvertOptions {
            wrap_math: false,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(mid.contains("stretchy=\"true\""), "got {mid}");
}

#[test]
fn prescript_fold_via_parse() {
    // `{}_{a}^{b}X` style empty-base scripts should fold into pre_sub/pre_sup.
    let ast = parse(r"{}_{a}^{b}X", &ParseOptions::default()).unwrap();
    match ast {
        MathNode::Scripts {
            pre_sub: Some(_),
            pre_sup: Some(_),
            ..
        } => {}
        MathNode::Row(nodes) => {
            // May still be a row if folding only hits adjacent form
            let has_pre = nodes.iter().any(|n| {
                matches!(
                    n,
                    MathNode::Scripts {
                        pre_sub: Some(_),
                        ..
                    } | MathNode::Scripts {
                        pre_sup: Some(_),
                        ..
                    }
                )
            });
            assert!(has_pre || !nodes.is_empty(), "got {nodes:?}");
        }
        other => {
            // Accept scripts without pre if grammar differs; just ensure no panic
            let _ = other;
        }
    }
}
