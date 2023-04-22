pub mod ast;
pub mod error;
pub mod interpreter;
pub mod parser;
mod roundtrip_test;

use reedline::*;

use crate::interpreter::interpret;
use crate::parser::parse;

fn main() {
    let mut line_editor = Reedline::create();
    let prompt = DefaultPrompt {
        left_prompt: DefaultPromptSegment::Empty,
        right_prompt: DefaultPromptSegment::Empty,
    };

    loop {
        let sig = line_editor.read_line(&prompt);
        match sig {
            Ok(Signal::Success(buffer)) => match run(&buffer) {
                Ok(()) => (),
                Err(message) => eprintln!("{}", message),
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

fn run(buffer: &str) -> Result<(), String> {
    let expr = parse(buffer).map_err(|err| format!("{}", err))?;
    let result = interpret(expr);
    println!("{}", result);
    Ok(())
}
