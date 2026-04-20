use super::super::*;
use winnow::Parser;

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
fn test_generate_mathml_integration() {
    let mut input = "\\frac{a}{b} 42";
    let ast = parse_math.parse_next(&mut input).unwrap();
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
    let ast = parse_math.parse_next(&mut input).unwrap();
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
fn test_parse_group() {
    let mut input = "{a + b}^2";
    let ast = parse_math.parse_next(&mut input).unwrap();
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
fn test_parse_sqrt() {
    let mut input = "\\sqrt{x}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let expected = MathNode::Sqrt(Box::new(MathNode::Identifier("x".to_string())));
    assert_eq!(ast, expected);
}

#[test]
fn test_parse_root() {
    let mut input = "\\sqrt[3]{x}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let expected = MathNode::Root {
        index: Box::new(MathNode::Number("3".to_string())),
        content: Box::new(MathNode::Identifier("x".to_string())),
    };
    assert_eq!(ast, expected);
}

#[test]
fn test_parse_left_right() {
    let mut input = "\\left( x \\right)";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let expected = MathNode::Fenced {
        open: "(".to_string(),
        content: Box::new(MathNode::Identifier("x".to_string())),
        close: ")".to_string(),
    };
    assert_eq!(ast, expected);
}

#[test]
fn test_parse_symbols() {
    let mut input = "\\alpha + \\infty \\le \\sum_{i=1}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<mrow><mi>α</mi><mo>+</mo><mi>∞</mi><mo>≤</mo><munder><mo>∑</mo><mrow><mi>i</mi><mo>=</mo><mn>1</mn></mrow></munder></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_parse_macro_aliases() {
    let mut input = "\\ne \\implies \\iff \\coloncolonequals \\sube \\dArr \\Rarr";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    // \ne -> ≠, \implies -> ⟹, \iff -> ⟺, \sube -> ⊆, \dArr -> ⇓, \Rarr -> ⇒
    let expected = "<mrow><mo>≠</mo><mo>⟹</mo><mo>⟺</mo><mi>\\coloncolonequals</mi><mo>⊆</mo><mo>⇓</mo><mo stretchy=\"true\">⇒</mo></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_parse_spacing_macros() {
    let mut input = "a \\, b \\: c \\; d \\! e \\enspace f \\enskip g \\quad h \\qquad i";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<mrow><mi>a</mi><mspace width=\"0.1667em\"/><mi>b</mi><mspace width=\"0.2222em\"/><mi>c</mi><mspace width=\"0.2778em\"/><mi>d</mi><mspace width=\"-0.1667em\"/><mi>e</mi><mspace width=\"0.5em\"/><mi>f</mi><mspace width=\"0.5em\"/><mi>g</mi><mspace width=\"1em\"/><mi>h</mi><mspace width=\"2em\"/><mi>i</mi></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_parse_escaped_symbols() {
    let mut input = "\\% \\$ \\{ \\} \\_ \\& \\#";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected =
        "<mrow><mi>%</mi><mi>$</mi><mo>{</mo><mo>}</mo><mi>_</mi><mi>&amp;</mi><mi>#</mi></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_parse_math_functions() {
    let mut input = "\\sin x + \\log y";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<mrow><mi mathvariant=\"normal\">sin</mi><mi>x</mi><mo>+</mo><mi mathvariant=\"normal\">log</mi><mi>y</mi></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_parse_explicit_spacing() {
    let mut input = "a \\quad b \\, c";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<mrow><mi>a</mi><mspace width=\"1em\"/><mi>b</mi><mspace width=\"0.1667em\"/><mi>c</mi></mrow>";
    assert_eq!(mathml, expected);
}

// === 新增：错误恢复与宽容渲染测试 ===

#[test]
fn test_algebra_standard_functions() {
    let mut input = "\\dim p, \\deg q, \\det m, \\ker\\phi";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // Check that all four standard functions are rendered correctly with normal mathvariant
    assert!(mathml.contains("<mi mathvariant=\"normal\">dim</mi>"));
    assert!(mathml.contains("<mi mathvariant=\"normal\">deg</mi>"));
    assert!(mathml.contains("<mi mathvariant=\"normal\">det</mi>"));
    assert!(mathml.contains("<mi mathvariant=\"normal\">ker</mi>"));

    // Check for the phi symbol rendering correctly
    assert!(mathml.contains("<mi>ϕ</mi>"));
}

#[test]
fn test_special_character_aliases() {
    let mut input = "\\AA, \\aa, \\O, \\o";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // Check that the Angstrom symbols and slashed O symbols are correctly parsed
    assert!(mathml.contains("<mi>Å</mi>"));
    assert!(mathml.contains("<mi>å</mi>"));
    assert!(mathml.contains("<mi>Ø</mi>"));
    assert!(mathml.contains("<mi>ø</mi>"));
}

#[test]
fn test_math_symbol_aliases() {
    let mut input = "\\N, \\R, \\Z, \\C, \\Q, \\H";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // Check that aliases correctly use the double-struck variant style
    assert!(mathml.contains("<mstyle mathvariant=\"double-struck\"><mi>N</mi></mstyle>"));
    assert!(mathml.contains("<mstyle mathvariant=\"double-struck\"><mi>R</mi></mstyle>"));
    assert!(mathml.contains("<mstyle mathvariant=\"double-struck\"><mi>Z</mi></mstyle>"));
    assert!(mathml.contains("<mstyle mathvariant=\"double-struck\"><mi>C</mi></mstyle>"));
    assert!(mathml.contains("<mstyle mathvariant=\"double-struck\"><mi>Q</mi></mstyle>"));
    assert!(mathml.contains("<mstyle mathvariant=\"double-struck\"><mi>H</mi></mstyle>"));
}

#[test]
fn test_var_greek_letters() {
    let mut input = "\\varGamma \\varDelta \\varTheta \\varLambda \\varXi \\varPi \\varSigma \\varPhi \\varUpsilon \\varOmega";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // Check that variant Greek letters correctly use the italic variant style
    assert!(mathml.contains("<mstyle mathvariant=\"italic\"><mi>Γ</mi></mstyle>"));
    assert!(mathml.contains("<mstyle mathvariant=\"italic\"><mi>Δ</mi></mstyle>"));
    assert!(mathml.contains("<mstyle mathvariant=\"italic\"><mi>Θ</mi></mstyle>"));
    assert!(mathml.contains("<mstyle mathvariant=\"italic\"><mi>Λ</mi></mstyle>"));
    assert!(mathml.contains("<mstyle mathvariant=\"italic\"><mi>Ξ</mi></mstyle>"));
    assert!(mathml.contains("<mstyle mathvariant=\"italic\"><mi>Π</mi></mstyle>"));
    assert!(mathml.contains("<mstyle mathvariant=\"italic\"><mi>Σ</mi></mstyle>"));
    assert!(mathml.contains("<mstyle mathvariant=\"italic\"><mi>Φ</mi></mstyle>"));
    assert!(mathml.contains("<mstyle mathvariant=\"italic\"><mi>Υ</mi></mstyle>"));
    assert!(mathml.contains("<mstyle mathvariant=\"italic\"><mi>Ω</mi></mstyle>"));
}

#[test]
fn test_vertical_arrow_fences() {
    let mut input = "\\left \\uparrow \\frac{a}{b} \\right \\downarrow \\quad";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // Check that the vertical arrows are correctly parsed as stretchy fences
    assert!(mathml.contains("<mo stretchy=\"true\">↑</mo>"));
    assert!(mathml.contains("<mo stretchy=\"true\">↓</mo>"));

    // Check that the fraction is inside the mrow
    assert!(mathml.contains("<mfrac><mi>a</mi><mi>b</mi></mfrac>"));
}

#[test]
fn test_parse_phantom() {
    // phantom 用于生成与内部内容等大但不显示的占位符，常用于手动对齐
    let mut input = "x + \\phantom{y + z} + a";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    assert!(mathml.contains("<mphantom><mrow><mi>y</mi><mo>+</mo><mi>z</mi></mrow></mphantom>"));
}

#[test]
fn test_parse_decimal_number() {
    // 小数应作为单个 <mn> 节点，而非 <mn>3</mn><mo>.</mo><mn>14</mn>
    let mut input = "3.14";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert_eq!(mathml, "<mn>3.14</mn>");
}

#[test]
fn test_decimal_in_expression() {
    // 小数出现在表达式中
    let mut input = "1.5 + 0.5";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mn>1.5</mn>"));
    assert!(mathml.contains("<mn>0.5</mn>"));
}

// --- Fix 3: \overline 作为 stretch op 而非 accent ---

#[test]
fn test_left_right_langle_rangle() {
    let mut input = "\\left\\langle x \\right\\rangle";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mo stretchy=\"true\">⟨</mo>"));
    assert!(mathml.contains("<mo stretchy=\"true\">⟩</mo>"));
}

