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
    let mathml = format!(
        "<math>{}</math>",
        generate_mathml(&ast, RenderMode::Display)
    );
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
    let expected = MathNode::Scripts {
        base: Box::new(MathNode::Identifier("x".to_string())),
        sub: None,
        sup: Some(Box::new(MathNode::Number("2".to_string()))),
        behavior: LimitBehavior::Default,
        is_large_op: false,
        pre_sub: None,
        pre_sup: None,
    };
    assert_eq!(ast, expected);
}

#[test]
fn test_parse_subscript() {
    let mut input = "a_i";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let expected = MathNode::Scripts {
        base: Box::new(MathNode::Identifier("a".to_string())),
        sub: Some(Box::new(MathNode::Identifier("i".to_string()))),
        sup: None,
        behavior: LimitBehavior::Default,
        is_large_op: false,
        pre_sub: None,
        pre_sup: None,
    };
    assert_eq!(ast, expected);
}

#[test]
fn test_parse_subsup() {
    let mut input1 = "x_i^2";
    let mut input2 = "x^2_i";
    let ast1 = parse_row.parse_next(&mut input1).unwrap();
    let ast2 = parse_row.parse_next(&mut input2).unwrap();

    let expected = MathNode::Scripts {
        base: Box::new(MathNode::Identifier("x".to_string())),
        sub: Some(Box::new(MathNode::Identifier("i".to_string()))),
        sup: Some(Box::new(MathNode::Number("2".to_string()))),
        behavior: LimitBehavior::Default,
        is_large_op: false,
        pre_sub: None,
        pre_sup: None,
    };

    assert_eq!(ast1, expected);
    assert_eq!(ast2, expected);
}

#[test]
fn test_parse_group() {
    let mut input = "{a + b}^2";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let expected = MathNode::Scripts {
        base: Box::new(MathNode::Row(vec![
            MathNode::Identifier("a".to_string()),
            MathNode::Operator("+".to_string()),
            MathNode::Identifier("b".to_string()),
        ])),
        sub: None,
        sup: Some(Box::new(MathNode::Number("2".to_string()))),
        behavior: LimitBehavior::Default,
        is_large_op: false,
        pre_sub: None,
        pre_sup: None,
    };
    assert_eq!(ast, expected);
}

