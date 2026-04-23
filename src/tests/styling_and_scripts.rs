use super::super::*;
use winnow::Parser;

#[test]
fn test_parse_operator_with_scripts() {
    let mut input = "W = V V^* \\quad \\iff \\quad W_{ii} = | V_i |^2, \\quad W_{ik} = V_i \\; \\overline{V_k}, \\quad \\forall i, \\, k \\in \\{ 1, \\ldots, N \\}";
    let result = parse_math.parse_next(&mut input).unwrap();
    // 确保包含 ^*
    if let MathNode::Row(ref nodes) = result {
        assert!(nodes.len() > 5);
    } else {
        panic!("Expected Row");
    }

    let mathml = generate_mathml(&result, RenderMode::Display);
    let expected = r#"<mrow><mi>W</mi><mo>=</mo><mi>V</mi><msup><mi>V</mi><mo>*</mo></msup><mspace width="1em"/><mo>⟺</mo><mspace width="1em"/><msub><mi>W</mi><mrow><mi>i</mi><mi>i</mi></mrow></msub><mo>=</mo><mo>|</mo><msub><mi>V</mi><mi>i</mi></msub><msup><mo>|</mo><mn>2</mn></msup><mo>,</mo><mspace width="1em"/><msub><mi>W</mi><mrow><mi>i</mi><mi>k</mi></mrow></msub><mo>=</mo><msub><mi>V</mi><mi>i</mi></msub><mspace width="0.2778em"/><mover><msub><mi>V</mi><mi>k</mi></msub><mo stretchy="true">¯</mo></mover><mo>,</mo><mspace width="1em"/><mi>∀</mi><mi>i</mi><mo>,</mo><mspace width="0.1667em"/><mi>k</mi><mo>∈</mo><mo>{</mo><mn>1</mn><mo>,</mo><mo>…</mo><mo>,</mo><mi>N</mi><mo>}</mo></mrow>"#;
    assert_eq!(mathml, expected);
}

#[test]
fn test_parse_superscript() {
    let mut input = "x^2";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let expected = MathNode::Scripts {
        base: Box::new(MathNode::Identifier("x".to_string())),
        sub: None,
        sup: Some(Box::new(MathNode::Number("2".to_string()))),
        behavior: LimitBehavior::Default,
        pre_sub: None,
        pre_sup: None,
    };
    assert_eq!(ast, expected);
}

#[test]
fn test_parse_subscript() {
    let mut input = "a_i";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let expected = MathNode::Scripts {
        base: Box::new(MathNode::Identifier("a".to_string())),
        sub: Some(Box::new(MathNode::Identifier("i".to_string()))),
        sup: None,
        behavior: LimitBehavior::Default,
        pre_sub: None,
        pre_sup: None,
    };
    assert_eq!(ast, expected);
}

#[test]
fn test_parse_subsup() {
    let mut input1 = "x_i^2";
    let mut input2 = "x^2_i";
    let ast1 = parse_math.parse_next(&mut input1).unwrap();
    let ast2 = parse_math.parse_next(&mut input2).unwrap();

    let expected = MathNode::Scripts {
        base: Box::new(MathNode::Identifier("x".to_string())),
        sub: Some(Box::new(MathNode::Identifier("i".to_string()))),
        sup: Some(Box::new(MathNode::Number("2".to_string()))),
        behavior: LimitBehavior::Default,
        pre_sub: None,
        pre_sup: None,
    };

    assert_eq!(ast1, expected);
    assert_eq!(ast2, expected);
}

