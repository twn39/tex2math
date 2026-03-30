use crate::MathNode;

/// 将 LaTeX 命令名映射为具体的 MathNode (Identifier 或 Operator)
/// 包含了数百个常用的希腊字母、箭头、数学符号等。
pub fn lookup_symbol(cmd: &str) -> Option<MathNode> {
    match cmd {
        // ==========================================
        // 1. 希腊字母 (小写) - 全部作为 Identifier
        // ==========================================
        "alpha" => Some(MathNode::Identifier("α".to_string())),
        "beta" => Some(MathNode::Identifier("β".to_string())),
        "gamma" => Some(MathNode::Identifier("γ".to_string())),
        "delta" => Some(MathNode::Identifier("δ".to_string())),
        "epsilon" => Some(MathNode::Identifier("ϵ".to_string())),
        "varepsilon" => Some(MathNode::Identifier("ε".to_string())),
        "zeta" => Some(MathNode::Identifier("ζ".to_string())),
        "eta" => Some(MathNode::Identifier("η".to_string())),
        "theta" => Some(MathNode::Identifier("θ".to_string())),
        "vartheta" => Some(MathNode::Identifier("ϑ".to_string())),
        "iota" => Some(MathNode::Identifier("ι".to_string())),
        "kappa" => Some(MathNode::Identifier("κ".to_string())),
        "varkappa" => Some(MathNode::Identifier("ϰ".to_string())),
        "lambda" => Some(MathNode::Identifier("λ".to_string())),
        "mu" => Some(MathNode::Identifier("μ".to_string())),
        "nu" => Some(MathNode::Identifier("ν".to_string())),
        "xi" => Some(MathNode::Identifier("ξ".to_string())),
        "pi" => Some(MathNode::Identifier("π".to_string())),
        "varpi" => Some(MathNode::Identifier("ϖ".to_string())),
        "rho" => Some(MathNode::Identifier("ρ".to_string())),
        "varrho" => Some(MathNode::Identifier("ϱ".to_string())),
        "sigma" => Some(MathNode::Identifier("σ".to_string())),
        "varsigma" => Some(MathNode::Identifier("ς".to_string())),
        "tau" => Some(MathNode::Identifier("τ".to_string())),
        "upsilon" => Some(MathNode::Identifier("υ".to_string())),
        "phi" => Some(MathNode::Identifier("ϕ".to_string())),
        "varphi" => Some(MathNode::Identifier("φ".to_string())),
        "chi" => Some(MathNode::Identifier("χ".to_string())),
        "psi" => Some(MathNode::Identifier("ψ".to_string())),
        "omega" => Some(MathNode::Identifier("ω".to_string())),

        // ==========================================
        // 2. 希腊字母 (大写) - 全部作为 Identifier
        // ==========================================
        "Gamma" => Some(MathNode::Identifier("Γ".to_string())),
        "Delta" => Some(MathNode::Identifier("Δ".to_string())),
        "Theta" => Some(MathNode::Identifier("Θ".to_string())),
        "Lambda" => Some(MathNode::Identifier("Λ".to_string())),
        "Xi" => Some(MathNode::Identifier("Ξ".to_string())),
        "Pi" => Some(MathNode::Identifier("Π".to_string())),
        "Sigma" => Some(MathNode::Identifier("Σ".to_string())),
        "Upsilon" => Some(MathNode::Identifier("Υ".to_string())),
        "Phi" => Some(MathNode::Identifier("Φ".to_string())),
        "Psi" => Some(MathNode::Identifier("Ψ".to_string())),
        "Omega" => Some(MathNode::Identifier("Ω".to_string())),

        // ==========================================
        // 3. 特殊常数与普通符号 - 作为 Identifier
        // ==========================================
        "infty" => Some(MathNode::Identifier("∞".to_string())),
        "partial" => Some(MathNode::Identifier("∂".to_string())),
        "nabla" => Some(MathNode::Identifier("∇".to_string())),
        "emptyset" => Some(MathNode::Identifier("∅".to_string())),
        "varnothing" => Some(MathNode::Identifier("∅".to_string())),
        "Re" => Some(MathNode::Identifier("ℜ".to_string())),
        "Im" => Some(MathNode::Identifier("ℑ".to_string())),
        "aleph" => Some(MathNode::Identifier("ℵ".to_string())),
        "ell" => Some(MathNode::Identifier("ℓ".to_string())),
        "wp" => Some(MathNode::Identifier("℘".to_string())),
        "hbar" => Some(MathNode::Identifier("ℏ".to_string())),
        "angle" => Some(MathNode::Identifier("∠".to_string())),
        "triangle" => Some(MathNode::Identifier("△".to_string())),
        "bot" => Some(MathNode::Identifier("⊥".to_string())),
        "top" => Some(MathNode::Identifier("⊤".to_string())),

        // ==========================================
        // 4. 二元操作符 (Binary Operators) - 作为 Operator
        // ==========================================
        "pm" => Some(MathNode::Operator("±".to_string())),
        "mp" => Some(MathNode::Operator("∓".to_string())),
        "times" => Some(MathNode::Operator("×".to_string())),
        "div" => Some(MathNode::Operator("÷".to_string())),
        "cdot" => Some(MathNode::Operator("·".to_string())),
        "ast" => Some(MathNode::Operator("*".to_string())),
        "star" => Some(MathNode::Operator("⋆".to_string())),
        "circ" => Some(MathNode::Operator("∘".to_string())),
        "bullet" => Some(MathNode::Operator("∙".to_string())),
        "oplus" => Some(MathNode::Operator("⊕".to_string())),
        "ominus" => Some(MathNode::Operator("⊖".to_string())),
        "otimes" => Some(MathNode::Operator("⊗".to_string())),
        "oslash" => Some(MathNode::Operator("⊘".to_string())),
        "odot" => Some(MathNode::Operator("⊙".to_string())),
        "setminus" => Some(MathNode::Operator("∖".to_string())),
        "uplus" => Some(MathNode::Operator("⊎".to_string())),
        "sqcap" => Some(MathNode::Operator("⊓".to_string())),
        "sqcup" => Some(MathNode::Operator("⊔".to_string())),
        "vee" => Some(MathNode::Operator("∨".to_string())),
        "wedge" => Some(MathNode::Operator("∧".to_string())),
        "amalg" => Some(MathNode::Operator("⨿".to_string())),

        // ==========================================
        // 5. 关系操作符 (Relational Operators)
        // ==========================================
        "leq" | "le" => Some(MathNode::Operator("≤".to_string())),
        "geq" | "ge" => Some(MathNode::Operator("≥".to_string())),
        "neq" | "ne" => Some(MathNode::Operator("≠".to_string())),
        "equiv" => Some(MathNode::Operator("≡".to_string())),
        "sim" => Some(MathNode::Operator("∼".to_string())),
        "simeq" => Some(MathNode::Operator("≃".to_string())),
        "approx" => Some(MathNode::Operator("≈".to_string())),
        "cong" => Some(MathNode::Operator("≅".to_string())),
        "propto" => Some(MathNode::Operator("∝".to_string())),
        "ll" => Some(MathNode::Operator("≪".to_string())),
        "gg" => Some(MathNode::Operator("≫".to_string())),
        "asymp" => Some(MathNode::Operator("≍".to_string())),
        "doteq" => Some(MathNode::Operator("≐".to_string())),
        "models" => Some(MathNode::Operator("⊨".to_string())),

        // ==========================================
        // 6. 集合逻辑操作符
        // ==========================================
        "in" => Some(MathNode::Operator("∈".to_string())),
        "notin" => Some(MathNode::Operator("∉".to_string())),
        "ni" => Some(MathNode::Operator("∋".to_string())),
        "subset" => Some(MathNode::Operator("⊂".to_string())),
        "supset" => Some(MathNode::Operator("⊃".to_string())),
        "subseteq" => Some(MathNode::Operator("⊆".to_string())),
        "supseteq" => Some(MathNode::Operator("⊇".to_string())),
        "cup" => Some(MathNode::Operator("∪".to_string())),
        "cap" => Some(MathNode::Operator("∩".to_string())),
        "forall" => Some(MathNode::Operator("∀".to_string())),
        "exists" => Some(MathNode::Operator("∃".to_string())),
        "nexists" => Some(MathNode::Operator("∄".to_string())),
        "neg" | "lnot" => Some(MathNode::Operator("¬".to_string())),

        // ==========================================
        // 7. 箭头 (Arrows)
        // ==========================================
        "leftarrow" | "gets" => Some(MathNode::Operator("←".to_string())),
        "rightarrow" | "to" => Some(MathNode::Operator("→".to_string())),
        "leftrightarrow" => Some(MathNode::Operator("↔".to_string())),
        "Leftarrow" => Some(MathNode::Operator("⇐".to_string())),
        "Rightarrow" => Some(MathNode::Operator("⇒".to_string())),
        "Leftrightarrow" => Some(MathNode::Operator("⇔".to_string())),
        "mapsto" => Some(MathNode::Operator("↦".to_string())),
        "uparrow" => Some(MathNode::Operator("↑".to_string())),
        "downarrow" => Some(MathNode::Operator("↓".to_string())),
        "updownarrow" => Some(MathNode::Operator("↕".to_string())),
        "Uparrow" => Some(MathNode::Operator("⇑".to_string())),
        "Downarrow" => Some(MathNode::Operator("⇓".to_string())),
        "Updownarrow" => Some(MathNode::Operator("⇕".to_string())),
        "nearrow" => Some(MathNode::Operator("↗".to_string())),
        "searrow" => Some(MathNode::Operator("↘".to_string())),
        "swarrow" => Some(MathNode::Operator("↙".to_string())),
        "nwarrow" => Some(MathNode::Operator("↖".to_string())),
        "iff" => Some(MathNode::Operator("⟺".to_string())),
        "implies" => Some(MathNode::Operator("⟹".to_string())),

        // ==========================================
        // 8. 大运算符 (Large Operators)
        // ==========================================
        "sum" => Some(MathNode::Operator("∑".to_string())),
        "prod" => Some(MathNode::Operator("∏".to_string())),
        "coprod" => Some(MathNode::Operator("∐".to_string())),
        "int" => Some(MathNode::Operator("∫".to_string())),
        "iint" => Some(MathNode::Operator("∬".to_string())),
        "iiint" => Some(MathNode::Operator("∭".to_string())),
        "oint" => Some(MathNode::Operator("∮".to_string())),
        "bigcap" => Some(MathNode::Operator("⋂".to_string())),
        "bigcup" => Some(MathNode::Operator("⋃".to_string())),
        "bigsqcup" => Some(MathNode::Operator("⨆".to_string())),
        "bigvee" => Some(MathNode::Operator("⋁".to_string())),
        "bigwedge" => Some(MathNode::Operator("⋀".to_string())),
        "bigodot" => Some(MathNode::Operator("⨀".to_string())),
        "bigoplus" => Some(MathNode::Operator("⨁".to_string())),
        "bigotimes" => Some(MathNode::Operator("⨂".to_string())),

        _ => None, // 没匹配到
    }
}