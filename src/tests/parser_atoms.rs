use super::super::*;
use std::borrow::Cow;
use winnow::Parser;

#[test]
fn test_parse_number() {
    let mut input = "123";
    let result = parse_number.parse_next(&mut input).unwrap();
    assert_eq!(result, MathNode::Number(Cow::Borrowed("123")));
}

#[test]
fn test_parse_ident() {
    let mut input = "abc";
    let result = parse_ident.parse_next(&mut input).unwrap();
    assert_eq!(result, MathNode::Identifier(Cow::Borrowed("a")));
    assert_eq!(input, "bc"); // 剩余部分
}

#[test]
fn test_parse_fraction() {
    let mut input = "\\frac{a}{12}";
    let result = parse_fraction.parse_next(&mut input).unwrap();
    let expected = MathNode::Fraction(
        Box::new(MathNode::Identifier(Cow::Borrowed("a"))),
        Box::new(MathNode::Number(Cow::Borrowed("12"))),
    );
    assert_eq!(result, expected);
}

#[test]
fn test_parse_operator() {
    let mut input = "x + y = 2";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let expected = MathNode::Row(vec![
        MathNode::Identifier(Cow::Borrowed("x")),
        MathNode::Operator(Cow::Borrowed("+")),
        MathNode::Identifier(Cow::Borrowed("y")),
        MathNode::Operator(Cow::Borrowed("=")),
        MathNode::Number(Cow::Borrowed("2")),
    ]);
    assert_eq!(ast, expected);
}

#[test]
fn test_parse_group() {
    let mut input = "{a + b}^2";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let expected = MathNode::Scripts {
        base: Box::new(MathNode::Row(vec![
            MathNode::Identifier(Cow::Borrowed("a")),
            MathNode::Operator(Cow::Borrowed("+")),
            MathNode::Identifier(Cow::Borrowed("b")),
        ])),
        sub: None,
        sup: Some(Box::new(MathNode::Number(Cow::Borrowed("2")))),
        behavior: LimitBehavior::Default,
        pre_sub: None,
        pre_sup: None,
    };
    assert_eq!(ast, expected);
}

#[test]
fn test_parse_sqrt() {
    let mut input = "\\sqrt{x}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let expected = MathNode::Sqrt(Box::new(MathNode::Identifier(Cow::Borrowed("x"))));
    assert_eq!(ast, expected);
}

#[test]
fn test_parse_root() {
    let mut input = "\\sqrt[3]{x}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let expected = MathNode::Root {
        index: Box::new(MathNode::Number(Cow::Borrowed("3"))),
        content: Box::new(MathNode::Identifier(Cow::Borrowed("x"))),
    };
    assert_eq!(ast, expected);
}

#[test]
fn test_parse_left_right() {
    let mut input = "\\left( x \\right)";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let expected = MathNode::Fenced {
        open: Cow::Borrowed("("),
        content: Box::new(MathNode::Identifier(Cow::Borrowed("x"))),
        close: Cow::Borrowed(")"),
    };
    assert_eq!(ast, expected);
}