#[test]
fn test_left_right_lfloor_rfloor() {
    let mut input = "\\left\\lfloor x \\right\\rfloor";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mo stretchy=\"true\">⌊</mo>"));
    assert!(mathml.contains("<mo stretchy=\"true\">⌋</mo>"));
}

#[test]
fn test_left_right_lceil_rceil() {
    let mut input = "\\left\\lceil x \\right\\rceil";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mo stretchy=\"true\">⌈</mo>"));
    assert!(mathml.contains("<mo stretchy=\"true\">⌉</mo>"));
}

#[test]
fn test_left_right_lvert_norm() {
    let mut input = "\\left\\lVert x \\right\\rVert";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mo stretchy=\"true\">∥</mo>"));
}

// --- Fix 8: 撇号（prime）上标 ---

#[test]
fn test_dfrac_inline_forced_display() {
    // 即使在 inline 模式下，dfrac 也强制 displaystyle="true"
    let mut input = "\\dfrac{1}{n}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Inline);
    assert!(mathml.contains("<mstyle displaystyle=\"true\">"));
}

// --- Fix 10: \operatorname ---

#[test]
fn test_operatorname_renders_as_function() {
    let mut input = "\\operatorname{rank}(A)";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    // rank 应该渲染为 mathvariant="normal" 的 mi 标签（与函数相同）
    assert!(mathml.contains("<mrow><mstyle mathvariant=\"normal\"><mrow><mi>r</mi><mi>a</mi><mi>n</mi><mi>k</mi></mrow></mstyle></mrow>"));
}

