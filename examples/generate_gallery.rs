//! Generate an interactive visual snapshot gallery HTML for all fixtures in `tests/fixtures/`.
//!
//! Run with:
//! ```bash
//! cargo run --example generate_gallery
//! cargo run --example generate_gallery -- --open
//! ```

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
            mode: RenderMode::Display, // Default to display for gallery visual clarity
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
                    m.mode = if v.eq_ignore_ascii_case("inline") {
                        RenderMode::Inline
                    } else {
                        RenderMode::Display
                    };
                }
                "display" if v == "false" || v == "0" => m.mode = RenderMode::Inline,
                "mathml_core" => m.mathml_core = v == "true" || v == "1",
                "emit_intent" => m.emit_intent = v == "true" || v == "1",
                _ => {}
            }
        }
        m
    }
}

struct FixtureItem {
    suite: String,
    name: String,
    latex: String,
    mathml_rendered: String,
    status: String,
    mode: &'static str,
}

fn strip_comments(text: &str) -> String {
    text.lines()
        .map(|l| {
            if let Some(idx) = l.find('%') {
                &l[..idx]
            } else {
                l
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

fn walk_fixtures(dir: &Path, list: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk_fixtures(&path, list);
            } else if path.extension().and_then(|s| s.to_str()) == Some("tex") {
                list.push(path);
            }
        }
    }
}

fn open_in_browser(path: &Path) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(path).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", path.to_str().unwrap_or("")])
            .spawn();
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
    }
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut tex_files = Vec::new();
    walk_fixtures(&root, &mut tex_files);
    tex_files.sort();

    let mut items = Vec::new();

    for tex_path in &tex_files {
        let rel = tex_path.strip_prefix(&root).unwrap_or(tex_path);
        let suite = rel
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("root")
            .to_string();
        let name = tex_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let raw_tex = fs::read_to_string(tex_path).unwrap_or_default();
        let latex = strip_comments(&raw_tex);

        let meta_path = tex_path.with_extension("meta");
        let meta = if meta_path.exists() {
            let s = fs::read_to_string(&meta_path).unwrap_or_default();
            Meta::parse(&s)
        } else {
            Meta::default()
        };

        let opts = ConvertOptions {
            parse: ParseOptions::default(),
            mode: meta.mode,
            wrap_math: true,
            mathml_core: meta.mathml_core,
            emit_intent: meta.emit_intent,
        };

        let (mathml_rendered, status) = match convert(&latex, &opts) {
            Ok(mathml) => {
                let mathml_file = tex_path.with_extension("mathml");
                let mut status_str = "Converted".to_string();
                if mathml_file.exists() {
                    let expected = fs::read_to_string(&mathml_file).unwrap_or_default();
                    let expected_trimmed = expected.trim();
                    let inner = mathml
                        .trim()
                        .strip_prefix(
                            "<math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"block\">",
                        )
                        .or_else(|| {
                            mathml
                                .trim()
                                .strip_prefix("<math xmlns=\"http://www.w3.org/1998/Math/MathML\">")
                        })
                        .and_then(|s| s.strip_suffix("</math>"))
                        .unwrap_or(&mathml);
                    if inner.trim() == expected_trimmed {
                        status_str = "Exact Match".to_string();
                    }
                }
                (mathml, status_str)
            }
            Err(e) => (
                format!(
                    "<merror><mtext>{}</mtext></merror>",
                    escape_html(&e.message)
                ),
                "Error".to_string(),
            ),
        };

        let mode_name = match meta.mode {
            RenderMode::Display => "Display",
            RenderMode::Inline => "Inline",
        };

        items.push(FixtureItem {
            suite,
            name,
            latex,
            mathml_rendered,
            status,
            mode: mode_name,
        });
    }

    let total = items.len();
    let suites: std::collections::BTreeSet<_> = items.iter().map(|i| i.suite.clone()).collect();

    let mut suites_buttons =
        String::from("<button class=\"filter-btn active\" data-suite=\"all\">All Suites</button>");
    for s in &suites {
        suites_buttons.push_str(&format!(
            "<button class=\"filter-btn\" data-suite=\"{}\">{}</button>",
            escape_html(s),
            escape_html(s)
        ));
    }

    let mut cards_html = String::new();
    for item in &items {
        let escaped_latex = escape_html(&item.latex);
        let escaped_mathml = escape_html(&item.mathml_rendered);
        let status_class = match item.status.as_str() {
            "Exact Match" => "status-exact",
            "Error" => "status-error",
            _ => "status-ok",
        };

        cards_html.push_str(&format!(
            r#"<div class="card" data-suite="{suite}" data-name="{name}" data-latex="{escaped_latex}">
  <div class="card-header">
    <div class="card-title">
      <span class="suite-tag">{suite}</span>
      <strong>{name}</strong>
    </div>
    <div class="card-meta">
      <span class="badge">{mode}</span>
      <span class="status-badge {status_class}">{status}</span>
    </div>
  </div>

  <div class="render-box">
    {mathml_rendered}
  </div>

  <div class="code-section">
    <div class="code-title">LaTeX Source</div>
    <pre><code>{escaped_latex}</code></pre>
  </div>

  <details class="code-details">
    <summary>MathML XML Source</summary>
    <pre><code>{escaped_mathml}</code></pre>
  </details>
</div>"#,
            suite = escape_html(&item.suite),
            name = escape_html(&item.name),
            escaped_latex = escaped_latex,
            mode = item.mode,
            status = item.status,
            status_class = status_class,
            mathml_rendered = item.mathml_rendered,
            escaped_mathml = escaped_mathml,
        ));
    }

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>tex2math Visual Fixtures Gallery</title>
  <style>
    :root {{
      --bg: #090d16;
      --card-bg: #131b2e;
      --border: #23314f;
      --text: #f8fafc;
      --text-muted: #94a3b8;
      --accent: #38bdf8;
      --accent-glow: rgba(56, 189, 248, 0.15);
      --code-bg: #0b1120;
    }}
    * {{ box-sizing: border-box; }}
    body {{
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      background-color: var(--bg);
      color: var(--text);
      margin: 0;
      padding: 2rem 1.5rem;
      min-height: 100vh;
    }}
    .header {{
      max-width: 1200px;
      margin: 0 auto 2rem;
      display: flex;
      flex-direction: column;
      gap: 1.25rem;
    }}
    .header-top {{
      display: flex;
      justify-content: space-between;
      align-items: center;
      flex-wrap: wrap;
      gap: 1rem;
    }}
    h1 {{
      font-size: 1.8rem;
      margin: 0;
      color: var(--accent);
      display: flex;
      align-items: center;
      gap: 0.5rem;
    }}
    h1 span {{ color: var(--text); font-weight: 300; }}
    .stats {{
      color: var(--text-muted);
      font-size: 0.95rem;
    }}
    .controls {{
      display: flex;
      flex-wrap: wrap;
      gap: 1rem;
      align-items: center;
      background: var(--card-bg);
      border: 1px solid var(--border);
      border-radius: 12px;
      padding: 1rem;
    }}
    .search-input {{
      flex: 1;
      min-width: 260px;
      background: var(--code-bg);
      border: 1px solid var(--border);
      border-radius: 8px;
      color: var(--text);
      padding: 0.6rem 1rem;
      font-size: 0.95rem;
      outline: none;
      transition: border-color 0.2s;
    }}
    .search-input:focus {{
      border-color: var(--accent);
    }}
    .filter-group {{
      display: flex;
      gap: 0.5rem;
      flex-wrap: wrap;
    }}
    .filter-btn {{
      background: var(--code-bg);
      border: 1px solid var(--border);
      color: var(--text-muted);
      padding: 0.45rem 0.9rem;
      border-radius: 8px;
      cursor: pointer;
      font-size: 0.85rem;
      font-weight: 500;
      transition: all 0.2s;
    }}
    .filter-btn:hover, .filter-btn.active {{
      background: var(--accent);
      color: #090d16;
      border-color: var(--accent);
      font-weight: 600;
    }}
    .gallery-grid {{
      max-width: 1200px;
      margin: 0 auto;
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(360px, 1fr));
      gap: 1.5rem;
    }}
    .card {{
      background: var(--card-bg);
      border: 1px solid var(--border);
      border-radius: 12px;
      padding: 1.25rem;
      display: flex;
      flex-direction: column;
      gap: 1rem;
      transition: transform 0.2s, box-shadow 0.2s;
    }}
    .card:hover {{
      transform: translateY(-2px);
      box-shadow: 0 8px 20px rgba(0,0,0,0.3);
      border-color: #3b82f6;
    }}
    .card-header {{
      display: flex;
      justify-content: space-between;
      align-items: center;
      border-bottom: 1px solid rgba(255,255,255,0.06);
      padding-bottom: 0.75rem;
    }}
    .card-title {{
      display: flex;
      align-items: center;
      gap: 0.5rem;
      font-size: 1rem;
    }}
    .suite-tag {{
      background: rgba(56, 189, 248, 0.12);
      color: var(--accent);
      font-size: 0.75rem;
      padding: 0.2rem 0.5rem;
      border-radius: 6px;
      font-weight: 600;
    }}
    .card-meta {{
      display: flex;
      gap: 0.4rem;
      align-items: center;
    }}
    .badge {{
      background: rgba(255,255,255,0.08);
      color: var(--text-muted);
      font-size: 0.75rem;
      padding: 0.2rem 0.5rem;
      border-radius: 9999px;
    }}
    .status-badge {{
      font-size: 0.75rem;
      padding: 0.2rem 0.5rem;
      border-radius: 9999px;
      font-weight: 600;
    }}
    .status-exact {{ background: rgba(34, 197, 94, 0.2); color: #4ade80; }}
    .status-ok {{ background: rgba(56, 189, 248, 0.2); color: #38bdf8; }}
    .status-error {{ background: rgba(239, 68, 68, 0.2); color: #f87171; }}
    .render-box {{
      background: radial-gradient(circle at center, #17223b 0%, #0d1424 100%);
      border: 1px solid var(--border);
      border-radius: 8px;
      min-height: 110px;
      display: flex;
      align-items: center;
      justify-content: center;
      padding: 1.5rem;
      font-size: 1.8rem;
      overflow-x: auto;
    }}
    .code-section {{
      display: flex;
      flex-direction: column;
      gap: 0.35rem;
    }}
    .code-title {{
      font-size: 0.75rem;
      text-transform: uppercase;
      letter-spacing: 0.05em;
      color: var(--text-muted);
      font-weight: 600;
    }}
    pre {{
      background: var(--code-bg);
      border: 1px solid var(--border);
      border-radius: 6px;
      padding: 0.75rem;
      margin: 0;
      font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
      font-size: 0.82rem;
      color: #e2e8f0;
      overflow-x: auto;
      white-space: pre-wrap;
      word-break: break-all;
    }}
    details {{
      font-size: 0.82rem;
    }}
    summary {{
      color: var(--text-muted);
      cursor: pointer;
      padding: 0.3rem 0;
      user-select: none;
    }}
    summary:hover {{ color: var(--accent); }}
    math {{
      font-family: "Latin Modern Math", "STIX Two Math", "Cambria Math", serif;
    }}
  </style>
</head>
<body>
  <div class="header">
    <div class="header-top">
      <h1>tex2math <span>Visual Gallery</span></h1>
      <div class="stats">Showing <span id="count">{total}</span> of {total} fixtures</div>
    </div>
    <div class="controls">
      <input type="text" id="search" class="search-input" placeholder="🔍 Search fixtures by name, suite, or LaTeX formula...">
      <div class="filter-group">
        {suites_buttons}
      </div>
    </div>
  </div>

  <div class="gallery-grid" id="gallery">
    {cards_html}
  </div>

  <script>
    const searchInput = document.getElementById('search');
    const filterBtns = document.querySelectorAll('.filter-btn');
    const cards = document.querySelectorAll('.card');
    const countSpan = document.getElementById('count');

    let activeSuite = 'all';
    let searchQuery = '';

    function filterGallery() {{
      let visible = 0;
      cards.forEach(card => {{
        const suite = card.getAttribute('data-suite');
        const name = card.getAttribute('data-name').toLowerCase();
        const latex = card.getAttribute('data-latex').toLowerCase();

        const matchesSuite = (activeSuite === 'all' || suite === activeSuite);
        const matchesSearch = (!searchQuery || name.includes(searchQuery) || latex.includes(searchQuery) || suite.toLowerCase().includes(searchQuery));

        if (matchesSuite && matchesSearch) {{
          card.style.display = 'flex';
          visible++;
        }} else {{
          card.style.display = 'none';
        }}
      }});
      countSpan.textContent = visible;
    }}

    searchInput.addEventListener('input', (e) => {{
      searchQuery = e.target.value.trim().toLowerCase();
      filterGallery();
    }});

    filterBtns.forEach(btn => {{
      btn.addEventListener('click', () => {{
        filterBtns.forEach(b => b.classList.remove('active'));
        btn.classList.add('active');
        activeSuite = btn.getAttribute('data-suite');
        filterGallery();
      }});
    }});
  </script>
</body>
</html>"#,
        total = total,
        suites_buttons = suites_buttons,
        cards_html = cards_html
    );

    let out_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures_gallery.html");
    fs::write(&out_path, html).expect("Failed to write fixtures_gallery.html");
    println!("✅ Visual Gallery generated at: {}", out_path.display());

    let should_open = std::env::args().any(|a| a == "--open" || a == "-o");
    if should_open {
        println!("🚀 Opening in browser...");
        open_in_browser(&out_path);
    }
}
