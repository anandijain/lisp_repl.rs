use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    passes::PassManager,
    types::{BasicMetadataTypeEnum, BasicTypeEnum},
    values::{BasicMetadataValueEnum, BasicValueEnum, FloatValue, FunctionValue, PointerValue},
    FloatPredicate,
};
use peg::{error::ParseError, parser};
use std::{
    borrow::Borrow,
    collections::HashMap,
    fmt,
    iter::Peekable,
    num::{ParseFloatError, ParseIntError},
    ops::DerefMut,
    str::Chars,
};

// mod compiler;
// (define fact
//     (lambda (n)
//       (if (< n 2)
//           1
//           (* n (fact (- n 1))))))
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

pub struct Compiler<'a, 'ctx> {
    pub context: &'ctx Context,
    pub builder: &'a Builder<'ctx>,
    pub module: &'a Module<'ctx>,
    pub function: &'a Expr,

    variables: HashMap<String, PointerValue<'ctx>>,
}

impl<'a, 'ctx> Compiler<'a, 'ctx> {
    #[inline]
    fn get_function(&self, name: &str) -> Option<FunctionValue<'ctx>> {
        self.module.get_function(name)
    }

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
                let op = match &exprs[0] {
                    Expr::Symbol(op) => op.as_str(),
                    _ => {
                        // let error_msg = format!("Expected operator, got {:?}", exprs[0].clone());
                        return Err("expected operator");
                    }
                };

                let args: Result<Vec<_>, _> = exprs[1..]
                    .iter()
                    .map(|expr| self.compile_expr(expr))
                    .collect();

                // println!("variables: {:?}", self.variables);

                let result = match args {
                    Ok(compiled_args) => match op {
                        "+" => compiled_args
                            .into_iter()
                            .reduce(|lhs, rhs| self.builder.build_float_add(lhs, rhs, "tmpadd")),
                        "-" => compiled_args
                            .into_iter()
                            .reduce(|lhs, rhs| self.builder.build_float_sub(lhs, rhs, "tmpsub")),
                        "*" => compiled_args
                            .into_iter()
                            .reduce(|lhs, rhs| self.builder.build_float_mul(lhs, rhs, "tmpmul")),
                        "/" => compiled_args
                            .into_iter()
                            .reduce(|lhs, rhs| self.builder.build_float_div(lhs, rhs, "tmpdiv")),
                        _ => return Err("Unknown binary operator."),
                    },
                    Err(err) => {
                        return Err("idk");
                    }
                };

                match result {
                    Some(res) => Ok(res),
                    None => Err("Insufficient arguments for the operator."),
                }
            }
            _ => Err("Unknown expression."),
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

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
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
