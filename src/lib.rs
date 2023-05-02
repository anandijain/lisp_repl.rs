#[warn(unused_imports)]
use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    passes::PassManager,
    types::{BasicMetadataTypeEnum, BasicTypeEnum},
    values::{BasicMetadataValueEnum, BasicValueEnum, FloatValue, FunctionValue, PointerValue},
    FloatPredicate,
};
use inkwell::{
    types::BasicType,
    values::{AnyValue, AnyValueEnum, BasicValue, GenericValue},
    OptimizationLevel,
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

use std::error::Error;

parser! {
    grammar lisp_parser() for str {
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
                    ['a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '+' | '*' | '/' | '?' | '!' | '@' | '#' | '$' | '%' | '&' | '|' | '<' | '>' | '=' | ':' | '.' ]*  )
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

fn extract_op_and_args<'a>(exprs: &'a [Expr]) -> Result<(&'a str, &'a [Expr]), &'static str> {
    match exprs.split_first() {
        Some((Expr::Symbol(op), args)) => Ok((op.as_str(), args)),
        _ => Err("expected operator"),
    }
}

pub struct Compiler<'a, 'ctx> {
    pub context: &'ctx Context,
    pub builder: &'a Builder<'ctx>,
    pub fpm: &'a PassManager<FunctionValue<'ctx>>,
    pub module: &'a Module<'ctx>,
    pub expr: &'a Expr,
    // scopes: Vec<HashMap<String, FloatValue<'ctx>>>,
    pub global_scope: &'a mut HashMap<String, FloatValue<'ctx>>,
}

impl<'a, 'ctx> Compiler<'a, 'ctx> {
    /// Gets a defined function given its name.
    #[inline]
    fn get_function(&self, name: &str) -> Option<FunctionValue<'ctx>> {
        self.module.get_function(name)
    }

    // fn lookup_variable(&self, var_name: &str) -> Option<&FloatValue<'ctx>> {
    //     for scope in self.scopes.iter().rev() {
    //         if let Some(value) = scope.get(var_name) {
    //             return Some(value);
    //         }
    //     }
    //     None
    // }

    /// Compiles the specified `Expr` into an LLVM `FloatValue`.
    pub fn compile_expr(&mut self, expr: &'a Expr) -> Result<FloatValue<'ctx>, &'static str> {
        match expr {
            Expr::Float(nb) => Ok(self.context.f64_type().const_float(*nb)),
            Expr::Integer(nb) => Ok(self.context.f64_type().const_float(*nb as f64)),
            Expr::Symbol(ref name) => match self.global_scope.get(name.as_str()) {
                Some(value) => Ok(*value),
                None => Err("Could not find a matching variable."),
            },
            Expr::List(ref exprs) => {
                let (op, args) = extract_op_and_args(exprs)?;

                match op {
                    "define" => {
                        if args.len() != 2 {
                            return Err("define requires a variable name or function definition and a value or expression.");
                        }

                        if let Expr::List(func_def) = &args[0] {
                            if func_def.len() < 1 {
                                return Err("Function definition should have a name and may have parameters.");
                            }

                            if let Expr::Symbol(func_name) = &func_def[0] {
                                let arg_names: Vec<String> = func_def
                                    .iter()
                                    .skip(1)
                                    .filter_map(|arg| match arg {
                                        Expr::Symbol(s) => Some(s.to_string()),
                                        _ => None,
                                    })
                                    .collect();
                                let arity = arg_names.len();

                                if arity != func_def.len() - 1 {
                                    return Err("Function arguments should be symbols.");
                                }
                                let arg_types = self.context.f64_type();
                                // arg_names.len();
                                // let bmte2 = Into::<BasicMetadataTypeEnum>::into(arg_types.as_basic_type_enum());
                                let bmte = BasicMetadataTypeEnum::FloatType(arg_types);
                                let v = vec![bmte; arg_names.len()];
                                let func_type = self.context.f64_type().fn_type(
                                    v.as_slice(),
                                    // &[self.context.f64_type().into(); arg_names.len()][..],
                                    false,
                                );
                                let func_value =
                                    self.module.add_function(func_name, func_type, None);

                                let basic_block =
                                    self.context.append_basic_block(func_value, "entry");
                                self.builder.position_at_end(basic_block);

                                // Create a new scope for function arguments
                                let mut arg_scope = HashMap::new();
                                for (i, &ref arg_name) in arg_names.iter().enumerate() {
                                    let param = func_value
                                        .get_nth_param(i as u32)
                                        .unwrap()
                                        .into_float_value();
                                    arg_scope.insert(arg_name.to_string(), param);
                                }

                                // Push local scope on the scopes stack
                                // self.scopes.push(arg_scope);

                                let body = self.compile_expr(&args[1])?;
                                self.builder.build_return(Some(&body));

                                // Pop local scope from the stack
                                // self.scopes.pop();

                                Ok(body)
                            } else {
                                Err("Function definition should start with a symbol for its name.")
                            }
                        } else {
                            // Handle the define variable case
                            if let Expr::Symbol(var_name) = &args[0] {
                                let value = self.compile_expr(&args[1])?;
                                self.global_scope.insert(var_name.clone(), value);
                                Ok(value)
                            } else {
                                Err("define requires a variable name to be a symbol.")
                            }
                        }
                    }

                    "+" => Ok(self.builder.build_float_add(
                        self.compile_expr(&args[0])?,
                        self.compile_expr(&args[1])?,
                        "tmpadd",
                    )),
                    "-" => Ok(self.builder.build_float_sub(
                        self.compile_expr(&args[0])?,
                        self.compile_expr(&args[1])?,
                        "tmpsub",
                    )),
                    "*" => Ok(self.builder.build_float_mul(
                        self.compile_expr(&args[0])?,
                        self.compile_expr(&args[1])?,
                        "tmpmul",
                    )),
                    "/" => Ok(self.builder.build_float_div(
                        self.compile_expr(&args[0])?,
                        self.compile_expr(&args[1])?,
                        "tmpdiv",
                    )),
                    // "-" => Ok(self.builder.build_float_sub(lhs, rhs, "tmpsub")),
                    // "*" => Ok(self.builder.build_float_mul(lhs, rhs, "tmpmul")),
                    // "/" => Ok(self.builder.build_float_div(lhs, rhs, "tmpdiv")),
                    _ => {
                        // Handle other operators/funcs, e.g. (+ x y)
                        // or custom functions if you plan to support them.
                        Err("Not implemented!")
                    }
                }
            }
        }
    }

    /// Compiles the specified `Function` in the given `Context` and using the specified `Builder`, `PassManager`, and `Module`.
    pub fn compile(
        context: &'ctx Context,
        builder: &'a Builder<'ctx>,
        pass_manager: &'a PassManager<FunctionValue<'ctx>>,
        module: &'a Module<'ctx>,
        expr: &Expr,
        global_scope: &'a mut HashMap<String, FloatValue<'ctx>>,
    ) -> Result<FloatValue<'ctx>, &'static str> {
        let mut compiler = Compiler {
            context,
            builder,
            fpm: pass_manager,
            module,
            expr,
            global_scope, // scopes: vec![global_scope],
        };
        // Directly call the modified compile_expr method
        compiler.compile_expr(expr)
    }
}
