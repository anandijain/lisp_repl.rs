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

// pub fn eval(expr: &Expr) -> Result<(), &'static str> {
//     let context = Context::create();
//     let module = context.create_module("repl");
//     let builder = context.create_builder();

//     let compiled_expr = Compiler::compile(&context, &builder, &module, expr)?;
//     // Ok(compiled_expr)
//     Ok(())
//     // compiled_expr.
//     // match compiled_expr.get_constant() {
//     //     Some(x) => Ok(x.0),
//     //     None => Err("Expression did not evaluate to a constant."),
//     // }
// }

pub struct Compiler<'a, 'ctx> {
    pub context: &'ctx Context,
    pub builder: &'a Builder<'ctx>,
    pub module: &'a Module<'ctx>,
    pub expr: &'a Expr,
    // pub functions: HashMap<String, FunctionValue<'ctx>>,
    pub variables: HashMap<String, PointerValue<'ctx>>,
    fpm: &'a PassManager<FunctionValue<'ctx>>,
    fn_value_opt: Option<FunctionValue<'ctx>>,
}

impl<'a, 'ctx> Compiler<'a, 'ctx> {
    #[inline]
    fn get_function(&self, name: &str) -> Option<FunctionValue<'ctx>> {
        let x = self.module.get_function(name);
        x
        // match  {
        //     Some(x) => Some(x)
        //     None => {

        //     }
        // }

