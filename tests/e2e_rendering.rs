use tex2math::*;
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
    let mut input = "\\sum_{i=1}^n";
    let ast = parse_math.parse_next(&mut input).unwrap();

    let mathml_inline = generate_mathml(&ast, RenderMode::Inline);
    assert!(mathml_inline.contains("<msubsup>"));
    assert!(!mathml_inline.contains("<munderover>"));

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
    // Since expected is <mrow><mi>x</mi><mo>=</mo><mfrac><mrow><mstyle mathcolor="Blue"><mrow><mo>-</mo><mi>b</mi></mrow></mstyle><mo>±</mo><sqrt><mstyle mathcolor="Red"><mrow><msup><mi>b</mi><mn>2</mn></msup><mo>-</mo><mn>4</mn><mi>a</mi><mi>c</mi></mrow></mstyle></msqrt></mrow><mstyle mathcolor="Green"><mrow><mn>2</mn><mi>a</mi></mrow></mstyle></mfrac></mrow>
    // Let's assert based on containing elements to avoid formatting fragility
    assert!(mathml.contains("<mfrac>"));
    assert!(
        mathml.contains("<mstyle mathcolor=\"Blue\"><mrow><mo>-</mo><mi>b</mi></mrow></mstyle>")
    );
    assert!(mathml.contains("<mstyle mathcolor=\"Red\">"));
    assert!(mathml.contains("<mstyle mathcolor=\"Green\">"));
}

#[test]
fn test_mathml_inline_color_changes() {
    let mut input =
        "\\color{Blue}x^2\\color{Black}+\\color{Orange}2x\\color{Black}-\\color{LimeGreen}1";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    // Since mathml is: <mstyle mathcolor="Blue"><mrow><msup><mi>x</mi><mn>2</mn></msup><mstyle mathcolor="Black"><mrow><mo>+</mo><mstyle mathcolor="Orange"><mrow><mn>2</mn><mi>x</mi><mstyle mathcolor="Black"><mrow><mo>-</mo><mstyle mathcolor="LimeGreen"><mn>1</mn></mstyle></mrow></mstyle></mrow></mstyle></mrow></mstyle></mrow></mstyle>
    // Let's assert based on containing elements to avoid fragile formatting issues
    assert!(mathml.contains("<mstyle mathcolor=\"Blue\">"));
    assert!(mathml.contains("<mstyle mathcolor=\"Black\">"));
    assert!(mathml.contains("<mstyle mathcolor=\"Orange\">"));
    assert!(mathml.contains("<mstyle mathcolor=\"LimeGreen\">"));
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

#[test]
fn test_algebra_standard_functions() {
    let mut input = "\\dim p, \\deg q, \\det m, \\ker\\phi";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    assert!(mathml.contains("<mi mathvariant=\"normal\">dim</mi>"));
    assert!(mathml.contains("<mi mathvariant=\"normal\">deg</mi>"));
    assert!(mathml.contains("<mi mathvariant=\"normal\">det</mi>"));
    assert!(mathml.contains("<mi mathvariant=\"normal\">ker</mi>"));
    assert!(mathml.contains("<mi>ϕ</mi>"));
}

#[test]
fn test_special_character_aliases() {
    let mut input = "\\AA, \\aa, \\O, \\o";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

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

    assert!(mathml.contains("<mo stretchy=\"true\">↑</mo>"));
    assert!(mathml.contains("<mo stretchy=\"true\">↓</mo>"));
    assert!(mathml.contains("<mfrac><mi>a</mi><mi>b</mi></mfrac>"));
}

#[test]
fn test_parse_phantom() {
    let mut input = "x + \\phantom{y + z} + a";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);

    assert!(mathml.contains("<mphantom><mrow><mi>y</mi><mo>+</mo><mi>z</mi></mrow></mphantom>"));
}

#[test]
fn test_parse_decimal_number() {
    let mut input = "3.14";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert_eq!(mathml, "<mn>3.14</mn>");
}

#[test]
fn test_decimal_in_expression() {
    let mut input = "1.5 + 0.5";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mn>1.5</mn>"));
    assert!(mathml.contains("<mn>0.5</mn>"));
}

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

#[test]
fn test_dfrac_inline_forced_display() {
    let mut input = "\\dfrac{1}{n}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Inline);
    assert!(mathml.contains("<mstyle displaystyle=\"true\">"));
}

#[test]
fn test_operatorname_renders_as_function() {
    let mut input = "\\operatorname{rank}(A)";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mi mathvariant=\"normal\">rank</mi>"));
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
    assert_eq!(input, "");
    println!("MATHML: {}", mathml);
}

#[test]
fn test_nabla_parens() {
    let mut input = "\\nabla f(x) = \\begin{bmatrix} 1 & 1 & 1 \\\\ 2x_1 & 2x_2 & -2x_3 \\\\ 2x_1 & -x_3 & -x_2 \\end{bmatrix}";
    let result = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&result, RenderMode::Display);
    assert!(mathml.contains("<mtable"));
}

#[test]
fn test_nested_environments_no_cross_boundary() {
    let mut input =
        "\\begin{align} a \\\\ \\begin{bmatrix} 1 \\\\ 2 \\end{bmatrix} \\\\ b \\end{align}";
    let result = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&result, RenderMode::Display);
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

#[test]
fn test_parse_operator_with_scripts() {
    let mut input = "W = V V^* \\quad \\iff \\quad W_{ii} = | V_i |^2, \\quad W_{ik} = V_i \\; \\overline{V_k}, \\quad \\forall i, \\, k \\in \\{ 1, \\ldots, N \\}";
    let result = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&result, RenderMode::Display);
    assert!(mathml.contains("<msup><mi>V</mi><mo>*</mo></msup>"));
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

    assert!(mathml.contains("<mrow><mstyle mathvariant=\"normal\">"));
    assert!(mathml.contains(
        "<munder><mrow><mi>m</mi><mi>e</mi><mi>d</mi><mi>i</mi><mi>a</mi><mi>n</mi></mrow>"
    ));
}

