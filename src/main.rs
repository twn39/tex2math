use clap::Parser;
use std::io::{self, Read};
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
        mode: if cli.display {
            RenderMode::Display
        } else {
            RenderMode::Inline
        },
        wrap_math: !cli.no_wrap,
        mathml_core,
        emit_intent: cli.emit_intent,
    };

    match convert(&latex_input, &opts) {
        Ok(out) => println!("{out}"),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
