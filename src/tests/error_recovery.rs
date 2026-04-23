use super::super::*;
use winnow::Parser;

#[test]
fn test_empty_group() {
    let mut input = "\\frac{}{} {}";
    let _ast = parse_math.parse_next(&mut input).unwrap();
}

#[test]
fn test_error_recovery_missing_brace() {
    // 经典错误：忘了写右括号
    let mut input = "\\frac{a}{b";

    // 我们期望解析不会 Panic，而是捕获错误，将出问题的部分包裹在 Error 里，
    // 外层不崩溃，甚至依然生成部分结果
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // 应该包含 merror 标签
    assert!(mathml.contains("<merror>"));
}

#[test]
fn test_error_recovery_unknown_environment() {
    // 环境没有被闭合
    let mut input = "\\begin{matrix} a & b";

    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    assert!(mathml.contains("<merror>"));
}

// ==========================================
// 终极学术级边界与压力测试 (Inspired by KaTeX & texmath)
// ==========================================

#[test]
fn test_error_recovery_missing_denominator() {
    let mut input = "\\frac{a}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // 分母缺失应当触发一个 Missing denominator block 的错误节点，并且嵌在 mfrac 中
    assert!(mathml.contains("<mfrac>"));
    assert!(mathml.contains("<merror><mtext mathcolor=\"red\">Syntax Error: Missing denominator block</mtext></merror>"));
}

#[test]
fn test_error_recovery_nested_unclosed_groups() {
    // 外层大括号未闭合，但内部大括号完美闭合
    let mut input = "{a + {b + c }";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // 依然成功渲染了 `a + ` 以及内部的 `b + c`
    assert!(mathml.contains("<mi>a</mi><mo>+</mo><mrow><mi>b</mi><mo>+</mo><mi>c</mi></mrow>"));
    // 最外层的缺失触发了残缺警告 (由于引号和括号在 XML 中被转义了，所以是 &apos; )
    assert!(mathml.contains("<merror><mtext mathcolor=\"red\">Syntax Error: Missing &apos;}&apos;, found: &apos;&apos;</mtext></merror>"));
}

#[test]
fn test_error_recovery_unclosed_environment_complex() {
    // 一个有复杂内容的矩阵环境，忘了写 \end{bmatrix}
    let mut input = "\\begin{bmatrix} 1 & 2 \\\\ 3 & 4";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // 解析器依然将其作为未闭合矩阵渲染，但追加一个错误提示节点，而不丢失矩阵内容
    assert!(mathml.contains("<mtable><mtr><mtd><mn>1</mn></mtd><mtd><mn>2</mn></mtd></mtr><mtr><mtd><mn>3</mn></mtd><mtd><mn>4</mn></mtd></mtr></mtable>"));
    assert!(mathml.contains("<merror><mtext mathcolor=\"red\">Syntax Error: Missing \\end{bmatrix}</mtext></merror>"));
}

#[test]
fn test_error_recovery_orphan_commands() {
    // 孤立的 \limits 或 \nolimits 没有任何大操作符依附
    let mut input = "\\limits_{x \\to 0}";
    let ast = parse_math.parse_next(&mut input);

    // 虽然这不是合法的数学用法，但我们的引擎能够将其平稳地降为普通的样式节点或被隔离的 identifier，不会崩溃
    assert!(ast.is_ok());

    let mut input2 = "\\nolimits^2";
    let ast2 = parse_math.parse_next(&mut input2);
    assert!(ast2.is_ok());
}
