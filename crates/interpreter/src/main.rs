use miette::IntoDiagnostic;
use reedline::*;

use boo::*;

fn main() {
    if atty::is(atty::Stream::Stdin) {
        repl();
    } else {
        match read_and_interpret(std::io::stdin()) {
            Ok(()) => (),
            Err(report) => eprintln!("{:?}", report),
        }
    }
}

fn read_and_interpret(mut input: impl std::io::Read) -> miette::Result<()> {
    let mut buffer = String::new();
    input.read_to_string(&mut buffer).into_diagnostic()?;
    interpret(&buffer).map_err(|report| report.with_source_code(buffer))
}

fn repl() {
    let mut line_editor = Reedline::create();
    let prompt = DefaultPrompt {
        left_prompt: DefaultPromptSegment::Empty,
        right_prompt: DefaultPromptSegment::Empty,
    };

    loop {
        let sig = line_editor.read_line(&prompt);
        match sig {
            Ok(Signal::Success(buffer)) => match interpret(&buffer) {
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

fn interpret(buffer: &str) -> miette::Result<()> {
    let tokens = lex(buffer)?;
    let expr = parse(&tokens)?;
    let result = evaluate(expr)?;
    println!("{}", result);
    Ok(())
}
