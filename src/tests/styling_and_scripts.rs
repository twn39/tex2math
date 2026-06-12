use super::super::*;
use winnow::Parser;

#[test]
fn test_parse_superscript() {
    let mut input = "x^2";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let expected = MathNode::Scripts {
        base: Box::new(MathNode::Identifier("x".to_string())),
        sub: None,
        sup: Some(Box::new(MathNode::Number("2".to_string()))),
        behavior: LimitBehavior::Default,
        pre_sub: None,
        pre_sup: None,
    };
    assert_eq!(ast, expected);
}

#[test]
fn test_parse_subscript() {
    let mut input = "a_i";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let expected = MathNode::Scripts {
        base: Box::new(MathNode::Identifier("a".to_string())),
        sub: Some(Box::new(MathNode::Identifier("i".to_string()))),
        sup: None,
        behavior: LimitBehavior::Default,
        pre_sub: None,
        pre_sup: None,
    };
    assert_eq!(ast, expected);
}

#[test]
fn test_parse_subsup() {
    let mut input1 = "x_i^2";
    let mut input2 = "x^2_i";
    let ast1 = parse_math.parse_next(&mut input1).unwrap();
    let ast2 = parse_math.parse_next(&mut input2).unwrap();

    let expected = MathNode::Scripts {
        base: Box::new(MathNode::Identifier("x".to_string())),
        sub: Some(Box::new(MathNode::Identifier("i".to_string()))),
        sup: Some(Box::new(MathNode::Number("2".to_string()))),
        behavior: LimitBehavior::Default,
        pre_sub: None,
        pre_sup: None,
    };

    assert_eq!(ast1, expected);
    assert_eq!(ast2, expected);
}
