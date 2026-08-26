use tex2math::{convert, ConvertOptions};

#[test]
fn test_dirac_bra_ket() {
    let opts = ConvertOptions {
        wrap_math: false,
        ..Default::default()
    };

    // \bra{\psi} -> <mrow><mo>⟨</mo><mi>ψ</mi><mo>|</mo></mrow>
    let bra = convert(r"\bra{\psi}", &opts).unwrap();
    assert!(
        bra.contains("&#x27E8;") || bra.contains("\u{27E8}"),
        "got: {bra}"
    );
    assert!(bra.contains("|"), "got: {bra}");

    // \ket{\phi} -> <mrow><mo>|</mo><mi>ϕ</mi><mo>⟩</mo></mrow>
    let ket = convert(r"\ket{\phi}", &opts).unwrap();
    assert!(ket.contains("|"), "got: {ket}");
    assert!(
        ket.contains("&#x27E9;") || ket.contains("\u{27E9}"),
        "got: {ket}"
    );

    // \braket{\phi|\psi}
    let braket1 = convert(r"\braket{\phi|\psi}", &opts).unwrap();
    assert!(braket1.contains("\u{27E8}") || braket1.contains("&#x27E8;"));
    assert!(braket1.contains("\u{27E9}") || braket1.contains("&#x27E9;"));
    assert!(braket1.contains("|"));

    // \braket{\phi}{\psi} (two arguments form)
    let braket2 = convert(r"\braket{\phi}{\psi}", &opts).unwrap();
    assert!(braket2.contains("|"));

    // \ketbra{\psi}{\phi}
    let ketbra = convert(r"\ketbra{\psi}{\phi}", &opts).unwrap();
    assert!(ketbra.contains("\u{27E8}") || ketbra.contains("&#x27E8;"));
    assert!(ketbra.contains("\u{27E9}") || ketbra.contains("&#x27E9;"));

    // \mel{a}{O}{b}
    let mel = convert(r"\mel{a}{O}{b}", &opts).unwrap();
    assert!(mel.contains("<mi>a</mi>"));
    assert!(mel.contains("<mi>O</mi>"));
    assert!(mel.contains("<mi>b</mi>"));

    // \ev{A}
    let ev = convert(r"\ev{A}", &opts).unwrap();
    assert!(ev.contains("<mi>A</mi>"));
}

#[test]
fn test_physics_derivatives() {
    let opts = ConvertOptions {
        wrap_math: false,
        ..Default::default()
    };

    // \dd{x} -> <mrow><mstyle mathvariant="normal"><mi>d</mi></mstyle><mi>x</mi></mrow>
    let dd = convert(r"\dd{x}", &opts).unwrap();
    assert!(dd.contains("mathvariant=\"normal\""));
    assert!(dd.contains("<mi>d</mi>"));
    assert!(dd.contains("<mi>x</mi>"));

    // \dd[3]{x}
    let dd3 = convert(r"\dd[3]{x}", &opts).unwrap();
    assert!(dd3.contains("<msup>"));
    assert!(dd3.contains("<mn>3</mn>"));

    // \dv{f}{x}
    let dv = convert(r"\dv{f}{x}", &opts).unwrap();
    assert!(dv.contains("<mfrac>"));
    assert!(dv.contains("<mi>f</mi>"));
    assert!(dv.contains("<mi>x</mi>"));

    // \dv[2]{f}{x}
    let dv2 = convert(r"\dv[2]{f}{x}", &opts).unwrap();
    assert!(dv2.contains("<mfrac>"));
    assert!(dv2.contains("<mn>2</mn>"));

    // \dv{x} -> d/dx
    let dv_x = convert(r"\dv{x}", &opts).unwrap();
    assert!(dv_x.contains("<mfrac>"));
    assert!(dv_x.contains("<mi>x</mi>"));

    // \pdv{f}{x}
    let pdv = convert(r"\pdv{f}{x}", &opts).unwrap();
    assert!(pdv.contains("<mfrac>"));
    assert!(pdv.contains("&#x2202;") || pdv.contains("\u{2202}") || pdv.contains("∂"));

    // \pdv{f}{x}{y} (mixed second partial)
    let pdv_xy = convert(r"\pdv{f}{x}{y}", &opts).unwrap();
    assert!(pdv_xy.contains("<mfrac>"));
    assert!(pdv_xy.contains("<mn>2</mn>"));
    assert!(pdv_xy.contains("<mi>x</mi>"));
    assert!(pdv_xy.contains("<mi>y</mi>"));
}

#[test]
fn test_vector_operators_and_units() {
    let opts = ConvertOptions {
        wrap_math: false,
        ..Default::default()
    };

    // \grad -> ∇
    let grad = convert(r"\grad \psi", &opts).unwrap();
    assert!(grad.contains("&#x2207;") || grad.contains("\u{2207}") || grad.contains("∇"));

    // \div -> ∇ ⋅
    let div = convert(r"\div \mathbf{E}", &opts).unwrap();
    assert!(div.contains("&#x22C5;") || div.contains("\u{22C5}") || div.contains("⋅"));

    // \curl -> ∇ ×
    let curl = convert(r"\curl \mathbf{B}", &opts).unwrap();
    assert!(curl.contains("&#x00D7;") || curl.contains("\u{00D7}") || curl.contains("×"));

    // \laplacian -> ∇²
    let lap = convert(r"\laplacian \Phi", &opts).unwrap();
    assert!(lap.contains("<msup>"));
    assert!(lap.contains("<mn>2</mn>"));

    // \unit{kg}
    let unit = convert(r"\unit{kg}", &opts).unwrap();
    assert!(unit.contains("mathvariant=\"normal\""));
}
