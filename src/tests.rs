use super::*;

// ... (省略之前的测试用例) ...
#[test]
fn test_parse_number() {
    let mut input = "123";
    let result = parse_number.parse_next(&mut input).unwrap();
    assert_eq!(result, MathNode::Number("123".to_string()));
}

#[test]
fn test_parse_ident() {
    let mut input = "abc";
    // 现在 parse_ident 每次只消费一个字母，这是为了防止 4ac 被当成一个变量
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
            vec![MathNode::Identifier("a".to_string()), MathNode::Identifier("b".to_string())],
            vec![MathNode::Identifier("c".to_string()), MathNode::Identifier("d".to_string())],
        ]
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
    let expected = "<munderover><mo>∑</mo><mrow><mi>i</mi><mo>=</mo><mn>1</mn></mrow><mi>n</mi></munderover>";
    assert_eq!(mathml, expected);
}

// === 新增：从 texmath 借鉴的高级边界测试 ===

#[test]
fn test_texmath_quadratic_formula() {
    // 求根公式：测试分式的嵌套、前置减号、带有加减的根号
    let mut input = "x=\\frac{-b\\pm\\sqrt{b^2-4ac}}{2a}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast);
    let expected = "<mrow><mi>x</mi><mo>=</mo><mfrac><mrow><mo>-</mo><mi>b</mi><mo>±</mo><msqrt><mrow><msup><mi>b</mi><mn>2</mn></msup><mo>-</mo><mn>4</mn><mi>a</mi><mi>c</mi></mrow></msqrt></mrow><mrow><mn>2</mn><mi>a</mi></mrow></mfrac></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_texmath_nested_fences() {
    // 测试复杂的定界符嵌套：\left( 内部有 \frac，\frac 内部又有 \left(
    let mut input = "2 = \\left( \\frac{\\left(3-x\\right) \\times 2}{3-x} \\right)";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast);
    let expected = "<mrow><mn>2</mn><mo>=</mo><mrow><mo stretchy=\"true\">(</mo><mfrac><mrow><mrow><mo stretchy=\"true\">(</mo><mrow><mn>3</mn><mo>-</mo><mi>x</mi></mrow><mo stretchy=\"true\">)</mo></mrow><mo>×</mo><mn>2</mn></mrow><mrow><mn>3</mn><mo>-</mo><mi>x</mi></mrow></mfrac><mo stretchy=\"true\">)</mo></mrow></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_nested_scripts_with_braces() {
    // 测试标准的上标嵌套 a^{b^c}
    let mut input = "a^{b^c}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast);
    let expected = "<msup><mi>a</mi><msup><mi>b</mi><mi>c</mi></msup></msup>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_empty_group() {
    // 测试空花括号和空分式，这在排版中是合法的占位符，不应该导致整个解析失败
    let mut input = "\\frac{}{} {}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    // 预期空组返回空 Row，或者能被容忍
    // 我们暂时只用 assert 确保解析不会抛出 Unwrap Error。
}
// === 新增：字体样式与文本模式 ===

#[test]
fn test_parse_font_styles() {
    let mut input = "\\mathbf{X} + \\mathbb{R}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast);
    
    // mathbf 应该生成带有 mathvariant="bold" 的 <mrow>，或者作用于内部的 <mi>
    // mathbb 对应 double-struck
    let expected = "<mrow><mrow mathvariant=\"bold\"><mi>X</mi></mrow><mo>+</mo><mrow mathvariant=\"double-struck\"><mi>R</mi></mrow></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_parse_text_mode() {
    // 文本模式中，空格是被保留的，并且字母连在一起！
    let mut input = "x = 1 \\text{ if } y > 0";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast);
    
    let expected = "<mrow><mi>x</mi><mo>=</mo><mn>1</mn><mtext> if </mtext><mi>y</mi><mo>&gt;</mo><mn>0</mn></mrow>";
    assert_eq!(mathml, expected);
}

// === 新增：数学重音与装饰 ===

#[test]
fn test_parse_accents() {
    let mut input = "\\hat{y} + \\vec{v} + \\bar{x} + \\dot{x}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast);
    
    // 重音符号在 MathML 中应该渲染为 <mover accent="true">
    let expected = "<mrow><mover accent=\"true\"><mi>y</mi><mo>^</mo></mover><mo>+</mo><mover accent=\"true\"><mi>v</mi><mo>→</mo></mover><mo>+</mo><mover accent=\"true\"><mi>x</mi><mo>¯</mo></mover><mo>+</mo><mover accent=\"true\"><mi>x</mi><mo>˙</mo></mover></mrow>";
    assert_eq!(mathml, expected);
}


// === 新增：数学函数与显式空格 ===

#[test]
fn test_parse_math_functions() {
    let mut input = "\\sin x + \\log y";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast);
    
    // sin 和 log 应该被解析为普通的 Operator 或带有特定样式的 mi，这里我们采用最普遍的 <mo> 或带属性的 <mi>
    // 参考规范，我们将它们映射为 <mi mathvariant="normal">
    let expected = "<mrow><mi mathvariant=\"normal\">sin</mi><mi>x</mi><mo>+</mo><mi mathvariant=\"normal\">log</mi><mi>y</mi></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_parse_lim_with_limits() {
    // 像 lim, max, min 这种函数，不仅是非斜体的，它还能接受上下界！
    let mut input = "\\lim_{x \\to 0}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast);
    
    // 必须生成 <munder>，而不是 <msub>
    let expected = "<munder><mi mathvariant=\"normal\">lim</mi><mrow><mi>x</mi><mo>→</mo><mn>0</mn></mrow></munder>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_parse_explicit_spacing() {
    let mut input = "a \\quad b \\, c";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast);
    
    // \quad 等价于 1em, \, 等价于 0.1667em
    let expected = "<mrow><mi>a</mi><mspace width=\"1em\"/><mi>b</mi><mspace width=\"0.1667em\"/><mi>c</mi></mrow>";
    assert_eq!(mathml, expected);
}

