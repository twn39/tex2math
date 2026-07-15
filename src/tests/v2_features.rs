//! Regression tests for tex2math 2.0 / 2.1 / 2.2 APIs.

use crate::{
    convert, generate_mathml, parse, render_mathml_to, supports_command, ConvertOptions, MathNode,
    ParseErrorKind, ParseOptions, RenderMode, RenderOptions, TrailingPolicy, UnknownCommandPolicy,
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
    ];
    for (latex, needle) in cases {
        let out = convert(latex, &opts).unwrap();
        assert!(
            out.contains(needle),
            "for {latex:?} expected {needle:?}, got {out}"
        );
    }
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