#[test]
fn test_nested_scripts_with_braces() {
    let mut input = "a^{b^c}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<msup><mi>a</mi><msup><mi>b</mi><mi>c</mi></msup></msup>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_parse_font_styles() {
    let mut input = "\\mathbf{X} + \\mathbb{R}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<mrow><mstyle mathvariant=\"bold\"><mi>X</mi></mstyle><mo>+</mo><mstyle mathvariant=\"double-struck\"><mi>R</mi></mstyle></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_parse_text_mode() {
    let mut input = "x = 1 \\text{ if } y > 0";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<mrow><mi>x</mi><mo>=</mo><mn>1</mn><mtext> if </mtext><mi>y</mi><mo>&gt;</mo><mn>0</mn></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_parse_accents() {
    let mut input = "\\hat{y} + \\vec{v} + \\bar{x} + \\dot{x}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<mrow><mover accent=\"true\"><mi>y</mi><mo>^</mo></mover><mo>+</mo><mover accent=\"true\"><mi>v</mi><mo>→</mo></mover><mo>+</mo><mover accent=\"true\"><mi>x</mi><mo>¯</mo></mover><mo>+</mo><mover accent=\"true\"><mi>x</mi><mo>˙</mo></mover></mrow>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_complex_operatorname_with_underset() {
    let mut input = "\\operatorname{\\underset{\\mathit{j\\,\\ne\\,i}}{median}} X_{i,j}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // operatorname should be wrapped in <mrow><mstyle mathvariant="normal">
    assert!(mathml.contains("<mrow><mstyle mathvariant=\"normal\">"));

    // It should contain an <munder> with "median" on top and the mathit block on bottom
    assert!(mathml.contains(
        "<munder><mrow><mi>m</mi><mi>e</mi><mi>d</mi><mi>i</mi><mi>a</mi><mi>n</mi></mrow>"
    ));
    assert!(mathml.contains("<mstyle mathvariant=\"italic\"><mrow><mi>j</mi><mspace width=\"0.1667em\"/><mo>≠</mo><mspace width=\"0.1667em\"/><mi>i</mi></mrow></mstyle>"));

    // The X_{i,j} part should be correctly parsed as a subscript
    assert!(mathml.contains("<msub><mi>X</mi><mrow><mi>i</mi><mo>,</mo><mi>j</mi></mrow></msub>"));
}

#[test]
fn test_parse_textcolor() {
    // 块级着色：只有括号里的被染色
    let mut input = "x + \\textcolor{red}{y + z} = 1";
    let ast = parse_math.parse_next(&mut input).unwrap();
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
    let ast = parse_math.parse_next(&mut input).unwrap();
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
    let ast = parse_math.parse_next(&mut input).unwrap();
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
fn test_complex_align_with_colors() {
    // 带有颜色切换和作用域染色的复杂多行方程
    let mut input =
        "\\begin{align} x &= \\color{red} y + 1 \\\\ \\textcolor{blue}{x - 1} &= y \\end{align}";
    let ast = parse_math.parse_next(&mut input).unwrap();
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
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // sum 应该是 munderover
    assert!(mathml.contains(
        "<munderover><mo>∑</mo><mrow><mi>i</mi><mo>=</mo><mn>1</mn></mrow><mi>∞</mi></munderover>"
    ));
    // sin 应该是 normal mi
    assert!(mathml.contains("<mi mathvariant=\"normal\">sin</mi>"));
    // lim 也应该是 munderover
    assert!(mathml.contains("<munder><mi mathvariant=\"normal\">lim</mi><mrow><mi>n</mi><mo stretchy=\"true\">→</mo><mi>∞</mi></mrow></munder>"));
    // 空格和文本
    assert!(mathml.contains("<mspace width=\"1em\"/><mtext>and</mtext><mspace width=\"1em\"/>"));
}

// === 新增：可拉伸的顶部/底部修饰 (Stretch Operators) ===

#[test]
fn test_underbrace_with_subscript() {
    // 带有标注的底部大括号
    let mut input = "\\underbrace{a + b + c}_{= X}";
    let ast = parse_math.parse_next(&mut input).unwrap();
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
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // 应该只有一个 mover 包含拉伸括号
    assert!(mathml.contains("<mover><mrow><msup><mi>x</mi><mn>2</mn></msup><mo>+</mo><msup><mi>y</mi><mn>2</mn></msup></mrow><mo stretchy=\"true\">⏞</mo></mover>"));
}

// === 新增：张量与前置角标 (Prescripts) ===

#[test]
fn test_prescripts_tensor_folding() {
    // 经典的核同位素或者张量写法
    let mut input = "{}_1^2 X";
    let ast = parse_math.parse_next(&mut input).unwrap();
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
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    assert!(mathml.contains("<mmultiscripts>"));
    // 右侧有 c 和 d
    assert!(mathml.contains("<mi>X</mi><mi>c</mi><mi>d</mi>"));
    // 左侧有 a 和 b
    assert!(mathml.contains("<mprescripts/><mi>a</mi><mi>b</mi></mmultiscripts>"));
}

// === 最新添加：深度边角排版测试 (Based on texmath/KaTeX cases) ===

#[test]
fn test_parse_cancel() {
    // 分式推导中极其常用的删除线
    let mut input = "\\frac{\\cancel{x} + y}{\\cancel{x}}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    assert!(mathml.contains("<menclose notation=\"updiagonalstrike\"><mi>x</mi></menclose>"));
}

