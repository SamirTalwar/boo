use std::io::IsTerminal;

use clap::Parser;
use miette::IntoDiagnostic;
use reedline::*;

use boo::evaluation::Evaluator;
use boo::*;

#[derive(Debug, Parser)]
struct Args {
    /// Use the naive evaluator instead of the optimized one
    #[arg(long)]
    naive: bool,
}

fn main() {
    let args = Args::parse();
    let evaluator: Box<dyn Evaluator> = if args.naive {
        Box::new(boo_naive_evaluator::NaiveEvaluator::new())
    } else {
        Box::new(OptimizedEvaluator::new())
    };

    let stdin = std::io::stdin();
    if stdin.is_terminal() {
        repl(evaluator.as_ref());
    } else {
        match read_and_interpret(evaluator.as_ref(), stdin) {
            Ok(()) => (),
            Err(report) => eprintln!("{:?}", report),
        }
    }
}

fn read_and_interpret(
    evaluator: &dyn Evaluator,
    mut input: impl std::io::Read,
) -> miette::Result<()> {
    let mut buffer = String::new();
    input.read_to_string(&mut buffer).into_diagnostic()?;
    interpret(evaluator, &buffer).map_err(|report| report.with_source_code(buffer))
}

fn repl(evaluator: &dyn Evaluator) {
    let mut line_editor = Reedline::create();
    let prompt = DefaultPrompt {
        left_prompt: DefaultPromptSegment::Empty,
        right_prompt: DefaultPromptSegment::Empty,
    };

    loop {
        let sig = line_editor.read_line(&prompt);
        match sig {
            Ok(Signal::Success(buffer)) => match interpret(evaluator, &buffer) {
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

fn interpret(evaluator: &dyn Evaluator, buffer: &str) -> miette::Result<()> {
    let parsed = parse(buffer)?;
    let expr = boo::builtins::prepare(parsed);
    let result = evaluator.evaluate(expr)?;
    println!("{}", result);
    Ok(())
}
