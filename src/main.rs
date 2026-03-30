use clap::Parser;
use std::io::{self, Read};
use tex2math::{generate_mathml, parse_row, RenderMode};
use winnow::Parser as WinnowParser; // 需要引入 traits 才能调用 parse_next

/// tex2math: A blazing fast, zero-copy LaTeX to MathML converter
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The LaTeX string to convert. If not provided, it will read from standard input (stdin).
    #[arg(value_name = "INPUT")]
    input: Option<String>,

    /// Render the math formula in display mode (forces <math display="block"> and large operators to use over/under limits).
    #[arg(short, long, default_value_t = false)]
    display: bool,
    
    /// Do not wrap the output in <math> tags, only return the inner MathML nodes.
    #[arg(long, default_value_t = false)]
    no_wrap: bool,
}

fn main() {
    // 1. 解析命令行参数
    let cli = Cli::parse();

    // 2. 获取 LaTeX 输入 (优先使用参数，否则读标准输入)
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

    // 3. 决定渲染模式
    let mode = if cli.display {
        RenderMode::Display
    } else {
        RenderMode::Inline
    };

    // 4. 执行解析
    // 由于 winnow 的解析器要求一个可变引用（游标）
    let mut cursor = latex_input.as_str();
    
    match parse_row.parse_next(&mut cursor) {
        Ok(ast) => {
            // 5. 生成 MathML
            let mathml = generate_mathml(&ast, mode);
            
            // 6. 输出结果
            if cli.no_wrap {
                println!("{}", mathml);
            } else {
                let display_attr = if cli.display { " display=\"block\"" } else { "" };
                println!("<math xmlns=\"http://www.w3.org/1998/Math/MathML\"{}>{}</math>", display_attr, mathml);
            }
            
            // 检查是否有未解析完的垃圾尾缀
            if !cursor.trim().is_empty() {
                eprintln!("Warning: Some trailing characters were not parsed: '{}'", cursor);
            }
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    }
}
