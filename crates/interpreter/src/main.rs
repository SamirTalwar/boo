use std::io::IsTerminal;

use clap::Parser;
use miette::IntoDiagnostic;
use reedline::*;

use boo::*;
use boo_naive_evaluator::naively_evaluate;

#[derive(Debug, Parser)]
struct Args {
    /// Use the naive evaluator instead of the optimized one
    #[arg(long)]
    naive: bool,
}

fn main() {
    let args = Args::parse();
    let stdin = std::io::stdin();
    if stdin.is_terminal() {
        repl(&args);
    } else {
        match read_and_interpret(&args, stdin) {
            Ok(()) => (),
            Err(report) => eprintln!("{:?}", report),
        }
    }
}

fn read_and_interpret(args: &Args, mut input: impl std::io::Read) -> miette::Result<()> {
    let mut buffer = String::new();
    input.read_to_string(&mut buffer).into_diagnostic()?;
    interpret(args, &buffer).map_err(|report| report.with_source_code(buffer))
}

fn repl(args: &Args) {
    let mut line_editor = Reedline::create();
    let prompt = DefaultPrompt {
        left_prompt: DefaultPromptSegment::Empty,
        right_prompt: DefaultPromptSegment::Empty,
    };

    loop {
        let sig = line_editor.read_line(&prompt);
        match sig {
            Ok(Signal::Success(buffer)) => match interpret(args, &buffer) {
                Ok(()) => (),
                Err(report) => eprintln!("{:?}", report.with_source_code(buffer)),
            },
            Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => {
                break;
            }
            Err(err) => {
                eprintln!("Error: {}", err);
            }
        }
    }
}

fn interpret(args: &Args, buffer: &str) -> miette::Result<()> {
    let parsed = parse(buffer)?;
    let expr = boo::builtins::prepare(parsed);
    if args.naive {
        let result = naively_evaluate(expr)?;
        println!("{}", result);
    } else {
        let result = evaluate(expr)?;
        println!("{}", result);
    }
    Ok(())
}