        // add intrinsic fallback?
    }

    /// Returns the `FunctionValue` representing the function being compiled.
    #[inline]
    fn fn_value(&self) -> FunctionValue<'ctx> {
        self.fn_value_opt.unwrap()
    }

    /// Creates a new stack allocation instruction in the entry block of the function.
    fn create_entry_block_alloca(
        &self,
        function_value: FunctionValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();

        let entry = function_value.get_first_basic_block().unwrap();

        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }

        builder.build_alloca(self.context.f64_type(), name)
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<AnyValueEnum<'ctx>, &'static str> {
        match *expr {
            Expr::Float(nb) => Ok(self.context.f64_type().const_float(nb).into()),
            Expr::Integer(nb) => Ok(self.context.f64_type().const_float(nb as f64).into()),
            Expr::Symbol(ref name) => match self.variables.get(name.as_str()) {
                Some(var) => Ok(self
                    .builder
                    .build_load(*var, name.as_str())
                    .into_float_value()
                    .into()),
                None => Err("Could not find a matching variable."),
            },
            Expr::List(ref exprs) => {
                let (op, args) = match exprs.split_first() {
                    Some((Expr::Symbol(op), args)) => (op.as_str(), args),
                    _ => return Err("expected operator"),
                };

                if op == "define" {
                    match args.split_first() {
                        Some((Expr::Symbol(name), value_exprs)) => {
                            let value_expr = value_exprs
                                .get(0)
                                .ok_or("Expected a value expression for define")?;

                            let val = self.compile_expr(value_expr)?;

                            let temp_fn_type = self.context.void_type().fn_type(&[], false);
                            let temp_fn_value = self.module.add_function("temp_define_fn", temp_fn_type, None);
                            let temp_entry_block = self.context.append_basic_block(temp_fn_value, "entry");

                            let alloca = self.create_entry_block_alloca(temp_fn_value, name);
                            self.builder.build_store(alloca, val.into_float_value());

                            self.variables.insert(name.to_string(), alloca);
                            println!("variables: {:?}", self.variables);
                            // Return the assigned value as the result of the define
                            Ok(self.context.f64_type().const_float(0.).into())
                        }
                        _ => Err("Expected symbol as first argument to define."),
                    }
                    // let sig = args[1];
                    // let body = args[2];
                    // let (fn_name, fn_arg_names) = match sig {
                    //     Expr::List(ref sig_args) => {
                    //         (sig_args[1]
                    //     }
                    // }
                    // let fn_name =
                    // return Ok(self.compile_fn(op, compiled_args).unwrap().as_any_value_enum())
                    // return Err("define not implemented");
                } else {
                    let mut compiled_args = Vec::with_capacity(args.len());
                    for arg in args {
                        compiled_args.push(self.compile_expr(arg)?);
                    }
                    match self.get_function(op) {
                        Some(fun) => {
                            // inkwell::values::An
                            let argsv: Vec<BasicMetadataValueEnum> = compiled_args
                                .iter()
                                .by_ref()
                                .map(|&val| val.into_float_value().into())
                                .collect();

                            match self
                                .builder
                                .build_call(fun, argsv.as_slice(), "tmp")
                                .try_as_basic_value()
                                .left()
                            {
                                Some(value) => Ok(value.into_float_value().into()),
                                None => Err("Invalid call produced."),
                            }
                        }
                        None => {
                            let lhs = compiled_args[0].into_float_value();
                            let rhs = compiled_args[1].into_float_value();
                            match op {
                                "+" => Ok(self.builder.build_float_add(lhs, rhs, "tmpadd").into()),
                                "-" => Ok(self.builder.build_float_sub(lhs, rhs, "tmpsub").into()),
                                "*" => Ok(self.builder.build_float_mul(lhs, rhs, "tmpmul").into()),
                                "/" => Ok(self.builder.build_float_div(lhs, rhs, "tmpdiv").into()),
                                _ => Err("Unknown function"),
                            }
                        }
                    }
                }
            }
            _ => Err("Unknown expression."),
        }
    }
    /// Compiles the specified `Prototype` into an extern LLVM `FunctionValue`.
    /// expr is the whole list (define (name args) body))
    fn compile_prototype(&self, expr: &Expr) -> Result<FunctionValue<'ctx>, &'static str> {
        // i think basically, if head(expr) != "define" then the function is anon with 0 args
        // assert!()

        let ret_type = self.context.f64_type();
        let args_types = std::iter::repeat(ret_type)
            .take(proto.args.len())
            .map(|f| f.into())
            .collect::<Vec<BasicMetadataTypeEnum>>();
        let args_types = args_types.as_slice();

        let fn_type = self.context.f64_type().fn_type(args_types, false);
        let fn_val = self.module.add_function(proto.name.as_str(), fn_type, None);

        // set arguments names
        for (i, arg) in fn_val.get_param_iter().enumerate() {
            arg.into_float_value().set_name(proto.args[i].as_str());
        }

        // finally return built prototype
        Ok(fn_val)
    }

    /// Compiles the specified `Function` into an LLVM `FunctionValue`.
    fn compile_fn(&mut self) -> Result<FunctionValue<'ctx>, &'static str> {
        // let proto = &self.expr.prototype;
        let function = self.compile_prototype(proto)?;

        // got external function, returning only compiled prototype
        if self.function.body.is_none() {
            return Ok(function);
        }

        let entry = self.context.append_basic_block(function, "entry");

        self.builder.position_at_end(entry);

        // update fn field
        self.fn_value_opt = Some(function);

        // build variables map
        self.variables.reserve(proto.args.len());

        for (i, arg) in function.get_param_iter().enumerate() {
            let arg_name = proto.args[i].as_str();
            let alloca = self.create_entry_block_alloca(arg_name);

            self.builder.build_store(alloca, arg);

            self.variables.insert(proto.args[i].clone(), alloca);
        }

        // compile body
        let body = self.compile_expr(self.function.body.as_ref().unwrap())?;

        self.builder.build_return(Some(&body));

        // return the whole thing after verification and optimization
        if function.verify(true) {
            self.fpm.run_on(&function);

            Ok(function)
        } else {
            unsafe {
                function.delete();
            }

            Err("Invalid generated function.")
        }
    }
    pub fn compile(
        context: &'ctx Context,
        builder: &'a Builder<'ctx>,
        pass_manager: &'a PassManager<FunctionValue<'ctx>>,
        module: &'a Module<'ctx>,
        expr: &Expr,
    ) -> Result<AnyValueEnum<'ctx>, &'static str> {
        let mut compiler = Compiler {
            context,
            builder,
            fpm: pass_manager,
            module,
            expr,
            fn_value_opt: None,
            variables: HashMap::new(),
        };

        // Ok(
        // compiler.compile_expr(expr)
        compiler.compile_fn()
        // .unwrap()
        // .as_any_value_enum()
        // .into_float_value())
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