#[test]
fn test_mathml_generation_advanced() {
    let mut input = "x_i^2 + y_i^2 = 1";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
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
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<mrow><mo stretchy=\"true\">[</mo><mroot><mi>x</mi><mn>3</mn></mroot><mo stretchy=\"true\">]</mo></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_parse_symbols() {
    let mut input = "\\alpha + \\infty \\le \\sum_{i=1}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<mrow><mi>α</mi><mo>+</mo><mi>∞</mi><mo>≤</mo><munder><mo>∑</mo><mrow><mi>i</mi><mo>=</mo><mn>1</mn></mrow></munder></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_parse_macro_aliases() {
    let mut input = "\\ne \\implies \\iff \\coloncolonequals \\sube \\dArr \\Rarr";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    // \ne -> ≠, \implies -> ⟹, \iff -> ⟺, \sube -> ⊆, \dArr -> ⇓, \Rarr -> ⇒
    let expected = "<mrow><mo>≠</mo><mo>⟹</mo><mo>⟺</mo><mi>\\coloncolonequals</mi><mo>⊆</mo><mo>⇓</mo><mo>⇒</mo></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_parse_spacing_macros() {
    let mut input = "a \\, b \\: c \\; d \\! e \\enspace f \\enskip g \\quad h \\qquad i";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<mrow><mi>a</mi><mspace width=\"0.1667em\"/><mi>b</mi><mspace width=\"0.2222em\"/><mi>c</mi><mspace width=\"0.2778em\"/><mi>d</mi><mspace width=\"-0.1667em\"/><mi>e</mi><mspace width=\"0.5em\"/><mi>f</mi><mspace width=\"0.5em\"/><mi>g</mi><mspace width=\"1em\"/><mi>h</mi><mspace width=\"2em\"/><mi>i</mi></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_parse_escaped_symbols() {
    let mut input = "\\% \\$ \\{ \\} \\_ \\& \\#";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected =
        "<mrow><mi>%</mi><mi>$</mi><mo>{</mo><mo>}</mo><mi>_</mi><mi>&amp;</mi><mi>#</mi></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_parse_matrix_environment() {
    let mut input = "\\begin{matrix} a & b \\\\ c & d \\end{matrix}";
    let ast = parse_row.parse_next(&mut input).unwrap();
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
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<mrow><mo stretchy=\"true\">(</mo><mtable><mtr><mtd><mn>1</mn></mtd><mtd><mn>0</mn></mtd></mtr><mtr><mtd><mn>0</mn></mtd><mtd><mn>1</mn></mtd></mtr></mtable><mo stretchy=\"true\">)</mo></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_mathml_large_operator_limits() {
    let mut input = "\\sum_{i=1}^n";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected =
        "<munderover><mo>∑</mo><mrow><mi>i</mi><mo>=</mo><mn>1</mn></mrow><mi>n</mi></munderover>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_texmath_quadratic_formula() {
    let mut input = "x=\\frac{-b\\pm\\sqrt{b^2-4ac}}{2a}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<mrow><mi>x</mi><mo>=</mo><mfrac><mrow><mo>-</mo><mi>b</mi><mo>±</mo><msqrt><mrow><msup><mi>b</mi><mn>2</mn></msup><mo>-</mo><mn>4</mn><mi>a</mi><mi>c</mi></mrow></msqrt></mrow><mrow><mn>2</mn><mi>a</mi></mrow></mfrac></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_texmath_nested_fences() {
    let mut input = "2 = \\left( \\frac{\\left(3-x\\right) \\times 2}{3-x} \\right)";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<mrow><mn>2</mn><mo>=</mo><mrow><mo stretchy=\"true\">(</mo><mfrac><mrow><mrow><mo stretchy=\"true\">(</mo><mrow><mn>3</mn><mo>-</mo><mi>x</mi></mrow><mo stretchy=\"true\">)</mo></mrow><mo>×</mo><mn>2</mn></mrow><mrow><mn>3</mn><mo>-</mo><mi>x</mi></mrow></mfrac><mo stretchy=\"true\">)</mo></mrow></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_nested_scripts_with_braces() {
    let mut input = "a^{b^c}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
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
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<mrow><mrow mathvariant=\"bold\"><mi>X</mi></mrow><mo>+</mo><mrow mathvariant=\"double-struck\"><mi>R</mi></mrow></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_parse_text_mode() {
    let mut input = "x = 1 \\text{ if } y > 0";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<mrow><mi>x</mi><mo>=</mo><mn>1</mn><mtext> if </mtext><mi>y</mi><mo>&gt;</mo><mn>0</mn></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_parse_accents() {
    let mut input = "\\hat{y} + \\vec{v} + \\bar{x} + \\dot{x}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<mrow><mover accent=\"true\"><mi>y</mi><mo>^</mo></mover><mo>+</mo><mover accent=\"true\"><mi>v</mi><mo>→</mo></mover><mo>+</mo><mover accent=\"true\"><mi>x</mi><mo>¯</mo></mover><mo>+</mo><mover accent=\"true\"><mi>x</mi><mo>˙</mo></mover></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_parse_math_functions() {
    let mut input = "\\sin x + \\log y";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<mrow><mi mathvariant=\"normal\">sin</mi><mi>x</mi><mo>+</mo><mi mathvariant=\"normal\">log</mi><mi>y</mi></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_parse_lim_with_limits() {
    let mut input = "\\lim_{x \\to 0}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<munder><mi mathvariant=\"normal\">lim</mi><mrow><mi>x</mi><mo>→</mo><mn>0</mn></mrow></munder>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_parse_explicit_spacing() {
    let mut input = "a \\quad b \\, c";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
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
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // 应该包含 merror 标签
    assert!(mathml.contains("<merror>"));
}

#[test]
fn test_error_recovery_unknown_environment() {
    // 环境没有被闭合
    let mut input = "\\begin{matrix} a & b";

    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    assert!(mathml.contains("<merror>"));
}

// ==========================================
// 终极学术级边界与压力测试 (Inspired by KaTeX & texmath)
// ==========================================

#[test]
fn test_katex_continuous_relations() {
    let mut input = "x = y < z \\le 1";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected =
        "<mrow><mi>x</mi><mo>=</mo><mi>y</mi><mo>&lt;</mo><mi>z</mi><mo>≤</mo><mn>1</mn></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_texmath_calculus_integral() {
    let mut input = "\\int_0^\\infty f(x) \\, dx";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<mrow><munderover><mo>∫</mo><mn>0</mn><mi>∞</mi></munderover><mi>f</mi><mo>(</mo><mi>x</mi><mo>)</mo><mspace width=\"0.1667em\"/><mi>d</mi><mi>x</mi></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_texmath_nested_roots() {
    let mut input = "\\sqrt{\\sqrt{\\sqrt{x}}}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<msqrt><msqrt><msqrt><mi>x</mi></msqrt></msqrt></msqrt>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_texmath_empty_matrix_cells() {
    // 严苛的环境边界：带有空的单元格 (开头直接是 &，或末尾无内容)
    let mut input = "\\begin{pmatrix} & b \\\\ c & \\end{pmatrix}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // 如果空元素被正确处理，应该返回两行两列，有两处 <mtd></mtd> 或内部只带个空 <mrow>
    // 这其实也是在测试我们之前加入的允许长度为 0 的 parse_row!
    let expected = "<mrow><mo stretchy=\"true\">(</mo><mtable><mtr><mtd><mrow></mrow></mtd><mtd><mi>b</mi></mtd></mtr><mtr><mtd><mi>c</mi></mtd><mtd><mrow></mrow></mtd></mtr></mtable><mo stretchy=\"true\">)</mo></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_katex_text_mode_preservation() {
    let mut input = "\\text{a b }   \\text{ c}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<mrow><mtext>a b </mtext><mtext> c</mtext></mrow>";
    assert_eq!(mathml, expected);
}

// === 新增：上下文显示模式与强制界限控制 ===

#[test]
fn test_render_mode_inline_vs_display() {
    // 同样的 \sum_{i=1}^n
    let mut input = "\\sum_{i=1}^n";
    let ast = parse_row.parse_next(&mut input).unwrap();

    // 在 Inline 模式下，它必须退化为角标 msubsup
    let mathml_inline = generate_mathml(&ast, RenderMode::Inline);
    assert!(mathml_inline.contains("<msubsup>"));
    assert!(!mathml_inline.contains("<munderover>"));

    // 在 Display 模式下，它应该是正上正下 munderover
    let mathml_display = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml_display.contains("<munderover>"));
}

