//! Fixture-driven coverage tests.
//!
//! Layout under `tests/fixtures/`:
//!
//! ```text
//! tests/fixtures/<suite>/<name>.tex      # LaTeX input (no $ delimiters)
//! tests/fixtures/<suite>/<name>.mathml   # exact MathML (convert, wrap_math=false)
//! tests/fixtures/<suite>/<name>.contains # optional: one required substring per line
//! tests/fixtures/<suite>/<name>.meta     # optional: key=value lines
//! ```
//!
//! Supported `.meta` keys:
//! - `mode=display|inline` (default inline)
//! - `mathml_core=true|false` (default true)
//! - `emit_intent=true|false` (default false)
//! - `display=true` (alias for mode=display)
//!
//! A fixture must provide `.mathml` and/or `.contains`. Comments in `.tex`
//! start with `%` at the beginning of a line and are stripped before parse.

use std::fs;
use std::path::{Path, PathBuf};
use tex2math::{convert, ConvertOptions, ParseOptions, RenderMode};

#[derive(Debug)]
struct Meta {
    mode: RenderMode,
    mathml_core: bool,
    emit_intent: bool,
}

impl Default for Meta {
    fn default() -> Self {
        Self {
            mode: RenderMode::Inline,
            mathml_core: true,
            emit_intent: false,
        }
    }
}

impl Meta {
    fn parse(text: &str) -> Self {
        let mut m = Meta::default();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            let k = k.trim();
            let v = v.trim();
            match k {
                "mode" => {
                    m.mode = if v.eq_ignore_ascii_case("display") {
                        RenderMode::Display
                    } else {
                        RenderMode::Inline
                    };
                }
                "display" if v == "true" || v == "1" => m.mode = RenderMode::Display,
                "mathml_core" => m.mathml_core = v == "true" || v == "1",
                "emit_intent" => m.emit_intent = v == "true" || v == "1",
                _ => {}
            }
        }
        m
    }
}

fn strip_tex_comments(src: &str) -> String {
    src.lines()
        .filter(|l| !l.trim_start().starts_with('%'))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn collect_tex_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_tex_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("tex") {
            out.push(path);
        }
    }
}

fn run_one(tex_path: &Path) -> Result<(), String> {
    let stem = tex_path.with_extension("");
    let mathml_path = stem.with_extension("mathml");
    let contains_path = stem.with_extension("contains");
    let meta_path = stem.with_extension("meta");

    let has_mathml = mathml_path.is_file();
    let has_contains = contains_path.is_file();
    if !has_mathml && !has_contains {
        return Err(format!(
            "{}: need {}.mathml and/or {}.contains",
            tex_path.display(),
            stem.file_name().unwrap_or_default().to_string_lossy(),
            stem.file_name().unwrap_or_default().to_string_lossy()
        ));
    }

    let latex_raw =
        fs::read_to_string(tex_path).map_err(|e| format!("read {}: {e}", tex_path.display()))?;
    let latex = strip_tex_comments(&latex_raw);
    if latex.is_empty() {
        return Err(format!(
            "{}: empty after stripping comments",
            tex_path.display()
        ));
    }

    let meta = if meta_path.is_file() {
        Meta::parse(
            &fs::read_to_string(&meta_path)
                .map_err(|e| format!("read {}: {e}", meta_path.display()))?,
        )
    } else {
        Meta::default()
    };

    let opts = ConvertOptions {
        parse: ParseOptions::default(),
        mode: meta.mode,
        wrap_math: false,
        mathml_core: meta.mathml_core,
        emit_intent: meta.emit_intent,
    };

    let got = convert(&latex, &opts)
        .map_err(|e| format!("{}: convert failed for {latex:?}: {e}", tex_path.display()))?;

    if has_mathml {
        let expected = fs::read_to_string(&mathml_path)
            .map_err(|e| format!("read {}: {e}", mathml_path.display()))?
            .trim()
            .to_string();
        if got != expected {
            return Err(format!(
                "{}: MathML mismatch\n  latex:    {latex}\n  expected: {expected}\n  got:      {got}",
                tex_path.display()
            ));
        }
    }

    if has_contains {
        let needles = fs::read_to_string(&contains_path)
            .map_err(|e| format!("read {}: {e}", contains_path.display()))?;
        for needle in needles.lines() {
            let needle = needle.trim();
            if needle.is_empty() || needle.starts_with('#') {
                continue;
            }
            if !got.contains(needle) {
                return Err(format!(
                    "{}: missing substring {needle:?}\n  latex: {latex}\n  got:   {got}",
                    tex_path.display()
                ));
            }
        }
    }

    Ok(())
}

#[test]
fn all_fixtures() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    assert!(
        root.is_dir(),
        "fixtures directory missing: {}",
        root.display()
    );

    let mut files = Vec::new();
    collect_tex_files(&root, &mut files);
    files.sort();
    assert!(
        !files.is_empty(),
        "no .tex fixtures under {}",
        root.display()
    );

    let mut failures = Vec::new();
    for path in &files {
        if let Err(e) = run_one(path) {
            failures.push(e);
        }
    }

    if !failures.is_empty() {
        panic!(
            "{} fixture(s) failed:\n\n{}",
            failures.len(),
            failures.join("\n\n")
        );
    }
}
