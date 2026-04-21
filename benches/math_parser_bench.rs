use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tex2math::{generate_mathml, parse_math, RenderMode};
use winnow::Parser;

fn bench_complex_formulas(c: &mut Criterion) {
    let mut group = c.benchmark_group("tex2math_benchmarks");

    // Test 1: High level of nesting (Fractions & Roots)
    let nested_formula = r#"
        x = a_0 + \cfrac{1}{a_1 + \cfrac{1}{a_2 + \cfrac{1}{a_3 + a_4}}}
    "#;
    group.bench_function("nested_fraction", |b| {
        b.iter(|| {
            let mut input = black_box(nested_formula);
            let ast = parse_math.parse_next(&mut input).unwrap();
            let _mathml = generate_mathml(&ast, RenderMode::Display);
        })
    });

    // Test 2: Matrix environment with multi-line alignment
    let matrix_formula = r#"
        \begin{pmatrix}
        \alpha & \beta^{*} \\
        \gamma^{*} & \delta
        \end{pmatrix}
        = \begin{bmatrix}
        1 & 2 & 3 \\
        4 & 5 & 6 \\
        7 & 8 & 9
        \end{bmatrix}
    "#;
    group.bench_function("matrix_environment", |b| {
        b.iter(|| {
            let mut input = black_box(matrix_formula);
            let ast = parse_math.parse_next(&mut input).unwrap();
            let _mathml = generate_mathml(&ast, RenderMode::Display);
        })
    });

    // Test 3: Large operator limits and complex scripts
    let scripts_formula = r#"
        \lim_{x \to \infty} \exp(-x) \int_{0}^{x} \frac{\sin(t)}{t} dt = \prod_{i=1}^{n} \sum_{j=1}^{m} x_{ij}^2
    "#;
    group.bench_function("scripts_and_limits", |b| {
        b.iter(|| {
            let mut input = black_box(scripts_formula);
            let ast = parse_math.parse_next(&mut input).unwrap();
            let _mathml = generate_mathml(&ast, RenderMode::Display);
        })
    });

    group.finish();
}

criterion_group!(benches, bench_complex_formulas);
criterion_main!(benches);