#[test]
fn test_explicit_limits_override() {
    // 用户显式使用 \limits 强制积分号上下界放在正上方
    let mut input = "\\int\\limits_0^1";
    let ast = parse_row.parse_next(&mut input).unwrap();

    // 即使在 Inline 模式下，\limits 也应该覆盖默认行为，强制生成 munderover
    let mathml = generate_mathml(&ast, RenderMode::Inline);
    assert!(mathml.contains("<munderover>"));
}

#[test]
fn test_explicit_nolimits_override() {
    // 用户显式使用 \nolimits 强制求和号变成角标
    let mut input = "\\sum\\nolimits_0^1";
    let ast = parse_row.parse_next(&mut input).unwrap();

    // 即使在 Display 模式下，\nolimits 也会强制它退化为 msubsup
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<msubsup>"));
}

// === 新增：高级环境与特殊对齐规则 ===

#[test]
fn test_environment_align_alignment() {
    // align 环境：用于多行等式对齐。奇数列右对齐，偶数列左对齐。
    let mut input = "\\begin{align} x &= 1 \\\\ y &= 2 \\end{align}";
    let ast = parse_row.parse_next(&mut input).unwrap();
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
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // cases 会有左边的大括号，且内部的 mtable 应该被标记为 columnalign="left"
    assert!(mathml.contains("<mrow><mo stretchy=\"true\">{</mo>"));
    assert!(mathml.contains("<mtable columnalign=\"left\">"));
}

// === 新增：高级文本处理与颜色系统 ===

#[test]
fn test_parse_textcolor() {
    // 块级着色：只有括号里的被染色
    let mut input = "x + \\textcolor{red}{y + z} = 1";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // 生成的 mathml 中，y + z 应该被包在带有 mathcolor="red" 的 mstyle 里
    assert!(mathml.contains("<mstyle mathcolor=\"red\">"));
    assert!(mathml.contains("<mi>y</mi><mo>+</mo><mi>z</mi>"));
    assert!(mathml.contains("</mstyle><mo>=</mo><mn>1</mn>"));
}

