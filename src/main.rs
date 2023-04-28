use inkwell::context::Context;
use lisp_repl::*;
use rug::{Float, Integer, Rational};
use rustyline::error::ReadlineError;
use rustyline::history::History;
use rustyline::validate::MatchingBracketValidator;
use rustyline::{Cmd, Editor, EventHandler, KeyCode, KeyEvent, Modifiers};
use rustyline::{Completer, Helper, Highlighter, Hinter, Validator};
#[derive(Completer, Helper, Highlighter, Hinter, Validator)]
struct InputValidator {
    #[rustyline(Validator)]
    brackets: MatchingBracketValidator,
}

fn main() -> Result<(), ReadlineError> {
    let mut display_parser_output = true;
    let mut display_compiler_output = false;

    for arg in std::env::args() {
        match arg.as_str() {
            "--dp" => display_parser_output = true,
            "--dc" => display_compiler_output = true,
            _ => (),
        }
    }

    println!("p{} c{}", display_parser_output, display_compiler_output);

    let h = InputValidator {
        brackets: MatchingBracketValidator::new(),
    };
    let mut rl = Editor::new()?;
    rl.set_helper(Some(h));

    rl.bind_sequence(
        KeyEvent(KeyCode::Enter, Modifiers::ALT),
        EventHandler::Simple(Cmd::Newline),
    );

    let context = Context::create();
    let module = context.create_module("repl");
    let builder = context.create_builder();
    let mut previous_exprs = Vec::new();

    loop {
        let prompt_str = format! {"[%{}]>> ", rl.history().len().to_string()};

        let readline = rl.readline(prompt_str.as_str());
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                match read(&line) {
                    Ok(expr) => {
                        // for (index, prev) in previous_exprs.iter().enumerate() {
                        //     println!("{}: {:?}", index, prev);

                        //     Compiler::compile(&context, &builder, &module, prev)
                        //         .expect("Cannot re-add previously compiled function.");
                        // }

                        previous_exprs.push(expr.clone());
                        if display_parser_output {
                            println!("{:?}", expr);
                        }
                        match Compiler::compile(&context, &builder, &module, &expr) {
                            Ok(result) => println!("{}", result),
                            Err(err) => println!("Error: {}", err),
                        }
                    }
                    Err(err) => println!("Error: {}", err),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    rl.save_history("history.txt")?;
    Ok(())
}
