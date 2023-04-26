use peg::parser;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::fmt;
use std::num::{ParseFloatError, ParseIntError};

parser! {
    grammar lisp_parser() for str {
        pub rule expr() -> Expr
            = _ e:(number()
                / symbol()
                / list()) _ { e }

        rule number() -> Expr
            = n:$(['-']?['0'..='9']+ ("." ['0'..='9']*)?)
                { parse_number(n).unwrap() }

        rule symbol() -> Expr
            = s:$(['a'..='z' | 'A'..='Z' | '-' | '_' | '+' | '*' | '/' | '?' | '!' | '@' | '#' | '$' | '%' | '&' | '|' | '<' | '>' | '=' | ':' | '"']
                    ['a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '+' | '*' | '/' | '?' | '!' | '@' | '#' | '$' | '%' | '&' | '|' | '<' | '>' | '=' | ':']*  )
                { Expr::Symbol(s.into()) }

        rule list() -> Expr
            = "(" e:(expr() ** (_)) _ ")" { Expr::List(e) }

        rule _() = [' '|'\t'|'\r'|'\n']*
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Symbol(String),
    Integer(i64),
    Float(f64),
    List(Vec<Expr>),
}

#[derive(Debug)]
enum NumberParseError {
    Int(ParseIntError),
    Float(ParseFloatError),
}
impl From<ParseIntError> for NumberParseError {
    fn from(err: ParseIntError) -> Self {
        NumberParseError::Int(err)
    }
}

impl From<ParseFloatError> for NumberParseError {
    fn from(err: ParseFloatError) -> Self {
        NumberParseError::Float(err)
    }
}

fn parse_number(num_str: &str) -> Result<Expr, NumberParseError> {
    num_str
        .parse::<i64>()
        .map(Expr::Integer)
        .map_err(NumberParseError::from)
        .or_else(|_| {
            num_str
                .parse::<f64>()
                .map(Expr::Float)
                .map_err(NumberParseError::from)
        })
}

fn read(input: &str) -> Result<Expr, String> {
    lisp_parser::expr(input).map_err(|e| e.to_string())
}

// fn eval(expr: &Expr) -> Result<Expr, String> {
//     match expr {
//         Expr::Symbol(s) => match s.as_str() {
//             // Add cases for built-in functions here
//             _ => Err(format!("Unknown symbol: {}", s)),
//         },
//         Expr::Integer(_) | Expr::Float(_) => Ok(expr.clone()),
//         Expr::List(list) => {
//             if list.is_empty() {
//                 return Ok(Expr::List(vec![]));
//             }

//             let first = &list[0];

//             match first {
//                 Expr::Symbol(s) => match s.as_str() {
//                     "+" => {
//                         let mut sum = 0.0;
//                         for e in list.iter().skip(1) {
//                             match e {
//                                 Expr::Integer(n) => sum += *n as f64,
//                                 Expr::Float(f) => sum += *f,
//                                 _ => return Err(format!("Invalid operand for +: {:?}", e)),
//                             }
//                         }
//                         Ok(Expr::Float(sum))
//                     }
//                     // Add cases for other built-in functions here
//                     _ => Err(format!("Unknown function: {}", s)),
//                 },
//                 _ => Err(format!("Not a function: {:?}", first)),
//             }
//         }
//     }
// }
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
                // break;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        assert_eq!(read("42"), Ok(Expr::Integer(42)));
        assert_eq!(read("-42"), Ok(Expr::Integer(-42)));
        assert_eq!(read("3.14"), Ok(Expr::Float(3.14)));
        assert_eq!(read("-3.14"), Ok(Expr::Float(-3.14)));
    }

    #[test]
    fn test_parse_symbol() {
        assert_eq!(read("foo"), Ok(Expr::Symbol("foo".to_string())));
        assert_eq!(read("+ "), Ok(Expr::Symbol("+".to_string())));
        assert_eq!(read(" bar-42"), Ok(Expr::Symbol("bar-42".to_string())));
    }

    #[test]
    fn test_parse_list() {
        assert_eq!(
            read(" (+ 1 2)"),
            Ok(Expr::List(vec![
                Expr::Symbol("+".to_string()),
                Expr::Integer(1),
                Expr::Integer(2)
            ]))
        );
        assert_eq!(
            read("(+ (* 2 3) (/ 4 2)) "),
            Ok(Expr::List(vec![
                Expr::Symbol("+".to_string()),
                Expr::List(vec![
                    Expr::Symbol("*".to_string()),
                    Expr::Integer(2),
                    Expr::Integer(3)
                ]),
                Expr::List(vec![
                    Expr::Symbol("/".to_string()),
                    Expr::Integer(4),
                    Expr::Integer(2)
                ])
            ]))
        );
    }
    // Add more tests for other functions, such as `eval`, as you implement them.
}