#[test]
fn test_parse_color_switch() {
    // 状态切换着色：从 \color 命令开始，直到当前作用域（Row）结束
    let mut input = "{a + \\color{blue} b + c} + d";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // b + c 被包在了 blue 里面，但 d 不应该被影响，a 也不应该被影响
    assert!(mathml.contains(
        "<mstyle mathcolor=\"blue\"><mrow><mi>b</mi><mo>+</mo><mi>c</mi></mrow></mstyle>"
    ));
    assert!(mathml.contains("<mo>+</mo><mi>d</mi>"));
}

#[test]
fn test_parse_colorbox_and_boxed() {
    let mut input = "\\boxed{\\colorbox{#FF0000}{x}}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // boxed 会生成 menclose notation="box"
    // colorbox 会生成 mstyle mathbackground="#FF0000"
    let expected = "<menclose notation=\"box\"><mstyle mathbackground=\"#FF0000\"><mi>x</mi></mstyle></menclose>";
    assert_eq!(mathml, expected);
}

// ==========================================
// 组合金字塔测试：检验所有高级特性的交叉兼容性
// ==========================================

#[test]
fn test_complex_cases_environment() {
    // 带有不等式、分数、文本模式的复杂分段函数
    let mut input =
        "\\begin{cases} \\frac{1}{2} & -1 \\le x < 0 \\\\ 1 - x^2 & \\text{otherwise} \\end{cases}";
    let ast = parse_row.parse_next(&mut input).unwrap();
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
fn test_complex_align_with_colors() {
    // 带有颜色切换和作用域染色的复杂多行方程
    let mut input =
        "\\begin{align} x &= \\color{red} y + 1 \\\\ \\textcolor{blue}{x - 1} &= y \\end{align}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // 验证列对齐：只有 2 列，所以是 right left
    assert!(mathml.contains("columnalign=\"right left\">"));

    // 验证第一行的贪婪颜色：y + 1 都必须是红色的
    assert!(mathml.contains(
        "<mstyle mathcolor=\"red\"><mrow><mi>y</mi><mo>+</mo><mn>1</mn></mrow></mstyle>"
    ));

    // 验证第二行的块级颜色：只有 x - 1 是蓝色的
    assert!(mathml.contains("<mstyle mathcolor=\"blue\"><mrow><mi>x</mi><mo>-</mo><mn>1</mn></mrow></mstyle></mtd><mtd><mrow><mo>=</mo><mi>y</mi></mrow>"));
}

#[test]
fn test_mixed_scripts_and_functions() {
    // 大运算符、上下标、函数和显式间距的大乱炖
    let mut input = "\\sum_{i=1}^{\\infty} \\sin(x_i) \\quad \\text{and} \\quad \\lim_{n \\to \\infty} \\frac{1}{n}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // sum 应该是 munderover
    assert!(mathml.contains(
        "<munderover><mo>∑</mo><mrow><mi>i</mi><mo>=</mo><mn>1</mn></mrow><mi>∞</mi></munderover>"
    ));
    // sin 应该是 normal mi
    assert!(mathml.contains("<mi mathvariant=\"normal\">sin</mi>"));
    // lim 也应该是 munderover
    assert!(mathml.contains("<munder><mi mathvariant=\"normal\">lim</mi><mrow><mi>n</mi><mo>→</mo><mi>∞</mi></mrow></munder>"));
    // 空格和文本
    assert!(mathml.contains("<mspace width=\"1em\"/><mtext>and</mtext><mspace width=\"1em\"/>"));
}

// === 新增：可拉伸的顶部/底部修饰 (Stretch Operators) ===

#[test]
fn test_underbrace_with_subscript() {
    // 带有标注的底部大括号
    let mut input = "\\underbrace{a + b + c}_{= X}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // 1. a + b + c 应该在一个 munder 里面，底下的括号应该是 U+23DF 且带有 stretchy="true"
    // 2. 外层还应该有一个 munder 或 msub，把 = X 挂在这个大括号的下面

    // 由于它是作为大运算符界限行为的延续，在 Display 模式下下界会生成为 munder。
    // 在我们的结构中，underbrace 本身会生成 munder，外部附着的 _ 会让它升级为 munder。
    assert!(mathml.contains("<mo stretchy=\"true\">⏟</mo>")); // 内部拉伸括号
    assert!(mathml.contains("<munder><munder>")); // 嵌套的两个 munder
    assert!(mathml.contains("<mrow><mo>=</mo><mi>X</mi></mrow>")); // 外部挂载的下标 label
}

#[test]
fn test_overbrace_no_label() {
    // 没有顶部标注的顶部大括号
    let mut input = "\\overbrace{x^2 + y^2}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // 应该只有一个 mover 包含拉伸括号
    assert!(mathml.contains("<mover><mrow><msup><mi>x</mi><mn>2</mn></msup><mo>+</mo><msup><mi>y</mi><mn>2</mn></msup></mrow><mo stretchy=\"true\">⏞</mo></mover>"));
}

// === 新增：张量与前置角标 (Prescripts) ===

#[test]
fn test_prescripts_tensor_folding() {
    // 经典的核同位素或者张量写法
    let mut input = "{}_1^2 X";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // 在我们进行 AST 后处理折叠后，它应该生成一个完美的 mmultiscripts
    // 其中基底是 X，右侧全为空 <none/>，左侧有 1 和 2
    assert!(mathml.contains("<mmultiscripts>"));
    assert!(mathml.contains("<mi>X</mi>"));
    assert!(mathml.contains("<mprescripts/>"));
    assert!(mathml.contains("<mn>1</mn><mn>2</mn>"));
}

#[test]
fn test_prescripts_with_postscripts() {
    // 四角双全的张量
    let mut input = "{}_a^b X_c^d";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    assert!(mathml.contains("<mmultiscripts>"));
    // 右侧有 c 和 d
    assert!(mathml.contains("<mi>X</mi><mi>c</mi><mi>d</mi>"));
    // 左侧有 a 和 b
    assert!(mathml.contains("<mprescripts/><mi>a</mi><mi>b</mi></mmultiscripts>"));
}

// === 最新添加：深度边角排版测试 (Based on texmath/KaTeX cases) ===

#[test]
fn test_texmath_invisible_fences() {
    // 典型的微积分赋值边界：\left. 和 \right|_{t=0}
    // \left. 是一个极其常用的隐式定界符，不应该生成 <mo stretchy="true">.</mo>，而应为空或占位符
    let mut input = "\\left. \\frac{d}{dt} \\right|_{t=0}";
    let ast = parse_row.parse_next(&mut input).unwrap();
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
    let ast = parse_row.parse_next(&mut input).unwrap();
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
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // 应该生成 <munder>，以 B 为下界
    assert!(mathml.contains("<munder><mi mathvariant=\"normal\">max</mi><mi>B</mi></munder>"));
}

// === 新增：隐形占位符与划线约分 ===

#[test]
fn test_parse_phantom() {
    // phantom 用于生成与内部内容等大但不显示的占位符，常用于手动对齐
    let mut input = "x + \\phantom{y + z} + a";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    assert!(mathml.contains("<mphantom><mrow><mi>y</mi><mo>+</mo><mi>z</mi></mrow></mphantom>"));
}

#[test]
fn test_parse_cancel() {
    // 分式推导中极其常用的删除线
    let mut input = "\\frac{\\cancel{x} + y}{\\cancel{x}}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    assert!(mathml.contains("<menclose notation=\"updiagonalstrike\"><mi>x</mi></menclose>"));
}

