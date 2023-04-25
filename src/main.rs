use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use std::fmt;

#[derive(Clone, Debug)]
enum Expr {
    Symbol(String),
    Number(f64),
    List(Vec<Expr>),
}

fn read(input: &str) -> Expr {
    // Implement a simple parser here.
    Expr::Symbol(input.to_string())
}

fn eval(expr: &Expr) -> Result<Expr, > {
    // Implement the evaluation of the Lisp expression.
    Ok(expr.clone())
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Implement the conversion of an expression to a string.
        write!(f, "{:?}", self)
    }
}

fn main() -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                let expr = read(&line);
                match eval(&expr) {
                    Ok(result) => println!("{}", result),
                    Err(err) => println!("Error: {}", err),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
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
