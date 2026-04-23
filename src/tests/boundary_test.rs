use crate::parser::parse_math;
use crate::renderer::mathml::MathMLRenderer;
use crate::renderer::MathRenderer;
use crate::ast::RenderMode;
use std::time::Instant;

// ==========================================
// 1. 超长平铺序列测试 (O(N) 复杂度验证)
// ==========================================
#[test]
fn test_boundary_ultra_long_sequence() {
    let mut input = String::with_capacity(30000);
    for _ in 0..10000 {
        input.push_str("a + ");
    }
    input.push('b');

    let start = Instant::now();
    let mut i = input.as_str();
    let ast = parse_math(&mut i).unwrap();
    let duration = start.elapsed();

    // 必须能够迅速完成，绝不能指数级退化
    assert!(duration.as_millis() < 500, "Parsing took too long: {} ms", duration.as_millis());
    
    // 渲染结果应包含超过两万个节点
    let renderer = MathMLRenderer::new();
    let mathml = renderer.render(&ast, RenderMode::Inline);
    assert!(mathml.len() > 100_000);
}

// ==========================================
// 2. 超深层嵌套测试 (Stack Overflow 预防)
// ==========================================
// 暂时忽略，避免导致整个测试套件因 SIGABRT 直接崩溃，待实现保护机制后再启用
#[test]
#[ignore]
fn test_boundary_ultra_deep_nesting() {
    let mut input = String::new();
    let depth = 5000;
    for _ in 0..depth {
        input.push_str("\\frac{1}{");
    }
    input.push('2');
    for _ in 0..depth {
        input.push('}');
    }

    let mut i = input.as_str();
    let res = parse_math(&mut i);
    
    // 我们期望的是一个优雅的 Error (例如 Recursion Limit Exceeded)，而不是内核段错误
    assert!(res.is_err(), "Deep nesting should eventually hit a recursion limit gracefully.");
}

// ==========================================
// 3. 全量 Unicode 与 Emoji 边界安全
// ==========================================
#[test]
fn test_boundary_full_unicode_text() {
    let mut input = "\\text{你好，世界！🙋‍♂️ 123 α} + \\sum_{i=1}^n x_i";
    let ast = parse_math(&mut input).expect("Should parse full unicode successfully");
    
    let renderer = MathMLRenderer::new();
    let mathml = renderer.render(&ast, RenderMode::Display);

    assert!(mathml.contains("<mtext>你好，世界！🙋‍♂️ 123 α</mtext>"));
    assert!(mathml.contains("<mo>∑</mo>"));
}

// ==========================================
// 4. 极端小数与科学计数法格式测试
// ==========================================
#[test]
fn test_boundary_decimal_formats() {
    // KaTeX 会将 `.14` 和 `10.` 当作一个单一的 <mn> 处理，而不是 <mo>.</mo><mn>14</mn>
    let formats = vec![
        (".14", "<mn>.14</mn>"),
        ("10.", "<mn>10.</mn>"),
        ("0.5", "<mn>0.5</mn>"),
        ("42", "<mn>42</mn>"),
    ];

    let renderer = MathMLRenderer::new();

    for (math, expected) in formats {
        let mut input = math;
        let ast = parse_math(&mut input).unwrap();
        let mathml = renderer.render(&ast, RenderMode::Inline);
        assert!(mathml.contains(expected), "Failed on input '{}', got MathML: {}", math, mathml);
    }
    
    // 单独的 . 仍然应该被解析为操作符，而不是数字
    let mut input_dot = ".";
    let ast_dot = parse_math(&mut input_dot).unwrap();
    let mathml_dot = renderer.render(&ast_dot, RenderMode::Inline);
    assert!(mathml_dot.contains("<mo>.</mo>"));
}
