use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, digit1, multispace0},
    combinator::{map, recognize},
    multi::many0,
    sequence::{delimited, preceded, terminated},
    IResult,
};
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor};
use std::{fmt, io::Read};

#[derive(Clone, Debug)]
enum Expr {
    Symbol(String),
    Number(f64),
    List(Vec<Expr>),
}

fn parse_number(input: &str) -> IResult<&str, Expr> {
    map(
        recognize(terminated(digit1, many0(alt((tag("."), tag("e")))))),
        |num_str: &str| Expr::Number(num_str.parse::<f64>().unwrap()),
    )(input)
}

fn parse_symbol(input: &str) -> IResult<&str, Expr> {
    map(alpha1, |sym_str: &str| Expr::Symbol(sym_str.to_string()))(input)
}

fn parse_list(input: &str) -> IResult<&str, Expr> {
    delimited(
        tag("("),
        map(many0(preceded(multispace0, parse_expr)), Expr::List),
        tag(")"),
    )(input)
}

fn parse_expr(input: &str) -> IResult<&str, Expr> {
    alt((parse_number, parse_symbol, parse_list))(input)
}

fn read(input: &str) -> Result<Expr, String> {
    match parse_expr(input) {
        Ok((_, expr)) => Ok(expr),
        Err(err) => Err("yo".to_string()), // FIXME
    }
}

fn eval(expr: &Expr) -> Result<Expr, String> {
    // Implement the evaluation of the Lisp expression.
    Ok(expr.clone())
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Implement the conversion of an expression to a string.
        write!(f, "{:?}", self)
    }
}

fn main() -> Result<(), ReadlineError> {
    let mut rl = DefaultEditor::new()?;
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
