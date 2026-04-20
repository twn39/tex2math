use super::super::*;
use winnow::Parser;

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
fn test_parse_lim_with_limits() {
    let mut input = "\\lim_{x \\to 0}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<munder><mi mathvariant=\"normal\">lim</mi><mrow><mi>x</mi><mo stretchy=\"true\">→</mo><mn>0</mn></mrow></munder>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_category_theory_limits() {
    let mut input = "\\injlim, \\varinjlim, \\projlim, \\varprojlim";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // Check that injlim and projlim render as normal mi functions with space
    assert!(mathml.contains("<mi mathvariant=\"normal\">inj lim</mi>"));
    assert!(mathml.contains("<mi mathvariant=\"normal\">proj lim</mi>"));

    // Check that varinjlim renders as lim with an under arrow
    assert!(mathml.contains(
        "<munder><mi mathvariant=\"normal\">lim</mi><mo stretchy=\"true\">→</mo></munder>"
    ));

    // Check that varprojlim renders as lim with an under left arrow
    assert!(mathml.contains(
        "<munder><mi mathvariant=\"normal\">lim</mi><mo stretchy=\"true\">←</mo></munder>"
    ));
}

#[test]
fn test_accents_with_spaces() {
    let mut input = "\\prime, \\backprime, f^\\prime, f', f'', f^{(3)}, \\dot y, \\ddot y";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // Check that \dot y and \ddot y work correctly with spaces before the atom
    assert!(mathml.contains("<mover accent=\"true\"><mi>y</mi><mo>˙</mo></mover>"));
    assert!(mathml.contains("<mover accent=\"true\"><mi>y</mi><mo>¨</mo></mover>"));
}

#[test]
fn test_sized_delimiters() {
    let mut input =
        "( \\bigl( \\Bigl( \\biggl( \\Biggl( \\dots \\Biggr] \\biggr] \\Bigr] \\bigr] ]";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // Check that sized delimiters are correctly parsed and have minsize/maxsize attributes
    assert!(mathml.contains("<mo minsize=\"1.2em\" maxsize=\"1.2em\">(</mo>"));
    assert!(mathml.contains("<mo minsize=\"1.8em\" maxsize=\"1.8em\">(</mo>"));
    assert!(mathml.contains("<mo minsize=\"2.4em\" maxsize=\"2.4em\">(</mo>"));
    assert!(mathml.contains("<mo minsize=\"3.0em\" maxsize=\"3.0em\">(</mo>"));

    assert!(mathml.contains("<mo minsize=\"3.0em\" maxsize=\"3.0em\">]</mo>"));
    assert!(mathml.contains("<mo minsize=\"2.4em\" maxsize=\"2.4em\">]</mo>"));
    assert!(mathml.contains("<mo minsize=\"1.8em\" maxsize=\"1.8em\">]</mo>"));
    assert!(mathml.contains("<mo minsize=\"1.2em\" maxsize=\"1.2em\">]</mo>"));
}

#[test]
fn test_explicit_limits_override() {
    // 用户显式使用 \limits 强制积分号上下界放在正上方
    let mut input = "\\int\\limits_0^1";
    let ast = parse_math.parse_next(&mut input).unwrap();

    // 即使在 Inline 模式下，\limits 也应该覆盖默认行为，强制生成 munderover
    let mathml = generate_mathml(&ast, RenderMode::Inline);
    assert!(mathml.contains("<munderover>"));
}

#[test]
fn test_explicit_nolimits_override() {
    // 用户显式使用 \nolimits 强制求和号变成角标
    let mut input = "\\sum\\nolimits_0^1";
    let ast = parse_math.parse_next(&mut input).unwrap();

    // 即使在 Display 模式下，\nolimits 也会强制它退化为 msubsup
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<msubsup>"));
}

// === 新增：高级环境与特殊对齐规则 ===

#[test]
fn test_iint_display_limits() {
    // \iint (∬) 即使在 display 模式下，默认也是 \nolimits 行为，因此应该生成 <msub> (右下角标)
    // 若要强制上下限，用户需显式使用 \limits。
    let mut input = "\\iint_D f";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<msub>"));
}

#[test]
fn test_bigvee_display_limits() {
    // \bigvee 在 display 模式下应该使用 munder
    let mut input = "\\bigvee_{i=1}^n";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<munderover>"));
}

// --- Fix 5: Style 使用 <mstyle mathvariant> ---
