extern crate lisp_repl;
use lisp_repl::*;

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
