use crate::{parse_latex, MathNode};
use std::borrow::Cow;

#[test]
fn test_unified_api() {
    let math = "a & b \\\\ c & d";
    let ast = parse_latex(math).unwrap();
    match ast {
        MathNode::Environment { name, .. } => {
            assert!(
                name.starts_with("align"),
                "expected align-like env, got {name}"
            );
        }
        other => panic!("expected Environment AST, got {other:?}"),
    }
}

#[test]
fn test_trailing_garbage_is_error() {
    let err =
        parse_latex(r"x + 1 } leftover").expect_err("trailing non-whitespace must be reported");
    assert!(
        err.message.contains("trailing") || err.message.contains("Unexpected"),
        "unexpected message: {}",
        err.message
    );
    // Offset should point near the residual region (after the parseable prefix).
    assert!(
        err.offset() > 0,
        "offset should not be zero for mid-string residual"
    );
}

#[test]
fn test_trailing_whitespace_ok() {
    let ast = parse_latex("x + 1   \n\t  ").expect("trailing whitespace is fine");
    assert!(
        matches!(
            ast,
            MathNode::Row(_)
                | MathNode::Identifier(_)
                | MathNode::Scripts { .. }
                | MathNode::Operator(_)
                | MathNode::Number(_)
        ) || !matches!(ast, MathNode::Error(_))
    );
}

#[test]
fn test_simple_ast_shape_fraction() {
    // Layered unit-style check: assert AST shape, not MathML string.
    let ast = parse_latex(r"\frac{1}{2}").unwrap();
    match ast {
        MathNode::Fraction(num, den) => {
            assert_eq!(*num, MathNode::Number(Cow::Borrowed("1")));
            assert_eq!(*den, MathNode::Number(Cow::Borrowed("2")));
        }
        other => panic!("expected Fraction, got {other:?}"),
    }
}

#[test]
fn test_simple_ast_shape_scripts() {
    let ast = parse_latex(r"x^2").unwrap();
    match ast {
        MathNode::Scripts {
            base, sup: Some(s), ..
        } => {
            assert_eq!(*base, MathNode::Identifier(Cow::Borrowed("x")));
            assert_eq!(*s, MathNode::Number(Cow::Borrowed("2")));
        }
        other => panic!("expected Scripts, got {other:?}"),
    }
}
