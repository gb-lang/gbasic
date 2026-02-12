use clap::Parser as ClapParser;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use colored::Colorize;
use gbasic_common::error::GBasicError;
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

    /// Run the compiled binary after successful compilation
    #[arg(long)]
    run: bool,
}

fn print_error(filename: &str, source: &str, err: &GBasicError) {
    let mut files = SimpleFiles::new();
    let file_id = files.add(filename, source);

    let diagnostic = match err {
        GBasicError::SyntaxError { message, span } |
        GBasicError::TypeError { message, span } |
        GBasicError::NameError { message, span } => {
            let title = match err {
                GBasicError::SyntaxError { .. } => "Syntax error",
                GBasicError::TypeError { .. } => "Type error",
                GBasicError::NameError { .. } => "Name error",
                _ => unreachable!(),
            };
            Diagnostic::error()
                .with_message(title)
                .with_labels(vec![
                    Label::primary(file_id, span.start..span.end).with_message(message),
                ])
        }
        GBasicError::CodegenError { message, span } => {
            let diag = Diagnostic::error().with_message("Codegen error");
            if let Some(span) = span {
                diag.with_labels(vec![
                    Label::primary(file_id, span.start..span.end).with_message(message),
                ])
            } else {
                diag.with_notes(vec![message.clone()])
            }
        }
        GBasicError::InternalError { message } => {
            Diagnostic::error().with_message(format!("Internal error: {message}"))
        }
    };

    let writer = StandardStream::stderr(ColorChoice::Auto);
    let config = term::Config::default();
    let _ = term::emit(&mut writer.lock(), &config, &files, &diagnostic);
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
                print_error(&file, &source, err);
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
            print_error(&file, &source, &err);
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
        print_error(&file, &source, &err);
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

    // Run the binary if --run was specified
    if cli.run && !cli.dump_ir {
        let status = std::process::Command::new(&cli.output)
            .status()
            .unwrap_or_else(|e| {
                eprintln!("{}: failed to run {}: {}", "error".red().bold(), cli.output, e);
                process::exit(1);
            });
        process::exit(status.code().unwrap_or(1));
    }
}
