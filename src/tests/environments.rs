use super::super::*;
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
        name: "matrix".to_string(),
        format: None,
        rows: vec![
            (
                vec![
                    MathNode::Identifier("a".to_string()),
                    MathNode::Identifier("b".to_string()),
                ],
                None,
            ),
            (
                vec![
                    MathNode::Identifier("c".to_string()),
                    MathNode::Identifier("d".to_string()),
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
    // align 环境：用于多行等式对齐。奇数列右对齐，偶数列左对齐。
    let mut input = "\\begin{align} x &= 1 \\\\ y &= 2 \\end{align}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // 生成的 mtable 应该带有一个极其关键的属性：columnalign="right left"
    // 以保证在第一列和第二列之间实现紧密的等号对齐
    let expected = "<mtable columnalign=\"right left\"><mtr><mtd><mi>x</mi></mtd><mtd><mrow><mo>=</mo><mn>1</mn></mrow></mtd></mtr><mtr><mtd><mi>y</mi></mtd><mtd><mrow><mo>=</mo><mn>2</mn></mrow></mtd></mtr></mtable>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_environment_cases_alignment() {
    // cases 环境：用于分段函数。所有的列都应该是左对齐的！
    let mut input = "\\begin{cases} 0 & x < 0 \\\\ 1 & x \\ge 0 \\end{cases}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // cases 会有左边的大括号，且内部的 mtable 应该被标记为 columnalign="left"
    assert!(mathml.contains("<mrow><mo stretchy=\"true\">{</mo>"));
    assert!(mathml.contains("<mtable columnalign=\"left\">"));
}

// === 新增：高级文本处理与颜色系统 ===

#[test]
fn test_complex_cases_environment() {
    // 带有不等式、分数、文本模式的复杂分段函数
    let mut input =
        "\\begin{cases} \\frac{1}{2} & -1 \\le x < 0 \\\\ 1 - x^2 & \\text{otherwise} \\end{cases}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // 验证 cases 左对齐
    assert!(mathml.contains("columnalign=\"left\""));
    // 验证第一行的分式和不等式
    assert!(mathml.contains("<mfrac><mn>1</mn><mn>2</mn></mfrac>"));
    assert!(mathml.contains("<mo>≤</mo><mi>x</mi><mo>&lt;</mo><mn>0</mn>"));
    // 验证第二行的文本模式
    assert!(mathml.contains("<mtext>otherwise</mtext>"));
}

#[test]
fn test_environment_array_with_format() {
    // array 环境的精髓：它必须带有一个格式字符串，比如 r|cc
    let mut input = "\\begin{array}{r|cc} x & y & z \\end{array}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // r|cc 有 3 列，分隔符数为 N-1 = 2：第一个分隔符是 solid，第二个是 none
    assert!(mathml.contains("columnalign=\"right center center\""));
    // 改正后 columnlines 为 2 个条目（列数 - 1）
    assert!(mathml.contains("columnlines=\"solid none\""));
}

#[test]
fn test_environment_row_spacing() {
    // 测试带参数的换行符 \\[1em]
    let mut input = "\\begin{matrix} a \\\\ b \\\\[2em] c \\end{matrix}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // 在 b 和 c 之间，或者是包含 b 的那一行的 mtr 上，应该带有间距注入
    // 我们预期它将间距转换为 mpadded 或者直接加在 mtr 的 style 上
    assert!(mathml.contains("<mtr style=\"margin-bottom: 2em;\">") || mathml.contains("<mpadded"));
}

// ==========================================
// 以下是针对各项修复新增的测试
// ==========================================

// --- Fix 1: parse_number 支持小数 ---

#[test]
fn test_array_columnlines_count() {
    // r|cc: 3 列 → 2 个分隔符
    let mut input = "\\begin{array}{r|cc} x & y & z \\end{array}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    // columnlines 应为 2 个条目，而不是 3 个
    assert!(mathml.contains("columnlines=\"solid none\""));
    assert!(!mathml.contains("columnlines=\"solid none none\""));
}

// --- Fix 7: \left\langle 等多字符命令定界符 ---

#[test]
fn test_parse_matrix_with_trailing_newline() {
    let mut input = "\\begin{bmatrix} 1 & V_i^* \\\\ V_i & W_{ii} \\\\ \\end{bmatrix} \\succeq 0";
    let result = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&result, RenderMode::Display);
    let expected = r#"<mrow><mrow><mo stretchy="true">[</mo><mtable><mtr><mtd><mn>1</mn></mtd><mtd><msubsup><mi>V</mi><mi>i</mi><mo>*</mo></msubsup></mtd></mtr><mtr><mtd><msub><mi>V</mi><mi>i</mi></msub></mtd><mtd><msub><mi>W</mi><mrow><mi>i</mi><mi>i</mi></mrow></msub></mtd></mtr></mtable><mo stretchy="true">]</mo></mrow><mo>⪰</mo><mn>0</mn></mrow>"#;
    assert_eq!(mathml, expected);
}
