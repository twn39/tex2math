use tex2math::{convert, ConvertOptions};

#[test]
fn test_chemistry_simple_formulas() {
    let opts = ConvertOptions {
        wrap_math: false,
        ..Default::default()
    };

    // \ce{H2O}
    let h2o = convert(r"\ce{H2O}", &opts).unwrap();
    assert!(h2o.contains("mathvariant=\"normal\""));
    assert!(h2o.contains("<mi>H</mi>"));
    assert!(h2o.contains("<msub>"));
    assert!(h2o.contains("<mn>2</mn>"));
    assert!(h2o.contains("<mi>O</mi>"));

    // \ce{H2SO4}
    let h2so4 = convert(r"\ce{H2SO4}", &opts).unwrap();
    assert!(h2so4.contains("<mi>S</mi>"));
    assert!(h2so4.contains("<mn>4</mn>"));

    // \ce{Ca(OH)2}
    let ca_oh_2 = convert(r"\ce{Ca(OH)2}", &opts).unwrap();
    assert!(ca_oh_2.contains("<mi>Ca</mi>"));
    assert!(ca_oh_2.contains("(") && ca_oh_2.contains(")"));
    assert!(ca_oh_2.contains("<mn>2</mn>"));
}

#[test]
fn test_chemistry_reaction_equations() {
    let opts = ConvertOptions {
        wrap_math: false,
        ..Default::default()
    };

    // \ce{2H2 + O2 -> 2H2O}
    let rxn = convert(r"\ce{2H2 + O2 -> 2H2O}", &opts).unwrap();
    assert!(rxn.contains("<mn>2</mn>"));
    assert!(rxn.contains("<mo>+</mo>"));
    assert!(rxn.contains("&#x2192;") || rxn.contains("\u{2192}") || rxn.contains("→"));

    // \ce{N2 + 3H2 <=> 2NH3} (equilibrium arrow)
    let eq = convert(r"\ce{N2 + 3H2 <=> 2NH3}", &opts).unwrap();
    assert!(eq.contains("&#x21CC;") || eq.contains("\u{21CC}") || eq.contains("⇌"));

    // Reaction arrow with annotations: \ce{A ->[catalyst][\Delta] B}
    let arrow_ann = convert(r"\ce{A ->[catalyst][\Delta] B}", &opts).unwrap();
    assert!(arrow_ann.contains("<munderover") || arrow_ann.contains("<mover"));
}

#[test]
fn test_chemistry_ions_and_isotopes() {
    let opts = ConvertOptions {
        wrap_math: false,
        ..Default::default()
    };

    // \ce{Fe^{3+} + 3OH- -> Fe(OH)3 v}
    let ppt = convert(r"\ce{Fe^{3+} + 3OH- -> Fe(OH)3 v}", &opts).unwrap();
    assert!(ppt.contains("<msup>") || ppt.contains("<msubsup>"));
    assert!(ppt.contains("&#x2193;") || ppt.contains("\u{2193}") || ppt.contains("↓")); // precipitate

    // Isotope: \ce{^{238}_{92}U -> ^{234}_{90}Th + \alpha}
    let nuc = convert(r"\ce{^{238}_{92}U -> ^{234}_{90}Th + \alpha}", &opts).unwrap();
    assert!(nuc.contains("<mmultiscripts>"));
    assert!(nuc.contains("<mprescripts/>"));
    assert!(nuc.contains("<mn>238</mn>"));
    assert!(nuc.contains("<mn>92</mn>"));
}

#[test]
fn test_chemistry_hydrates_and_bonds() {
    let opts = ConvertOptions {
        wrap_math: false,
        ..Default::default()
    };

    // \ce{CuSO4 . 5H2O}
    let hydrate = convert(r"\ce{CuSO4 . 5H2O}", &opts).unwrap();
    assert!(hydrate.contains("&#x22C5;") || hydrate.contains("\u{22C5}") || hydrate.contains("⋅"));

    // \ce{H-C#C-H}
    let alkyne = convert(r"\ce{H-C#C-H}", &opts).unwrap();
    assert!(alkyne.contains("&#x2261;") || alkyne.contains("\u{2261}") || alkyne.contains("≡"));

    // \ce{K_{$x$}Fe_{2-$x$}S2} (nested math mode)
    let nested = convert(r"\ce{K_{$x$}Fe_{2-$x$}S2}", &opts).unwrap();
    assert!(nested.contains("<mi>K</mi>"));
    assert!(nested.contains("<mi>x</mi>"));
}
