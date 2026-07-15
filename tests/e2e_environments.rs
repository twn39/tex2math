use std::borrow::Cow;
use tex2math::*;
use winnow::Parser;

#[test]
fn test_environment_trailing_row() {
    let mut input = "\\begin{bmatrix} a & b \\\\ c & d \\\\ \\end{bmatrix}";
    let result = parse_environment.parse_next(&mut input).unwrap();
    if let MathNode::Environment { name, rows, .. } = result {
        assert_eq!(name, "bmatrix");
        // Trailing empty row should be skipped, so only 2 rows
        assert_eq!(rows.len(), 2);
    } else {
        panic!("Expected Environment");
    }
}

#[test]
fn test_parse_matrix_environment() {
    let mut input = "\\begin{matrix} a & b \\\\ c & d \\end{matrix}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let expected = MathNode::Environment {
        name: Cow::Borrowed("matrix"),
        format: None,
        rows: vec![
            (
                vec![
                    MathNode::Identifier(Cow::Borrowed("a")),
                    MathNode::Identifier(Cow::Borrowed("b")),
                ],
                None,
            ),
            (
                vec![
                    MathNode::Identifier(Cow::Borrowed("c")),
                    MathNode::Identifier(Cow::Borrowed("d")),
                ],
                None,
            ),
        ],
    };
    assert_eq!(ast, expected);
}

#[test]
fn test_mathml_pmatrix() {
    let mut input = "\\begin{pmatrix} 1 & 0 \\\\ 0 & 1 \\end{pmatrix}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<mrow><mo stretchy=\"true\">(</mo><mtable><mtr><mtd><mn>1</mn></mtd><mtd><mn>0</mn></mtd></mtr><mtr><mtd><mn>0</mn></mtd><mtd><mn>1</mn></mtd></mtr></mtable><mo stretchy=\"true\">)</mo></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_environment_align_alignment() {
    let mut input = "\\begin{align} x &= 1 \\\\ y &= 2 \\end{align}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    let expected = "<mtable columnalign=\"right left\"><mtr><mtd><mi>x</mi></mtd><mtd><mrow><mo>=</mo><mn>1</mn></mrow></mtd></mtr><mtr><mtd><mi>y</mi></mtd><mtd><mrow><mo>=</mo><mn>2</mn></mrow></mtd></mtr></mtable>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_environment_cases_alignment() {
    let mut input = "\\begin{cases} 0 & x < 0 \\\\ 1 & x \\ge 0 \\end{cases}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    assert!(mathml.contains("<mrow><mo stretchy=\"true\">{</mo>"));
    assert!(mathml.contains("<mtable columnalign=\"left\">"));
}

#[test]
fn test_complex_cases_environment() {
    let mut input =
        "\\begin{cases} \\frac{1}{2} & -1 \\le x < 0 \\\\ 1 - x^2 & \\text{otherwise} \\end{cases}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    assert!(mathml.contains("columnalign=\"left\""));
    assert!(mathml.contains("<mfrac><mn>1</mn><mn>2</mn></mfrac>"));
    assert!(mathml.contains("<mo>≤</mo><mi>x</mi><mo>&lt;</mo><mn>0</mn>"));
    assert!(mathml.contains("<mtext>otherwise</mtext>"));
}

#[test]
fn test_environment_array_with_format() {
    let mut input = "\\begin{array}{r|cc} x & y & z \\end{array}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    assert!(mathml.contains("columnalign=\"right center center\""));
    assert!(mathml.contains("columnlines=\"solid none\""));
}

#[test]
fn test_environment_row_spacing() {
    let mut input = "\\begin{matrix} a \\\\ b \\\\[2em] c \\end{matrix}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    assert!(mathml.contains("<mtr style=\"margin-bottom: 2em;\">") || mathml.contains("<mpadded"));
}

#[test]
fn test_array_columnlines_count() {
    let mut input = "\\begin{array}{r|cc} x & y & z \\end{array}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("columnlines=\"solid none\""));
    assert!(!mathml.contains("columnlines=\"solid none none\""));
}

#[test]
fn test_parse_matrix_with_trailing_newline() {
    let mut input = "\\begin{bmatrix} 1 & V_i^* \\\\ V_i & W_{ii} \\\\ \\end{bmatrix} \\succeq 0";
    let result = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&result, RenderMode::Display);
    let expected = r#"<mrow><mrow><mo stretchy="true">[</mo><mtable><mtr><mtd><mn>1</mn></mtd><mtd><msubsup><mi>V</mi><mi>i</mi><mo>*</mo></msubsup></mtd></mtr><mtr><mtd><msub><mi>V</mi><mi>i</mi></msub></mtd><mtd><msub><mi>W</mi><mrow><mi>i</mi><mi>i</mi></mrow></msub></mtd></mtr></mtable><mo stretchy="true">]</mo></mrow><mo>⪰</mo><mn>0</mn></mrow>"#;
    assert_eq!(mathml, expected);
}
