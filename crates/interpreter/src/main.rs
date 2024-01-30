use std::io::IsTerminal;

use clap::Parser;
use miette::IntoDiagnostic;
use reedline::*;

use boo::evaluation::{EvaluationContext, Evaluator};

#[derive(Debug, Parser)]
struct Args {
    /// Use evaluation by reduction instead of optimized evaluation.
    #[arg(long)]
    reduction: bool,
}

enum Command<'a> {
    Evaluate(&'a dyn Evaluator),
    ShowType,
}

fn main() {
    let args = Args::parse();
    let evaluator: Box<dyn Evaluator> = if args.reduction {
        let mut context = boo_evaluation_reduction::new();
        boo::builtins::prepare(&mut context).unwrap();
        Box::new(context.evaluator())
    } else {
        let mut context = boo::evaluator::new();
        boo::builtins::prepare(&mut context).unwrap();
        Box::new(context.evaluator())
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
    interpret(evaluator, &buffer)
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
                Err(report) => eprintln!("{:?}", report),
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
    let (command, expression) = if buffer.starts_with(':') {
        let (first, rest) = buffer.split_once(' ').unwrap_or((buffer, ""));
        let command_name = &first[1..];
        match command_name {
            "evaluate" => Ok((Command::Evaluate(evaluator), rest)),
            "type" | "t" => Ok((Command::ShowType, rest)),
            _ => Err(miette::miette!("Unknown command: {command_name:?}")),
        }
    } else {
        Ok((Command::Evaluate(evaluator), buffer))
    }?;

    interpret_command(command, expression)
        .map_err(|err| err.with_source_code(expression.to_string()))
}

fn interpret_command(command: Command, expression: &str) -> miette::Result<()> {
    match command {
        Command::Evaluate(evaluator) => {
            let parsed = boo::parse(expression)?;
            let expression = parsed.to_core()?;
            boo_types_hindley_milner::validate(&expression)?;
            let result = evaluator.evaluate(expression)?;
            println!("{result}");
        }
        Command::ShowType => {
            let parsed = boo::parse(expression)?;
            let expression = parsed.to_core()?;
            let expression_type = boo_types_hindley_milner::type_of(&expression)?;
            println!("{expression_type}");
        }
    }
    Ok(())
}
