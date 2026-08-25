use clap::Parser;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use tex2math::{
    convert, ConvertOptions, ParseOptions, RenderMode, TrailingPolicy, UnknownCommandPolicy,
};

/// tex2math 2.x: LaTeX → MathML converter
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The LaTeX string to convert. If omitted, read from stdin.
    #[arg(value_name = "INPUT")]
    input: Option<String>,

    /// Display mode (`display="block"`, large-op limits above/below).
    #[arg(short, long, default_value_t = false)]
    display: bool,

    /// Do not wrap output in `<math>` tags.
    #[arg(long, default_value_t = false)]
    no_wrap: bool,

    /// Output a standalone HTML page displaying the rendered MathML.
    #[arg(long, default_value_t = false)]
    html: bool,

    /// Generate an HTML preview and open it directly in the default browser.
    #[arg(short, long, default_value_t = false)]
    preview: bool,

    /// Maximum parse nesting depth (default 64).
    #[arg(long, default_value_t = 64)]
    max_depth: u32,

    /// Ignore trailing unparsed input instead of erroring.
    #[arg(long, default_value_t = false)]
    allow_trailing: bool,

    /// Treat unknown `\`-commands as errors (`<merror>`) instead of identifiers.
    #[arg(long, default_value_t = false)]
    unknown_error: bool,

    /// Use legacy non-Core constructs (e.g. `<menclose>` for `\cancel`/`\boxed`).
    /// Default is MathML Core–friendly emission.
    #[arg(long, default_value_t = false)]
    no_mathml_core: bool,

    /// Emit experimental MathML 4 `intent` attributes.
    #[arg(long, default_value_t = false)]
    emit_intent: bool,
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

fn render_html_document(latex: &str, mathml: &str, opts: &ConvertOptions) -> String {
    let mode_str = match opts.mode {
        RenderMode::Display => "Display",
        RenderMode::Inline => "Inline",
    };
    let core_str = if opts.mathml_core {
        "MathML Core"
    } else {
        "Full MathML"
    };
    let intent_str = if opts.emit_intent {
        "Intent Enabled"
    } else {
        "Intent Disabled"
    };
    let escaped_latex = escape_html(latex);
    let escaped_mathml = escape_html(mathml);

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>tex2math Preview</title>
  <style>
    :root {{
      --bg: #0f172a;
      --card-bg: #1e293b;
      --text: #f8fafc;
      --text-muted: #94a3b8;
      --accent: #38bdf8;
      --border: #334155;
      --code-bg: #090d16;
    }}
    body {{
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      background-color: var(--bg);
      color: var(--text);
      margin: 0;
      padding: 2.5rem 1rem;
      display: flex;
      flex-direction: column;
      align-items: center;
      min-height: 100vh;
      box-sizing: border-box;
    }}
    .container {{
      max-width: 820px;
      width: 100%;
      display: flex;
      flex-direction: column;
      gap: 1.5rem;
    }}
    header {{
      display: flex;
      justify-content: space-between;
      align-items: center;
      border-bottom: 1px solid var(--border);
      padding-bottom: 1rem;
    }}
    h1 {{
      font-size: 1.4rem;
      margin: 0;
      color: var(--accent);
      display: flex;
      align-items: center;
      gap: 0.5rem;
    }}
    h1 span {{
      color: var(--text);
      font-weight: 400;
    }}
    .badges {{
      display: flex;
      gap: 0.5rem;
    }}
    .badge {{
      background: #334155;
      color: #cbd5e1;
      font-size: 0.75rem;
      padding: 0.25rem 0.6rem;
      border-radius: 9999px;
      font-weight: 500;
    }}
    .card {{
      background: var(--card-bg);
      border: 1px solid var(--border);
      border-radius: 12px;
      padding: 1.75rem;
      box-shadow: 0 4px 6px -1px rgba(0,0,0,0.2), 0 2px 4px -2px rgba(0,0,0,0.2);
    }}
    .preview-card {{
      display: flex;
      justify-content: center;
      align-items: center;
      min-height: 160px;
      font-size: 2.4rem;
      overflow-x: auto;
      padding: 2.5rem 1.5rem;
      background: radial-gradient(circle at center, #1e293b 0%, #0f172a 100%);
    }}
    .card-title {{
      font-size: 0.82rem;
      text-transform: uppercase;
      letter-spacing: 0.06em;
      color: var(--text-muted);
      margin-bottom: 0.75rem;
      font-weight: 600;
    }}
    pre {{
      background: var(--code-bg);
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 1rem;
      overflow-x: auto;
      font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
      font-size: 0.88rem;
      margin: 0;
      color: #e2e8f0;
      white-space: pre-wrap;
      word-break: break-all;
      line-height: 1.5;
    }}
    math {{
      font-family: "Latin Modern Math", "STIX Two Math", "Cambria Math", serif;
    }}
  </style>
</head>
<body>
  <div class="container">
    <header>
      <h1>tex2math <span>Preview</span></h1>
      <div class="badges">
        <span class="badge">{mode_str}</span>
        <span class="badge">{core_str}</span>
        <span class="badge">{intent_str}</span>
      </div>
    </header>

    <div class="card preview-card">
      {mathml}
    </div>

    <div class="card">
      <div class="card-title">LaTeX Source</div>
      <pre>{escaped_latex}</pre>
    </div>

    <div class="card">
      <div class="card-title">Generated MathML</div>
      <pre>{escaped_mathml}</pre>
    </div>
  </div>
</body>
</html>"#
    )
}

fn open_in_browser(path: &Path) -> io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(path).spawn()?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", path.to_str().unwrap_or("")])
            .spawn()?;
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        std::process::Command::new("xdg-open").arg(path).spawn()?;
    }
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    let latex_input = match cli.input {
        Some(s) => s,
        None => {
            let mut buffer = String::new();
            if io::stdin().read_to_string(&mut buffer).is_err() {
                eprintln!("Error: Failed to read from stdin.");
                std::process::exit(1);
            }
            if buffer.trim().is_empty() {
                eprintln!("Error: No input provided.");
                std::process::exit(1);
            }
            buffer
        }
    };

    let mathml_core = !cli.no_mathml_core;

    let opts = ConvertOptions {
        parse: ParseOptions {
            max_depth: cli.max_depth,
            trailing: if cli.allow_trailing {
                TrailingPolicy::Ignore
            } else {
                TrailingPolicy::Error
            },
            unknown_command: if cli.unknown_error {
                UnknownCommandPolicy::Error
            } else {
                UnknownCommandPolicy::Identifier
            },
            ..ParseOptions::default()
        },
        mode: if cli.display || cli.html || cli.preview {
            RenderMode::Display
        } else {
            RenderMode::Inline
        },
        wrap_math: !cli.no_wrap || cli.html || cli.preview,
        mathml_core,
        emit_intent: cli.emit_intent,
    };

    match convert(&latex_input, &opts) {
        Ok(out) => {
            if cli.preview {
                let html_content = render_html_document(&latex_input, &out, &opts);
                let temp_path = std::env::temp_dir().join("tex2math_preview.html");
                if let Err(e) = fs::write(&temp_path, html_content) {
                    eprintln!("Error writing preview HTML file: {e}");
                    std::process::exit(1);
                }
                println!("Saved preview to {}", temp_path.display());
                if let Err(e) = open_in_browser(&temp_path) {
                    eprintln!("Error launching browser: {e}");
                }
            } else if cli.html {
                let html_content = render_html_document(&latex_input, &out, &opts);
                println!("{html_content}");
            } else {
                println!("{out}");
            }
        }
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