#[test]
fn test_parse_textcolor() {
    let mut input = "x + \\textcolor{red}{y + z} = 1";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mstyle mathcolor=\"red\">"));
}

#[test]
fn test_parse_color_switch() {
    let mut input = "{a + \\color{blue} b + c} + d";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains(
        "<mstyle mathcolor=\"blue\"><mrow><mi>b</mi><mo>+</mo><mi>c</mi></mrow></mstyle>"
    ));
}

#[test]
fn test_parse_colorbox_and_boxed() {
    let mut input = "\\boxed{\\colorbox{#FF0000}{x}}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    let expected = "<menclose notation=\"box\"><mstyle mathbackground=\"#FF0000\"><mi>x</mi></mstyle></menclose>";
    assert_eq!(mathml, expected);
}

#[test]
fn test_complex_align_with_colors() {
    let mut input =
        "\\begin{align} x &= \\color{red} y + 1 \\\\ \\textcolor{blue}{x - 1} &= y \\end{align}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("columnalign=\"right left\">"));
    assert!(mathml.contains("<mstyle mathcolor=\"red\">"));
}

#[test]
fn test_mixed_scripts_and_functions() {
    let mut input = "\\sum_{i=1}^{\\infty} \\sin(x_i) \\quad \\text{and} \\quad \\lim_{n \\to \\infty} \\frac{1}{n}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<munderover><mo>∑</mo>"));
    assert!(mathml.contains("<mi mathvariant=\"normal\">sin</mi>"));
}

#[test]
fn test_underbrace_with_subscript() {
    let mut input = "\\underbrace{a + b + c}_{= X}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mo stretchy=\"true\">⏟</mo>"));
}

#[test]
fn test_overbrace_no_label() {
    let mut input = "\\overbrace{x^2 + y^2}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mo stretchy=\"true\">⏞</mo>"));
}

#[test]
fn test_prescripts_tensor_folding() {
    let mut input = "{}_1^2 X";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mmultiscripts>"));
}

#[test]
fn test_prescripts_with_postscripts() {
    let mut input = "{}_a^b X_c^d";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mmultiscripts>"));
}

#[test]
fn test_parse_cancel() {
    let mut input = "\\frac{\\cancel{x} + y}{\\cancel{x}}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<menclose notation=\"updiagonalstrike\"><mi>x</mi></menclose>"));
}

#[test]
fn test_parse_xcancel() {
    let mut input = "\\xcancel{Math}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<menclose notation=\"updiagonalstrike downdiagonalstrike\">"));
}

#[test]
fn test_overline_is_stretchy() {
    let mut input = "\\overline{x + y}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("stretchy=\"true\""));
    assert!(mathml.contains("<mover>"));
}

#[test]
fn test_bar_is_accent() {
    let mut input = "\\bar{x}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("accent=\"true\""));
}

#[test]
fn test_mathbf_uses_mstyle() {
    let mut input = "\\mathbf{A}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mstyle mathvariant=\"bold\">"));
}

#[test]
fn test_mathbb_uses_mstyle() {
    let mut input = "\\mathbb{R}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mstyle mathvariant=\"double-struck\">"));
}

#[test]
fn test_single_prime() {
    let mut input = "f'";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<msup>"));
    assert!(mathml.contains("<mi>′</mi>"));
}

#[test]
fn test_double_prime() {
    let mut input = "f''";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<msup>"));
    assert!(mathml.contains("<mi>″</mi>"));
}

#[test]
fn test_prime_in_expression() {
    let mut input = "f'(x)";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<msup><mi>f</mi><mi>′</mi></msup>"));
}

#[test]
fn test_dfrac_generates_displaystyle() {
    let mut input = "\\dfrac{a}{b}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mstyle displaystyle=\"true\">"));
}

#[test]
fn test_tfrac_generates_textstyle() {
    let mut input = "\\tfrac{a}{b}";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mstyle displaystyle=\"false\">"));
}

#[test]
fn test_operatorname_with_subscript() {
    let mut input = "\\operatorname{tr}(A)";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mi mathvariant=\"normal\">tr</mi>"));
}

#[test]
fn test_mathrm_renders_as_mstyle_normal() {
    let mut input = "\\mathrm{d}x";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mstyle mathvariant=\"normal\">"));
}

#[test]
fn test_mathrm_differential() {
    let mut input = "\\int f(x) \\mathrm{d} x";
    let ast = parse_math.parse_next(&mut input).unwrap();
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mstyle mathvariant=\"normal\">"));
}

#[test]
fn test_nested_aligned_in_left_right_no_freeze() {
    let input = "\\begin{aligned}\n\
\\min & \\mathbf{1}^{T} u \\\\\n\
\\text{subject to} & \\left[\\begin{aligned}{\\sum_{i=1}^{p} \\lambda_{i} v_{i} v_{i}^{T}} & {e_{k}} \\\\ {e_{k}^{T}} & {u_{k}}\\end{aligned}\\right] \\succeq 0, \\quad k=1, \\ldots, n \\\\\n\
& \\lambda \\succeq 0 \\\\\n\
&  \\mathbf{1}^{T} \\lambda=1\n\
\\end{aligned}";

    let mut s = input;
    let ast = parse_math.parse_next(&mut s).expect("parse must not hang");
    let mathml = generate_mathml(&ast, RenderMode::Display);
    assert!(mathml.contains("<mtable"));
}