#[test]
fn test_not_in() {
    let mut input = "a \\not\\in B";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mo>∉</mo>"));
}

#[test]
fn test_not_subset() {
    let mut input = "A \\not\\subset B";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mo>⊄</mo>"));
}

#[test]
fn test_not_equal() {
    // \not= 应该生成 ≠（与 \neq 等价）
    let mut input = "a \\not= b";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mo>≠</mo>"));
}

#[test]
fn test_not_equiv() {
    let mut input = "a \\not\\equiv b";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mo>≢</mo>"));
}

// --- 端到端综合测试 ---

#[test]
fn test_parse_left_right_angle_brackets() {
    let mut input = "\\left< a, b \\right>";
    let result = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&result, RenderMode::Display);
    let expected = r#"<mrow><mo stretchy="true">⟨</mo><mrow><mrow><mi>a</mi><mo>,</mo><mi>b</mi></mrow></mrow><mo stretchy="true">⟩</mo></mrow>"#;
    assert_eq!(mathml, expected);
}

#[test]
fn test_parse_multiline_formula() {
    let mut input = "k \\; : \\; \\mathbb{R}^n \\times \\mathbb{R}^n \\; \\rightarrow \\mathbb{R}, \\qquad\n(s, t) \\mapsto \\left< \\Phi(s), \\Phi(t) \\right>";
    let result = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&result, RenderMode::Display);
    println!("MATHML: {}", mathml);
    println!("REMAINING: {}", input);
    assert_eq!(input, "");
}

