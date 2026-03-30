use super::*;

// ... (省略之前的测试用例，完整保留) ...
#[test]
fn test_parse_number() {
    let mut input = "123";
    let result = parse_number.parse_next(&mut input).unwrap();
    assert_eq!(result, MathNode::Number("123".to_string()));
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
    assert_eq!(mathml, "<math><mrow><mfrac><mi>a</mi><mi>b</mi></mfrac><mn>42</mn></mrow></math>");
}

#[test]
fn test_parse_operator() {
    let mut input = "x + y = 2";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let expected = MathNode::Row(vec![
        MathNode::Identifier("x".to_string()),
        MathNode::Operator("+".to_string()),
        MathNode::Identifier("y".to_string()),
        MathNode::Operator("=".to_string()),
        MathNode::Number("2".to_string()),
    ]);
    assert_eq!(ast, expected);
}

#[test]
fn test_parse_superscript() {
    let mut input = "x^2";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let expected = MathNode::Superscript(
        Box::new(MathNode::Identifier("x".to_string())),
        Box::new(MathNode::Number("2".to_string())),
    );
    assert_eq!(ast, expected);
}

#[test]
fn test_parse_subscript() {
    let mut input = "a_i";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let expected = MathNode::Subscript(
        Box::new(MathNode::Identifier("a".to_string())),
        Box::new(MathNode::Identifier("i".to_string())),
    );
    assert_eq!(ast, expected);
}

#[test]
fn test_parse_subsup() {
    let mut input1 = "x_i^2";
    let mut input2 = "x^2_i";
    let ast1 = parse_row.parse_next(&mut input1).unwrap();
    let ast2 = parse_row.parse_next(&mut input2).unwrap();
    
    let expected = MathNode::SubSup {
        base: Box::new(MathNode::Identifier("x".to_string())),
        sub: Box::new(MathNode::Identifier("i".to_string())),
        sup: Box::new(MathNode::Number("2".to_string())),
    };
    
    assert_eq!(ast1, expected);
    assert_eq!(ast2, expected);
}

#[test]
fn test_parse_group() {
    let mut input = "{a + b}^2";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let expected = MathNode::Superscript(
        Box::new(MathNode::Row(vec![
            MathNode::Identifier("a".to_string()),
            MathNode::Operator("+".to_string()),
            MathNode::Identifier("b".to_string()),
        ])),
        Box::new(MathNode::Number("2".to_string())),
    );
    assert_eq!(ast, expected);
}

#[test]
fn test_mathml_generation_advanced() {
    let mut input = "x_i^2 + y_i^2 = 1";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast);
    let expected = "<mrow><msubsup><mi>x</mi><mi>i</mi><mn>2</mn></msubsup><mo>+</mo><msubsup><mi>y</mi><mi>i</mi><mn>2</mn></msubsup><mo>=</mo><mn>1</mn></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_parse_sqrt() {
    let mut input = "\\sqrt{x}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let expected = MathNode::Sqrt(Box::new(MathNode::Identifier("x".to_string())));
    assert_eq!(ast, expected);
}

#[test]
fn test_parse_root() {
    let mut input = "\\sqrt[3]{x}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let expected = MathNode::Root {
        index: Box::new(MathNode::Number("3".to_string())),
        content: Box::new(MathNode::Identifier("x".to_string())),
    };
    assert_eq!(ast, expected);
}

#[test]
fn test_parse_left_right() {
    let mut input = "\\left( x \\right)";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let expected = MathNode::Fenced {
        open: "(".to_string(),
        content: Box::new(MathNode::Identifier("x".to_string())),
        close: ")".to_string(),
    };
    assert_eq!(ast, expected);
}

#[test]
fn test_mathml_left_right_sqrt() {
    let mut input = "\\left[ \\sqrt[3]{x} \\right]";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast);
    let expected = "<mrow><mo stretchy=\"true\">[</mo><mroot><mi>x</mi><mn>3</mn></mroot><mo stretchy=\"true\">]</mo></mrow>";
    assert_eq!(mathml, expected);
}

// === 新增测试：命令符号字典 ===

#[test]
fn test_parse_symbols() {
    let mut input = "\\alpha + \\infty \\le \\sum_{i=1}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast);
    // \alpha 映射为 <mi>α</mi>
    // \infty 映射为 <mi>∞</mi>
    // \le 映射为 <mo>≤</mo>
    // \sum 映射为 <mo>∑</mo>，并且带有下标
    let expected = "<mrow><mi>α</mi><mo>+</mo><mi>∞</mi><mo>≤</mo><msub><mo>∑</mo><mrow><mi>i</mi><mo>=</mo><mn>1</mn></mrow></msub></mrow>";
    assert_eq!(mathml, expected);
}
