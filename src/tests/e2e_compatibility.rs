use super::super::*;
use winnow::Parser;

#[test]
fn test_texmath_quadratic_formula() {
    let mut input = "x=\\frac{-b\\pm\\sqrt{b^2-4ac}}{2a}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<mrow><mi>x</mi><mo>=</mo><mfrac><mrow><mo>-</mo><mi>b</mi><mo>±</mo><msqrt><mrow><msup><mi>b</mi><mn>2</mn></msup><mo>-</mo><mn>4</mn><mi>a</mi><mi>c</mi></mrow></msqrt></mrow><mrow><mn>2</mn><mi>a</mi></mrow></mfrac></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_texmath_nested_fences() {
    let mut input = "2 = \\left( \\frac{\\left(3-x\\right) \\times 2}{3-x} \\right)";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<mrow><mn>2</mn><mo>=</mo><mrow><mo stretchy=\"true\">(</mo><mrow><mfrac><mrow><mrow><mo stretchy=\"true\">(</mo><mrow><mrow><mn>3</mn><mo>-</mo><mi>x</mi></mrow></mrow><mo stretchy=\"true\">)</mo></mrow><mo>×</mo><mn>2</mn></mrow><mrow><mn>3</mn><mo>-</mo><mi>x</mi></mrow></mfrac></mrow><mo stretchy=\"true\">)</mo></mrow></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_katex_continuous_relations() {
    let mut input = "x = y < z \\le 1";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected =
        "<mrow><mi>x</mi><mo>=</mo><mi>y</mi><mo>&lt;</mo><mi>z</mi><mo>≤</mo><mn>1</mn></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_texmath_calculus_integral() {
    let mut input = "\\int_0^\\infty f(x) \\, dx";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<mrow><msubsup><mo>∫</mo><mn>0</mn><mi>∞</mi></msubsup><mi>f</mi><mo>(</mo><mi>x</mi><mo>)</mo><mspace width=\"0.1667em\"/><mi>d</mi><mi>x</mi></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_texmath_nested_roots() {
    let mut input = "\\sqrt{\\sqrt{\\sqrt{x}}}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<msqrt><msqrt><msqrt><mi>x</mi></msqrt></msqrt></msqrt>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_texmath_empty_matrix_cells() {
    // 严苛的环境边界：带有空的单元格 (开头直接是 &，或末尾无内容)
    let mut input = "\\begin{pmatrix} & b \\\\ c & \\end{pmatrix}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // 如果空元素被正确处理，应该返回两行两列，有两处 <mtd></mtd> 或内部只带个空 <mrow>
    // 这其实也是在测试我们之前加入的允许长度为 0 的 parse_row!
    let expected = "<mrow><mo stretchy=\"true\">(</mo><mtable><mtr><mtd><mrow></mrow></mtd><mtd><mi>b</mi></mtd></mtr><mtr><mtd><mi>c</mi></mtd><mtd><mrow></mrow></mtd></mtr></mtable><mo stretchy=\"true\">)</mo></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_katex_text_mode_preservation() {
    let mut input = "\\text{a b }   \\text{ c}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<mrow><mtext>a b </mtext><mtext> c</mtext></mrow>";
    assert_eq!(mathml, expected);
}

// === 新增：上下文显示模式与强制界限控制 ===

#[test]
fn test_texmath_invisible_fences() {
    // 典型的微积分赋值边界：\left. 和 \right|_{t=0}
    // \left. 是一个极其常用的隐式定界符，不应该生成 <mo stretchy="true">.</mo>，而应为空或占位符
    let mut input = "\\left. \\frac{d}{dt} \\right|_{t=0}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // 这里只应该包含右侧的拉伸 |
    assert!(mathml.contains("<mo stretchy=\"true\">|</mo>"));
    // 不应该包含把 . 拉伸的标签
    assert!(!mathml.contains("<mo stretchy=\"true\">.</mo>"));
}

#[test]
fn test_texmath_overbrace_with_label() {
    // 带有上标的 overbrace
    // \overbrace 本身是 StretchOp(is_over: true)，会被 is_large_operator 捕获
    // 其后紧跟的 ^{term} 应该被当作上限 <mover>，从而形成嵌套的 mover
    let mut input = "\\overbrace{a+b}^{\\text{term}}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // 第一个 mover：包含拉伸大括号
    assert!(mathml.contains(
        "<mover><mrow><mi>a</mi><mo>+</mo><mi>b</mi></mrow><mo stretchy=\"true\">⏞</mo></mover>"
    ));
    // 第二个 mover：把 \text{term} 放在上面
    assert!(mathml.contains("<mover><mover><mrow><mi>a</mi><mo>+</mo><mi>b</mi></mrow><mo stretchy=\"true\">⏞</mo></mover><mtext>term</mtext></mover>"));
}

#[test]
fn test_texmath_max_with_subscript() {
    // \max 是一个大型数学函数，其 subscript 应该渲染为 under
    let mut input = "\\max_B X";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // 应该生成 <munder>，以 B 为下界
    assert!(mathml.contains("<munder><mi mathvariant=\"normal\">max</mi><mi>B</mi></munder>"));
}

// === 新增：隐形占位符与划线约分 ===

#[test]
fn test_complex_formula_with_new_features() {
    // 综合使用多项新功能：dfrac + langle/rangle + prime + operatorname
    let mut input = "\\left\\langle f', g \\right\\rangle = \\dfrac{1}{2}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    // 角括号定界符
    assert!(mathml.contains("<mo stretchy=\"true\">⟨</mo>"));
    // 撇号
    assert!(mathml.contains("<msup>"));
    // dfrac
    assert!(mathml.contains("<mstyle displaystyle=\"true\">"));
    assert!(mathml.contains("<mfrac>"));
}