#[test]
fn test_nabla_parens() {
    let mut input = "\\nabla f(x) = \\begin{bmatrix} 1 & 1 & 1 \\\\ 2x_1 & 2x_2 & -2x_3 \\\\ 2x_1 & -x_3 & -x_2 \\end{bmatrix}";
    let result = parse_math.parse_next(&mut input).unwrap();
    println!("{:?}", result);
    let mathml = generate_mathml(&result, RenderMode::Display);
    println!("{}", mathml);
}

#[test]
fn test_nested_environments_no_cross_boundary() {
    let mut input =
        "\\begin{align} a \\\\ \\begin{bmatrix} 1 \\\\ 2 \\end{bmatrix} \\\\ b \\end{align}";
    let result = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&result, RenderMode::Display);
    // Outer align should have 3 rows.
    // The middle row has a bmatrix with 2 rows.
    println!("{}", mathml);
    assert!(mathml.contains("<mtr><mtd><mrow><mo stretchy=\"true\">[</mo><mtable><mtr><mtd><mn>1</mn></mtd></mtr><mtr><mtd><mn>2</mn></mtd></mtr></mtable><mo stretchy=\"true\">]</mo></mrow></mtd></mtr>"));
}

#[test]
fn test_multiline_max_with_bullet() {
    let mut input = "\\text{max}  \\quad   0.25 L•X \\\\\n    \\text{    s.t.} \\quad   \\mathrm{diag}(X) = e \\\\\n                 \\qquad X \\succeq 0";
    let result = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&result, RenderMode::Display);
    let expected = r#"<mtable columnalign="right"><mtr><mtd><mrow><mtext>max</mtext><mspace width="1em"/><mn>0.25</mn><mi>L</mi><mo>•</mo><mi>X</mi></mrow></mtd></mtr><mtr><mtd><mrow><mtext>    s.t.</mtext><mspace width="1em"/><mstyle mathvariant="normal"><mrow><mi>d</mi><mi>i</mi><mi>a</mi><mi>g</mi></mrow></mstyle><mo>(</mo><mi>X</mi><mo>)</mo><mo>=</mo><mi>e</mi></mrow></mtd></mtr><mtr><mtd><mrow><mspace width="2em"/><mi>X</mi><mo>⪰</mo><mn>0</mn></mrow></mtd></mtr></mtable>"#;
    assert_eq!(mathml, expected);
}

// === 回归测试：嵌套 \begin{aligned} 在 \left[...\right] 中不得卡死 ===

#[test]
fn test_nested_aligned_in_left_right_no_freeze() {
    // 此公式之前导致无限循环（程序卡死），根本原因链：
    // Bug1: take_until 停在内层 \end{aligned}，截断外层 inner_str
    // Bug2: parse_left_right 失败回溯后 \left 成为不可解析原子
    // Bug3: separated(0..) / repeat(0..) 零消耗永远成功
    // Bug4: 空行守卫 inner_str.trim().is_empty() 永为 false → 无限循环
    let input = "\\begin{aligned}\n\
\\min & \\mathbf{1}^{T} u \\\\\n\
\\text{subject to} & \\left[\\begin{aligned}{\\sum_{i=1}^{p} \\lambda_{i} v_{i} v_{i}^{T}} & {e_{k}} \\\\ {e_{k}^{T}} & {u_{k}}\\end{aligned}\\right] \\succeq 0, \\quad k=1, \\ldots, n \\\\\n\
& \\lambda \\succeq 0 \\\\\n\
&  \\mathbf{1}^{T} \\lambda=1\n\
\\end{aligned}";

    let mut s = input;
    let ast = parse_math.parse_next(&mut s).expect("parse must not hang");
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // 必须产生 mtable（多行环境）
    assert!(
        mathml.contains("<mtable"),
        "Expected mtable, got: {}",
        &mathml[..200.min(mathml.len())]
    );
    // 关键符号可见
    assert!(mathml.contains("min"), "Expected 'min' in output");
    assert!(
        mathml.contains("mtext"),
        "Expected mtext for \\\\text{{subject to}}"
    );
    // 输出实质性内容，不是空壳
    assert!(
        mathml.len() > 200,
        "Output too short ({} chars), likely empty parse",
        mathml.len()
    );
}
