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
    types::{BasicType, PointerType},
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
    pub global_scope: &'a mut HashMap<String, PointerValue<'ctx>>,
    fn_value_opt: Option<FunctionValue<'ctx>>,
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

    #[inline]
    fn fn_value(&self) -> FunctionValue<'ctx> {
        self.fn_value_opt.unwrap()
    }

    /// Creates a new stack allocation instruction in the entry block of the function.
    fn create_entry_block_alloca(&self, name: &str) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();

        let entry = self.fn_value().get_first_basic_block().unwrap();

        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }

        builder.build_alloca(self.context.f64_type(), name)
    }

    /// Compiles the specified `Expr` into an LLVM `FloatValue`.
    pub fn compile_expr(&mut self, expr: &'a Expr) -> Result<FloatValue<'ctx>, &'static str> {
        match expr {
            Expr::Float(nb) => Ok(self.context.f64_type().const_float(*nb)),
            Expr::Integer(nb) => Ok(self.context.f64_type().const_float(*nb as f64)),
            Expr::Symbol(ref name) => match self.global_scope.get(name.as_str()) {
                // Some(value) => Ok(*value),
                Some(var) => {
                    println!("self.global_scope: {:?}", self.global_scope);

                    Ok(self
                        .builder
                        .build_load(*var, name.as_str())
                        .into_float_value())
                }
                None => {
                    println!("self.global_scope: {:?}", self.global_scope);
                    Err("Could not find a matching variable.")
                }
            },
            Expr::List(ref exprs) => {
                let (op, args) = extract_op_and_args(exprs)?;
                println!("{op}({:?})", args);
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

                                let function =
                                    self.compile_prototype(func_name, arg_names.clone())?;

                                let entry = self.context.append_basic_block(function, "entry");
                                self.builder.position_at_end(entry);
                                self.fn_value_opt = Some(function);

                                // Create a new scope for function arguments
                                // let mut arg_scope = HashMap::new();

                                // for (i, &ref arg_name) in arg_names.iter().enumerate() {
                                //     let param = function
                                //         .get_nth_param(i as u32)
                                //         .unwrap()
                                //         .into_float_value();
                                //     arg_scope.insert(arg_name.to_string(), param);
                                // }

                                // Push local scope on the scopes stack
                                // self.scopes.push(arg_scope);

                                // let mut prev_global_scope = self.global_scope.clone();
                                // self.global_scope.extend(arg_scope.clone());
                                for (i, arg) in function.get_param_iter().enumerate() {
                                    // let arg_name = proto.args[i].as_str();
                                    let alloca = self.create_entry_block_alloca(&arg_names[i]);
                                    self.builder.build_store(alloca, arg);
                                    self.global_scope.insert(arg_names[i].clone(), alloca);

                                    // self.variables.insert(proto.args[i].clone(), alloca);
                                }

                                let body = self.compile_expr(&args[1])?;
                                self.builder.build_return(Some(&body));

                                // self.global_scope = &mut prev_global_scope;

                                // Pop local scope from the stack
                                // self.scopes.pop();

                                Ok(body)
                            } else {
                                Err("Function definition should start with a symbol for its name.")
                            }
                        } else {
                            println!("defining a variable ");

                            // Handle the define variable case
                            if let Expr::Symbol(var_name) = &args[0] {
                                // let var_name_str = var_name.as_str();

                                // let initial_val = match *initializer {
                                //     Some(ref init) => self.compile_expr(init)?,
                                //     None => self.context.f64_type().const_float(0.),
                                // };

                                let value = self.compile_expr(&args[1])?;
                                let function = self.compile_prototype(&var_name, vec![])?;

                                let entry = self.context.append_basic_block(function, "entry");

                                self.builder.position_at_end(entry);

                                // update fn field
                                self.fn_value_opt = Some(function);

                                let alloca = self.create_entry_block_alloca(var_name);

                                self.builder.build_store(alloca, value);

                                self.global_scope.insert(var_name.clone(), alloca);
                                self.builder
                                    .build_return(Some(&value.as_basic_value_enum()));
                                // let alloca = self.create_entry_block_alloca(var_name);

                                // self.builder.build_store(alloca, initial_val);

                                Ok(value)
                            } else {
                                Err("define requires a variable name to be a symbol.")
                            }
                        }
                    }
                    // i think in all the cases below this we want to compile a prototype and anonymous function with zero args.
                    // "+" => Ok(self.builder.build_float_add(
                    //     self.compile_expr(&args[0])?,
                    //     self.compile_expr(&args[1])?,
                    //     "tmpadd",
                    // )),
                    // "-" => Ok(self.builder.build_float_sub(
                    //     self.compile_expr(&args[0])?,
                    //     self.compile_expr(&args[1])?,
                    //     "tmpsub",
                    // )),
                    // "*" => Ok(self.builder.build_float_mul(
                    //     self.compile_expr(&args[0])?,
                    //     self.compile_expr(&args[1])?,
                    //     "tmpmul",
                    // )),
                    // "/" => Ok(self.builder.build_float_div(
                    //     self.compile_expr(&args[0])?,
                    //     self.compile_expr(&args[1])?,
                    //     "tmpdiv",
                    // )),
                    // "-" => Ok(self.builder.build_float_sub(lhs, rhs, "tmpsub")),
                    // "*" => Ok(self.builder.build_float_mul(lhs, rhs, "tmpmul")),
                    // "/" => Ok(self.builder.build_float_div(lhs, rhs, "tmpdiv")),
                    _ => {
                        let compiled_args: Result<Vec<FloatValue<'ctx>>, _> =
                            args.into_iter().map(|arg| self.compile_expr(arg)).collect();
                        match compiled_args {
                            Ok(compiled_args) => {
                                match op {
                                    "+" => {
                                        match compiled_args.into_iter().reduce(|lhs, rhs| {
                                            self.builder.build_float_add(lhs, rhs, "tmpadd")
                                        }) {
                                            Some(result) => Ok(result),
                                            None => Err("Error: Addition requires at least one argument."),
                                        }
                                    },
                                    "-" => {
                                        match compiled_args.into_iter().reduce(|lhs, rhs| {
                                            self.builder.build_float_sub(lhs, rhs, "tmpsub")
                                        }) {
                                            Some(result) => Ok(result),
                                            None => Err("Error: Subtraction requires at least one argument."),
                                        }
                                    },
                                    "*" => {
                                        match compiled_args.into_iter().reduce(|lhs, rhs| {
                                            self.builder.build_float_mul(lhs, rhs, "tmpmul")
                                        }) {
                                            Some(result) => Ok(result),
                                            None => Err("Error: Multiplication requires at least one argument."),
                                        }
                                    },
                                    "/" => {
                                        match compiled_args.into_iter().reduce(|lhs, rhs| {
                                            self.builder.build_float_div(lhs, rhs, "tmpdiv")
                                        }) {
                                            Some(result) => Ok(result),
                                            None => Err("Error: Division requires at least one argument."),
                                        }
                                    },
                                    _ => {
                                        // (square 2)

                                        match self.module.get_function(op) {
                                            Some(f) => {
                                                let function =
                                                    self.compile_prototype("anon", vec![])?;

                                                let entry = self
                                                    .context
                                                    .append_basic_block(function, "entry");

                                                self.builder.position_at_end(entry);

                                                // update fn field
                                                self.fn_value_opt = Some(function);

                                                // self.builder.build_return(Some(&value.as_basic_value_enum()));

                                                let mut compiled_args = vec![];

                                                for arg in args.iter() {
                                                    let foo = BasicMetadataValueEnum::FloatValue(
                                                        self.compile_expr(arg).unwrap(),
                                                    );
                                                    compiled_args.push(foo);
                                                }
                                                let body = self
                                                    .builder
                                                    .build_call(
                                                        f,
                                                        compiled_args.as_slice(),
                                                        "tmpcall",
                                                    )
                                                    .try_as_basic_value()
                                                    .left()
                                                    .unwrap()
                                                    .into_float_value();

                                                self.builder.build_return(Some(&body));
                                                return Ok(body);
                                            }
                                            None => Err("no function found with that name"),
                                        }
                                    }
                                }
                            }
                            Err(err) => Err(err),
                        }
                    }
                }
            }
        }
    }

    /// Compiles the specified `Prototype` into an extern LLVM `FunctionValue`.
    /// nargs is the number of arguments the function takes. not the number of arguments in the List
    fn compile_prototype(
        &self,
        name: &str,
        arg_names: Vec<String>,
    ) -> Result<FunctionValue<'ctx>, &'static str> {
        let ret_type = self.context.f64_type();
        let nargs = arg_names.len();
        let args_types = std::iter::repeat(ret_type)
            .take(nargs)
            .map(|f| f.into())
            .collect::<Vec<BasicMetadataTypeEnum>>();
        let args_types = args_types.as_slice();

        let fn_type = self.context.f64_type().fn_type(args_types, false);
        let fn_val = self.module.add_function(name, fn_type, None);

        // set arguments names
        for (i, arg) in fn_val.get_param_iter().enumerate() {
            arg.into_float_value().set_name(&arg_names[i]);
        }

        // finally return built prototype
        Ok(fn_val)
    }

    // /// Compiles the specified `Function` into an LLVM `FunctionValue`.
    // fn compile_fn(&mut self) -> Result<FunctionValue<'ctx>, &'static str> {
    //     println!("in compile_fn compiling fn: {:?}", proto);
    //     println!("in compile_fn compiling fn: {:?}", proto);
    //     let function = self.compile_prototype(proto)?;

    //     let entry = self.context.append_basic_block(function, "entry");

    //     self.builder.position_at_end(entry);

    //     // update fn field
    //     self.fn_value_opt = Some(function);

    //     // build variables map
    //     self.variables.reserve(proto.args.len());

    //     for (i, arg) in function.get_param_iter().enumerate() {
    //         let arg_name = proto.args[i].as_str();
    //         let alloca = self.create_entry_block_alloca(arg_name);

    //         self.builder.build_store(alloca, arg);

    //         self.variables.insert(proto.args[i].clone(), alloca);
    //     }

    //     // compile body
    //     let body = self.compile_expr(self.function.body.as_ref().unwrap())?;

    //     self.builder.build_return(Some(&body));

    //     // return the whole thing after verification and optimization
    //     if function.verify(true) {
    //         self.fpm.run_on(&function);

    //         Ok(function)
    //     } else {
    //         unsafe {
    //             function.delete();
    //         }

    //         Err("Invalid generated function.")
    //     }
    // }

    /// Compiles the specified `Function` in the given `Context` and using the specified `Builder`, `PassManager`, and `Module`.
    pub fn compile(
        context: &'ctx Context,
        builder: &'a Builder<'ctx>,
        pass_manager: &'a PassManager<FunctionValue<'ctx>>,
        module: &'a Module<'ctx>,
        expr: &Expr,
        global_scope: &'a mut HashMap<String, PointerValue<'ctx>>,
    ) -> Result<FloatValue<'ctx>, &'static str> {
        let mut compiler = Compiler {
            context,
            builder,
            fpm: pass_manager,
            module,
            expr,
            global_scope, // scopes: vec![global_scope],
            fn_value_opt: None,
        };
        // Directly call the modified compile_expr method
        compiler.compile_expr(expr)
    }
}
