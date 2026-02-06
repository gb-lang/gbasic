use clap::Parser as ClapParser;
use colored::Colorize;
use std::fs;
use std::process;

#[derive(ClapParser)]
#[command(name = "gbasic")]
#[command(version)]
#[command(about = "The G-Basic programming language compiler", long_about = None)]
struct Cli {
    /// Source file to compile (.gb)
    file: Option<String>,

    /// Print the AST instead of compiling
    #[arg(long)]
    dump_ast: bool,

    /// Print tokens instead of compiling
    #[arg(long)]
    dump_tokens: bool,
}

fn main() {
    let cli = Cli::parse();

    let Some(file) = cli.file else {
        return;
    };

    let source = match fs::read_to_string(&file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}: {}: {}", "error".red().bold(), file, e);
            process::exit(1);
        }
    };

    if cli.dump_tokens {
        let tokens = gbasic_lexer::tokenize(&source);
        for tok in &tokens {
            println!("{:?} @ {}..{}", tok.token, tok.span.start, tok.span.end);
        }
        return;
    }

    let program = match gbasic_parser::parse(&source) {
        Ok(p) => p,
        Err(errors) => {
            for err in &errors {
                eprintln!("{}: {}", "error".red().bold(), err);
            }
            process::exit(1);
        }
    };

    if cli.dump_ast {
        println!("{:#?}", program);
        return;
    }

    println!(
        "{}: compiled {} ({} statements)",
        "ok".green().bold(),
        file,
        program.statements.len()
    );
}
