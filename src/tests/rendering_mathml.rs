use super::super::*;
use winnow::Parser;

#[test]
fn test_mathml_generation_advanced() {
    let mut input = "x_i^2 + y_i^2 = 1";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<mrow><msubsup><mi>x</mi><mi>i</mi><mn>2</mn></msubsup><mo>+</mo><msubsup><mi>y</mi><mi>i</mi><mn>2</mn></msubsup><mo>=</mo><mn>1</mn></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_mathml_left_right_sqrt() {
    let mut input = "\\left[ \\sqrt[3]{x} \\right]";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<mrow><mo stretchy=\"true\">[</mo><mrow><mroot><mi>x</mi><mn>3</mn></mroot></mrow><mo stretchy=\"true\">]</mo></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_mathml_large_operator_limits() {
    let mut input = "\\sum_{i=1}^n";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected =
        "<munderover><mo>∑</mo><mrow><mi>i</mi><mo>=</mo><mn>1</mn></mrow><mi>n</mi></munderover>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_render_mode_inline_vs_display() {
    // 同样的 \sum_{i=1}^n
    let mut input = "\\sum_{i=1}^n";
    let ast = parse_math.parse_next(&mut input).unwrap();

    // 在 Inline 模式下，它必须退化为角标 msubsup
    let mathml_inline = generate_mathml(&ast, RenderMode::Inline);
    assert!(mathml_inline.contains("<msubsup>"));
    assert!(!mathml_inline.contains("<munderover>"));

    // 在 Display 模式下，它应该是正上正下 munderover
    let mathml_display = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml_display.contains("<munderover>"));
}

#[test]
fn test_mathml_contour_integral() {
    let mut input = "\\oint_{(x,y)\\in C} x^3\\, dx + 4y^2\\, dy";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<mrow><msub><mo>∮</mo><mrow><mo>(</mo><mi>x</mi><mo>,</mo><mi>y</mi><mo>)</mo><mo>∈</mo><mi>C</mi></mrow></msub><msup><mi>x</mi><mn>3</mn></msup><mspace width=\"0.1667em\"/><mi>d</mi><mi>x</mi><mo>+</mo><mn>4</mn><msup><mi>y</mi><mn>2</mn></msup><mspace width=\"0.1667em\"/><mi>d</mi><mi>y</mi></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_mathml_colored_quadratic_formula() {
    let mut input = "x=\\frac{{\\color{Blue}-b}\\pm\\sqrt{\\color{Red}b^2-4ac}}{\\color{Green}2a}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<mrow><mi>x</mi><mo>=</mo><mfrac><mrow><mstyle mathcolor=\"Blue\"><mrow><mo>-</mo><mi>b</mi></mrow></mstyle><mo>±</mo><msqrt><mstyle mathcolor=\"Red\"><mrow><msup><mi>b</mi><mn>2</mn></msup><mo>-</mo><mn>4</mn><mi>a</mi><mi>c</mi></mrow></mstyle></msqrt></mrow><mstyle mathcolor=\"Green\"><mrow><mn>2</mn><mi>a</mi></mrow></mstyle></mfrac></mrow>";
    assert_eq!(mathml, expected);
}