#[test]
fn test_parse_xcancel() {
    // 交叉删除线 (大X)
    let mut input = "\\xcancel{Math}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    // 必须包含 text 节点，并且包裹在双向 strike 里
    assert!(mathml.contains("<menclose notation=\"updiagonalstrike downdiagonalstrike\">"));
}

// === 新增：终极数组格式与带参换行控制 ===

#[test]
fn test_overline_is_stretchy() {
    // \overline 应生成带 stretchy="true" 的 munder/mover，而非 accent
    let mut input = "\\overline{x + y}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    // 应该有 stretchy 属性（stretch op），而不是 accent="true"
    assert!(mathml.contains("stretchy=\"true\""));
    assert!(!mathml.contains("accent=\"true\""));
    assert!(mathml.contains("<mover>"));
}

#[test]
fn test_bar_is_accent() {
    // \bar 仍然应该是 accent
    let mut input = "\\bar{x}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("accent=\"true\""));
}

// --- Fix 4: 大型运算符列表扩充 ---

#[test]
fn test_mathbf_uses_mstyle() {
    let mut input = "\\mathbf{A}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mstyle mathvariant=\"bold\">"));
    assert!(!mathml.contains("<mrow mathvariant"));
}

#[test]
fn test_mathbb_uses_mstyle() {
    let mut input = "\\mathbb{R}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mstyle mathvariant=\"double-struck\">"));
}

// --- Fix 6: columnlines 修正 (N-1 个条目) ---

#[test]
fn test_single_prime() {
    // x' 等价于 x^{\prime}
    let mut input = "f'";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<msup>"));
    assert!(mathml.contains("<mi>′</mi>"));
}

#[test]
fn test_double_prime() {
    // f'' → ″
    let mut input = "f''";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<msup>"));
    assert!(mathml.contains("<mi>″</mi>"));
}

#[test]
fn test_prime_in_expression() {
    // f'(x) 不应奇怪地解析
    let mut input = "f'(x)";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<msup><mi>f</mi><mi>′</mi></msup>"));
}

// --- Fix 9: \dfrac 和 \tfrac ---

#[test]
fn test_dfrac_generates_displaystyle() {
    let mut input = "\\dfrac{a}{b}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    // dfrac 应包装在 mstyle displaystyle="true" 中
    assert!(mathml.contains("<mstyle displaystyle=\"true\">"));
    assert!(mathml.contains("<mfrac>"));
}

#[test]
fn test_tfrac_generates_textstyle() {
    let mut input = "\\tfrac{a}{b}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    // tfrac 应包装在 mstyle displaystyle="false" 中
    assert!(mathml.contains("<mstyle displaystyle=\"false\">"));
    assert!(mathml.contains("<mfrac>"));
}

#[test]
fn test_operatorname_with_subscript() {
    // \operatorname{tr} 后可以跟下标
    let mut input = "\\operatorname{tr}(A)";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mi mathvariant=\"normal\">tr</mi>"));
}

// --- Fix 11: \mathrm 作为 Style 而非 Text ---

#[test]
fn test_mathrm_renders_as_mstyle_normal() {
    // \mathrm{d} 应生成 mstyle mathvariant="normal"，而不是 mtext
    let mut input = "\\mathrm{d}x";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mstyle mathvariant=\"normal\">"));
    assert!(!mathml.contains("<mtext>"));
}

#[test]
fn test_mathrm_differential() {
    // 微积分中常见的用法：∫ f(x) \mathrm{d}x
    let mut input = "\\int f(x) \\mathrm{d} x";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mstyle mathvariant=\"normal\">"));
}

// --- Fix 12: \not 否定修饰符 ---
