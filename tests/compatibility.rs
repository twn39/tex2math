use tex2math::*;
use winnow::Parser;

#[test]
fn test_texmath_quadratic_formula() {
    let mut input = "x=\\frac{-b\\pm\\sqrt{b^2-4ac}}{2a}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // Check key rendering structure
    assert!(mathml.contains("<mfrac><mrow><mo>-</mo><mi>b</mi><mo>±</mo><msqrt><mrow><msup><mi>b</mi><mn>2</mn></msup><mo>-</mo><mn>4</mn><mi>a</mi><mi>c</mi></mrow></msqrt></mrow><mrow><mn>2</mn><mi>a</mi></mrow></mfrac>"));
}

#[test]
fn test_texmath_nested_fences() {
    let mut input = "2 = \\left( \\frac{\\left(3-x\\right) \\times 2}{3-x} \\right)";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mo stretchy=\"true\">(</mo>"));
    assert!(mathml.contains("<mo stretchy=\"true\">)</mo>"));
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
    let mut input = "\\begin{pmatrix} & b \\\\ c & \\end{pmatrix}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<mrow><mo stretchy=\"true\">(</mo><mtable columnalign=\"center\"><mtr><mtd><mrow></mrow></mtd><mtd><mi>b</mi></mtd></mtr><mtr><mtd><mi>c</mi></mtd><mtd><mrow></mrow></mtd></mtr></mtable><mo stretchy=\"true\">)</mo></mrow>";
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

#[test]
fn test_texmath_invisible_fences() {
    let mut input = "\\left. \\frac{d}{dt} \\right|_{t=0}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mo stretchy=\"true\">|</mo>"));
    assert!(!mathml.contains("<mo stretchy=\"true\">.</mo>"));
}

#[test]
fn test_texmath_overbrace_with_label() {
    let mut input = "\\overbrace{a+b}^{\\text{term}}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains(
        "<mover><mrow><mi>a</mi><mo>+</mo><mi>b</mi></mrow><mo stretchy=\"true\">⏞</mo></mover>"
    ));
    assert!(mathml.contains("<mover><mover><mrow><mi>a</mi><mo>+</mo><mi>b</mi></mrow><mo stretchy=\"true\">⏞</mo></mover><mtext>term</mtext></mover>"));
}

#[test]
fn test_texmath_max_with_subscript() {
    let mut input = "\\max_B X";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<munder><mi mathvariant=\"normal\">max</mi><mi>B</mi></munder>"));
}

#[test]
fn test_complex_formula_with_new_features() {
    let mut input = "\\left\\langle f', g \\right\\rangle = \\dfrac{1}{2}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mo stretchy=\"true\">⟨</mo>"));
    assert!(mathml.contains("<msup>"));
    assert!(mathml.contains("<mstyle displaystyle=\"true\">"));
    assert!(mathml.contains("<mfrac>"));
}
