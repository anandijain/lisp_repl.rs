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
    #[test]
    fn test_addition() {
        let input = "(+ 3.2 4.5)";
        let expected = 7.7;
        let expr = read(input).unwrap();
        let result = eval(&expr).unwrap();
        assert!((result - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn test_subtraction() {
        let input = "(- 7.0 2.5)";
        let expected = 4.5;
        let expr = read(input).unwrap();
        let result = eval(&expr).unwrap();
        assert!((result - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn test_multiplication() {
        let input = "(* 2.0 3.5)";
        let expected = 7.0;
        let expr = read(input).unwrap();
        let result = eval(&expr).unwrap();
        assert!((result - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn test_division() {
        let input = "(/ 10.0 2.0)";
        let expected = 5.0;
        let expr = read(input).unwrap();
        let result = eval(&expr).unwrap();
        assert!((result - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn test_nested_operations() {
        let input = "(+ (* 2.0 3.0) (- 10.0 4.0))";
        let expected = 12.0;
        let expr = read(input).unwrap();
        let result = eval(&expr).unwrap();
        assert!((result - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn test_complex_nested_operations() {
        let input = "(/ (* (- 8.0 2.0) (+ 1.5 2.5)) 10.0)";
        let expected = 2.4;
        let expr = read(input).unwrap();
        let result = eval(&expr).unwrap();
        assert!((result - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn test_variadic_eval() {
        let test_cases = vec![
            ("(+ 1 2 3)", 6.0),
            ("(* 2 3 4)", 24.0),
            ("(- 10 2 3)", 5.0),
            ("(/ 12 2 2)", 3.0),
            ("(+ 1.5 2.5 3)", 7.0),
            ("(* 1.5 2 3)", 9.0),
            ("(- 5.5 2.5 1)", 2.0),
            ("(/ 6.0 2 1.5)", 2.0),
            ("(+ 1 (* 2 3))", 7.0),
            ("(* (- 5 2) (+ 2 3))", 15.0),
            ("(- (/ 24 2) (* 2 3 1))", 6.0),
        ];

        for (i, (input, expected)) in test_cases.into_iter().enumerate() {
            let expr = read(input).unwrap();
            let result = eval(&expr).unwrap();

            assert!(
                result == expected,
                "Test case {}: failed on input '{}' ({:?}), expected {}, got {}",
                i + 1,
                input,
                expr,
                expected,
                result
            );
        }
    }
}
