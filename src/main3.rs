use lisp_repl::*;
use rustyline::error::ReadlineError;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Editor, Helper};

struct MultilineHelper;

impl Helper for MultilineHelper {}

impl Validator for MultilineHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        let input = ctx.input();
        if is_complete_expression(input) {
            Ok(ValidationResult::Valid(None))
        } else {
            Ok(ValidationResult::Incomplete)
        }
    }
}

fn main() -> Result<(), ReadlineError> {
    
    let mut rl = Editor::<MultilineHelper, ()>::new(); // <-- I changed the type parameters
    rl.set_helper(Some(MultilineHelper)); // <-- You also need to set the helper

    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                match read(&line) {
                    Ok(expr) => match eval(&expr) {
                        Ok(result) => println!("{}", result),
                        Err(err) => println!("Error: {}", err),
                    },
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

fn is_complete_expression(input: &str) -> bool {
    // Implement your logic to check if the input contains a complete expression
    // For example, you can check if the parentheses are balanced.
    let mut open_parens = 0;
    for ch in input.chars() {
        match ch {
            '(' => open_parens += 1,
            ')' => {
                if open_parens > 0 {
                    open_parens -= 1;
                } else {
                    return false;
                }
            }
            _ => (),
        }
    }
    open_parens == 0
}