#[test]
fn test_parse_xcancel() {
    // 交叉删除线 (大X)
    let mut input = "\\xcancel{Math}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // 必须包含 text 节点，并且包裹在双向 strike 里
    assert!(mathml.contains("<menclose notation=\"updiagonalstrike downdiagonalstrike\">"));
}

// === 新增：终极数组格式与带参换行控制 ===

#[test]
fn test_environment_array_with_format() {
    // array 环境的精髓：它必须带有一个格式字符串，比如 r|cc
    let mut input = "\\begin{array}{r|cc} x & y & z \\end{array}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // 生成的 mtable 应该包含极其精确的属性映射
    assert!(mathml.contains("columnalign=\"right center center\""));
    // 竖线应该变成 columnlines，第一个参数后面是 solid，其余是 none
    assert!(mathml.contains("columnlines=\"solid none none\""));
}

#[test]
fn test_environment_row_spacing() {
    // 测试带参数的换行符 \\[1em]
    let mut input = "\\begin{matrix} a \\\\ b \\\\[2em] c \\end{matrix}";
    let ast = parse_row.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // 在 b 和 c 之间，或者是包含 b 的那一行的 mtr 上，应该带有间距注入
    // 我们预期它将间距转换为 mpadded 或者直接加在 mtr 的 style 上
    assert!(mathml.contains("<mtr style=\"margin-bottom: 2em;\">") || mathml.contains("<mpadded"));
}
