use super::*;

// ... (省略之前的 29 个测试，完整保留) ...
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
    assert_eq!(result, MathNode::Identifier("a".to_string()));
    assert_eq!(input, "bc"); // 剩余部分
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
    assert_eq!(
        mathml,
        "<math><mrow><mfrac><mi>a</mi><mi>b</mi></mfrac><mn>42</mn></mrow></math>"
    );
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

#[test]
fn test_parse_symbols() {
    let mut input = "\\alpha + \\infty \\le \\sum_{i=1}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast);
    let expected = "<mrow><mi>α</mi><mo>+</mo><mi>∞</mi><mo>≤</mo><munder><mo>∑</mo><mrow><mi>i</mi><mo>=</mo><mn>1</mn></mrow></munder></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_parse_matrix_environment() {
    let mut input = "\\begin{matrix} a & b \\\\ c & d \\end{matrix}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let expected = MathNode::Environment {
        name: "matrix".to_string(),
        rows: vec![
            vec![
                MathNode::Identifier("a".to_string()),
                MathNode::Identifier("b".to_string()),
            ],
            vec![
                MathNode::Identifier("c".to_string()),
                MathNode::Identifier("d".to_string()),
            ],
        ],
    };
    assert_eq!(ast, expected);
}

#[test]
fn test_mathml_pmatrix() {
    let mut input = "\\begin{pmatrix} 1 & 0 \\\\ 0 & 1 \\end{pmatrix}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast);
    let expected = "<mrow><mo stretchy=\"true\">(</mo><mtable><mtr><mtd><mn>1</mn></mtd><mtd><mn>0</mn></mtd></mtr><mtr><mtd><mn>0</mn></mtd><mtd><mn>1</mn></mtd></mtr></mtable><mo stretchy=\"true\">)</mo></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_mathml_large_operator_limits() {
    let mut input = "\\sum_{i=1}^n";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast);
    let expected =
        "<munderover><mo>∑</mo><mrow><mi>i</mi><mo>=</mo><mn>1</mn></mrow><mi>n</mi></munderover>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_texmath_quadratic_formula() {
    let mut input = "x=\\frac{-b\\pm\\sqrt{b^2-4ac}}{2a}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast);
    let expected = "<mrow><mi>x</mi><mo>=</mo><mfrac><mrow><mo>-</mo><mi>b</mi><mo>±</mo><msqrt><mrow><msup><mi>b</mi><mn>2</mn></msup><mo>-</mo><mn>4</mn><mi>a</mi><mi>c</mi></mrow></msqrt></mrow><mrow><mn>2</mn><mi>a</mi></mrow></mfrac></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_texmath_nested_fences() {
    let mut input = "2 = \\left( \\frac{\\left(3-x\\right) \\times 2}{3-x} \\right)";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast);
    let expected = "<mrow><mn>2</mn><mo>=</mo><mrow><mo stretchy=\"true\">(</mo><mfrac><mrow><mrow><mo stretchy=\"true\">(</mo><mrow><mn>3</mn><mo>-</mo><mi>x</mi></mrow><mo stretchy=\"true\">)</mo></mrow><mo>×</mo><mn>2</mn></mrow><mrow><mn>3</mn><mo>-</mo><mi>x</mi></mrow></mfrac><mo stretchy=\"true\">)</mo></mrow></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_nested_scripts_with_braces() {
    let mut input = "a^{b^c}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast);
    let expected = "<msup><mi>a</mi><msup><mi>b</mi><mi>c</mi></msup></msup>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_empty_group() {
    let mut input = "\\frac{}{} {}";
    let _ast = parse_row.parse_next(&mut input).unwrap();
}

#[test]
fn test_parse_font_styles() {
    let mut input = "\\mathbf{X} + \\mathbb{R}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast);
    let expected = "<mrow><mrow mathvariant=\"bold\"><mi>X</mi></mrow><mo>+</mo><mrow mathvariant=\"double-struck\"><mi>R</mi></mrow></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_parse_text_mode() {
    let mut input = "x = 1 \\text{ if } y > 0";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast);
    let expected = "<mrow><mi>x</mi><mo>=</mo><mn>1</mn><mtext> if </mtext><mi>y</mi><mo>&gt;</mo><mn>0</mn></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_parse_accents() {
    let mut input = "\\hat{y} + \\vec{v} + \\bar{x} + \\dot{x}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast);
    let expected = "<mrow><mover accent=\"true\"><mi>y</mi><mo>^</mo></mover><mo>+</mo><mover accent=\"true\"><mi>v</mi><mo>→</mo></mover><mo>+</mo><mover accent=\"true\"><mi>x</mi><mo>¯</mo></mover><mo>+</mo><mover accent=\"true\"><mi>x</mi><mo>˙</mo></mover></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_parse_math_functions() {
    let mut input = "\\sin x + \\log y";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast);
    let expected = "<mrow><mi mathvariant=\"normal\">sin</mi><mi>x</mi><mo>+</mo><mi mathvariant=\"normal\">log</mi><mi>y</mi></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_parse_lim_with_limits() {
    let mut input = "\\lim_{x \\to 0}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast);
    let expected = "<munder><mi mathvariant=\"normal\">lim</mi><mrow><mi>x</mi><mo>→</mo><mn>0</mn></mrow></munder>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_parse_explicit_spacing() {
    let mut input = "a \\quad b \\, c";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast);
    let expected = "<mrow><mi>a</mi><mspace width=\"1em\"/><mi>b</mi><mspace width=\"0.1667em\"/><mi>c</mi></mrow>";
    assert_eq!(mathml, expected);
}

// === 新增：错误恢复与宽容渲染测试 ===

#[test]
fn test_error_recovery_missing_brace() {
    // 经典错误：忘了写右括号
    let mut input = "\\frac{a}{b";

    // 我们期望解析不会 Panic，而是捕获错误，将出问题的部分包裹在 Error 里，
    // 外层不崩溃，甚至依然生成部分结果
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast);

    // 应该包含 merror 标签
    assert!(mathml.contains("<merror>"));
}

#[test]
fn test_error_recovery_unknown_environment() {
    // 环境没有被闭合
    let mut input = "\\begin{matrix} a & b";

    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast);

    assert!(mathml.contains("<merror>"));
}
