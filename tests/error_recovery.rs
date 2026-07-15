use tex2math::*;
use winnow::Parser;

#[test]
fn test_empty_group() {
    let mut input = "\\frac{}{} {}";
    let _ast = parse_math.parse_next(&mut input).unwrap();
}

#[test]
fn test_error_recovery_missing_brace() {
    let mut input = "\\frac{a}{b";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<merror>"));
}

#[test]
fn test_error_recovery_unknown_environment() {
    let mut input = "\\begin{matrix} a & b";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<merror>"));
}

#[test]
fn test_error_recovery_missing_denominator() {
    let mut input = "\\frac{a}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mfrac>"));
    assert!(mathml.contains(
        "<merror><mtext mathcolor=\"red\">Syntax Error: Missing denominator block</mtext></merror>"
    ));
}

#[test]
fn test_error_recovery_nested_unclosed_groups() {
    let mut input = "{a + {b + c }";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mi>a</mi><mo>+</mo><mrow><mi>b</mi><mo>+</mo><mi>c</mi></mrow>"));
    assert!(mathml.contains("<merror><mtext mathcolor=\"red\">Syntax Error: Missing &apos;}&apos;, found: &apos;&apos;</mtext></merror>"));
}

#[test]
fn test_error_recovery_unclosed_environment_complex() {
    let mut input = "\\begin{bmatrix} 1 & 2 \\\\ 3 & 4";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(
        mathml.contains("<mtable")
            && mathml.contains("<mn>1</mn>")
            && mathml.contains("<mn>4</mn>"),
        "got {mathml}"
    );
    assert!(mathml.contains(
        "<merror><mtext mathcolor=\"red\">Syntax Error: Missing \\end{bmatrix}</mtext></merror>"
    ));
}

#[test]
fn test_unknown_environment_name_flags_merror() {
    let out = convert(
        r"\begin{notanenv}x\end{notanenv}",
        &ConvertOptions {
            wrap_math: false,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(out.contains("<merror"), "got {out}");
    assert!(out.contains("Unknown environment"), "got {out}");
    assert!(out.contains("<mi>x</mi>"), "got {out}");
}

#[test]
fn test_error_recovery_orphan_commands() {
    let mut input = "\\limits_{x \\to 0}";
    let ast = parse_math.parse_next(&mut input);
    assert!(ast.is_ok());

    let mut input2 = "\\nolimits^2";
    let ast2 = parse_math.parse_next(&mut input2);
    assert!(ast2.is_ok());
}

#[test]
fn test_nested_braces_in_text() {
    let mut input = "\\text{hello {world} nested}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert_eq!(mathml, "<mtext>hello {world} nested</mtext>");
}

#[test]
fn test_escaped_braces_in_text() {
    let mut input = "\\text{hello \\{world\\} nested}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert_eq!(mathml, "<mtext>hello \\{world\\} nested</mtext>");
}

#[test]
fn test_unclosed_braces_in_text_recovery() {
    let mut input = "\\text{unclosed";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mtext>unclosed</mtext>"));
    assert!(mathml.contains("<merror><mtext mathcolor=\"red\">Syntax Error: Missing &apos;}&apos; in text command</mtext></merror>"));
}

#[test]
fn test_nested_brackets_in_extensible_arrow() {
    let mut input = "\\xrightarrow[a=[b]]{c}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<munderover>"));
    assert!(mathml.contains("<mo stretchy=\"true\">→</mo>"));
    assert!(mathml.contains("<mrow><mi>a</mi><mo>=</mo><mo>[</mo><mi>b</mi><mo>]</mo></mrow>"));
}

#[test]
fn test_nested_array_format() {
    let mut input = "\\begin{array}{p{2cm}c} a & b \\end{array}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mtable columnalign=\"center center\">"));
}
