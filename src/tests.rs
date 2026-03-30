use super::*;

#[test]
fn test_parse_number() {
    let mut input = "123";
    let result = parse_number.parse_next(&mut input).unwrap();
    assert_eq!(result, MathNode::Number("123".to_string()));
    assert_eq!(input, ""); 
}

#[test]
fn test_parse_ident() {
    let mut input = "abc";
    let result = parse_ident.parse_next(&mut input).unwrap();
    assert_eq!(result, MathNode::Identifier("abc".to_string()));
}

#[test]
fn test_parse_fraction() {
    let mut input = "\\frac{a}{12}";
    let result = parse_fraction.parse_next(&mut input).unwrap();
    let expected = MathNode::Fraction(
        Box::new(MathNode::Identifier("a".to_string())),
        Box::new(MathNode::Number("12".to_string())),
    );
    assert_eq!(result, expected);
    assert_eq!(input, "");
}

#[test]
fn test_ignore_spaces() {
    let mut input = "\\frac { a } { b }";
    let result = parse_fraction.parse_next(&mut input).unwrap();
    let expected = MathNode::Fraction(
        Box::new(MathNode::Identifier("a".to_string())),
        Box::new(MathNode::Identifier("b".to_string())),
    );
    assert_eq!(result, expected);
}

#[test]
fn test_generate_mathml_integration() {
    let mut input = "\\frac{a}{b} 42";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = format!("<math>{}</math>", generate_mathml(&ast));
    
    assert_eq!(
        mathml,
        "<math><mrow><mfrac><mi>a</mi><mi>b</mi></mfrac><mn>42</mn></mrow></math>"
    );
}
