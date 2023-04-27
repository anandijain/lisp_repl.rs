use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::passes::PassManager;
use inkwell::values::{BasicValueEnum, FloatValue, PointerValue};
use inkwell::FloatPredicate;
use peg::error::ParseError;
use peg::parser;
use std::collections::HashMap;
use std::fmt;
use std::num::{ParseFloatError, ParseIntError};

// mod compiler;

parser! {
    grammar lisp_parser() for str {
        // viz
        // rule traced<T>(e: rule<T>) -> T =
        //     &(input:$([_]*) {
        //         #[cfg(feature = "trace")]
        //         println!("[PEG_INPUT_START]\n{}\n[PEG_TRACE_START]", input);
        //     })
        //     e:e()? {?
        //         #[cfg(feature = "trace")]
        //         println!("[PEG_TRACE_STOP]");
        //         e.ok_or("")
        //     }

        // pub rule toplevel() -> Toplevel = traced(<toplevel0()>)

        // actual grammar
        pub rule expr() -> Expr
            = _ e:(
                number()
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

pub fn read(input: &str) -> Result<Expr, String> {
    lisp_parser::expr(input).map_err(|e| e.to_string())
}

pub fn eval(expr: &Expr) -> Result<f64, &'static str> {
    let context = Context::create();
    let module = context.create_module("repl");
    let builder = context.create_builder();

    let compiled_expr = Compiler::compile(&context, &builder, &module, expr)?;
    match compiled_expr.get_constant() {
        Some(x) => Ok(x.0),
        None => Err("Expression did not evaluate to a constant."),
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Implement the conversion of an expression to a string.
        write!(f, "{:?}", self)
    }
}

pub struct Compiler<'a, 'ctx> {
    pub context: &'ctx Context,
    pub builder: &'a Builder<'ctx>,
    pub module: &'a Module<'ctx>,
    pub function: &'a Expr,

    variables: HashMap<String, PointerValue<'ctx>>,
}

impl<'a, 'ctx> Compiler<'a, 'ctx> {
    fn compile_expr(&mut self, expr: &Expr) -> Result<FloatValue<'ctx>, &'static str> {
        match *expr {
            Expr::Float(nb) => Ok(self.context.f64_type().const_float(nb)),
            Expr::Integer(nb) => Ok(self.context.f64_type().const_float(nb as f64)),
            Expr::Symbol(ref name) => match self.variables.get(name.as_str()) {
                Some(var) => Ok(self
                    .builder
                    .build_load(*var, name.as_str())
                    .into_float_value()),
                None => Err("Could not find a matching variable."),
            },
            Expr::List(ref exprs) => {
                if exprs.len() < 2 {
                    return Err("Too few arguments for a binary operation.");
                }

                let op = match exprs[0] {
                    Expr::Symbol(ref op) => op.as_str(),
                    _ => return Err("Expected a binary operator."),
                };

                let lhs = self.compile_expr(&exprs[1])?;
                let rhs = self.compile_expr(&exprs[2])?;

                match op {
                    "+" => Ok(self.builder.build_float_add(lhs, rhs, "tmpadd")),
                    "-" => Ok(self.builder.build_float_sub(lhs, rhs, "tmpsub")),
                    "*" => Ok(self.builder.build_float_mul(lhs, rhs, "tmpmul")),
                    "/" => Ok(self.builder.build_float_div(lhs, rhs, "tmpdiv")),
                    _ => Err("Unknown binary operator."),
                }
            }
        }
    }

    pub fn compile(
        context: &'ctx Context,
        builder: &'a Builder<'ctx>,
        module: &'a Module<'ctx>,
        function: &Expr,
    ) -> Result<FloatValue<'ctx>, &'static str> {
        let mut compiler = Compiler {
            context,
            builder,
            module,
            function,
            variables: HashMap::new(),
        };

        compiler.compile_expr(function)
    }
}
