use std::str::FromStr;
use std::sync::Arc;

use clap::Parser;
use miette::IntoDiagnostic;
use reedline::*;

use boo::ast::*;
use boo::identifier::*;
use boo::native::*;
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
    if atty::is(atty::Stream::Stdin) {
        repl(&args);
    } else {
        match read_and_interpret(&args, std::io::stdin()) {
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
    let builtins: Vec<(Identifier, boo::parser::Expr)> =
        vec![(Identifier::from_str("trace").unwrap(), {
            let parameter = Identifier::from_str("param").unwrap();
            boo::parser::Expr::new(
                0.into(),
                Expression::Function(Function {
                    parameter: parameter.clone(),
                    body: boo::parser::Expr::new(
                        0.into(),
                        Expression::Native(Native {
                            unique_name: Identifier::from_str("trace").unwrap(),
                            implementation: Arc::new(move |context| {
                                let value = context.lookup_value(&parameter)?;
                                eprintln!("trace: {}", value);
                                Ok(value)
                            }),
                        }),
                    ),
                }),
            )
        })];
    let mut expr = parse(buffer)?;
    for (name, builtin) in builtins.into_iter().rev() {
        expr = boo::parser::Expr::new(
            0.into(),
            Expression::Assign(Assign {
                name,
                value: builtin,
                inner: expr,
            }),
        );
    }
    if args.naive {
        let result = naively_evaluate(expr)?;
        println!("{}", result);
    } else {
        let result = evaluate(expr)?;
        println!("{}", result);
    }
    Ok(())
}
