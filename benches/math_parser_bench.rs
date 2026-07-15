use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tex2math::{convert, parse, ConvertOptions, ParseOptions, RenderMode};

const NESTED: &str = r#"
    x = a_0 + \cfrac{1}{a_1 + \cfrac{1}{a_2 + \cfrac{1}{a_3 + a_4}}}
"#;

const MATRIX: &str = r#"
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

const SCRIPTS: &str = r#"
    \lim_{x \to \infty} \exp(-x) \int_{0}^{x} \frac{\sin(t)}{t} dt = \prod_{i=1}^{n} \sum_{j=1}^{m} x_{ij}^2
"#;

fn bench_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("tex2math_pipeline");
    let parse_opts = ParseOptions::default();
    let convert_opts = ConvertOptions {
        wrap_math: false,
        mode: RenderMode::Display,
        ..Default::default()
    };

    group.bench_function("parse_only_nested", |b| {
        b.iter(|| {
            let _ = parse(black_box(NESTED), &parse_opts).unwrap();
        })
    });

    group.bench_function("convert_nested", |b| {
        b.iter(|| {
            let _ = convert(black_box(NESTED), &convert_opts).unwrap();
        })
    });

    group.bench_function("convert_matrix", |b| {
        b.iter(|| {
            let _ = convert(black_box(MATRIX), &convert_opts).unwrap();
        })
    });

    group.bench_function("convert_scripts_limits", |b| {
        b.iter(|| {
            let _ = convert(black_box(SCRIPTS), &convert_opts).unwrap();
        })
    });

    // Wide expression: many siblings rather than depth.
    let wide = (0..50)
        .map(|i| format!("x_{{{i}}}"))
        .collect::<Vec<_>>()
        .join("+");
    group.bench_function("convert_wide_sum", |b| {
        b.iter(|| {
            let _ = convert(black_box(wide.as_str()), &convert_opts).unwrap();
        })
    });

    group.finish();
}

criterion_group!(benches, bench_pipeline);
criterion_main!(benches);
