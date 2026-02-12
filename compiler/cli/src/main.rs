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

    /// Print LLVM IR instead of compiling
    #[arg(long)]
    dump_ir: bool,

    /// Typecheck only (no codegen)
    #[arg(long)]
    check: bool,

    /// Skip type checking
    #[arg(long)]
    skip_typecheck: bool,

    /// Output binary path
    #[arg(short, long, default_value = "output")]
    output: String,
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

    // Type checking
    if !cli.skip_typecheck {
        if let Err(err) = gbasic_typechecker::check(&program) {
            eprintln!("{}: {}", "type error".red().bold(), err);
            process::exit(1);
        }
    }

    if cli.check {
        println!(
            "{}: {} type-checked ({} statements)",
            "ok".green().bold(),
            file,
            program.statements.len()
        );
        return;
    }

    // Code generation
    if let Err(err) = gbasic_irgen::codegen(&program, &cli.output, cli.dump_ir) {
        eprintln!("{}: {}", "codegen error".red().bold(), err);
        process::exit(1);
    }

    if !cli.dump_ir {
        println!(
            "{}: compiled {} -> {}",
            "ok".green().bold(),
            file,
            cli.output
        );
    }
}
