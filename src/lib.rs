#[warn(unused_imports)]
use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    passes::PassManager,
    types::BasicMetadataTypeEnum,
    values::{BasicMetadataValueEnum, FloatValue, FunctionValue, PointerValue},
};
use peg::parser;
use std::{
    collections::HashMap,
    fmt,
    num::{ParseFloatError, ParseIntError},
};

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
                    // println!("self.global_scope: {:?}", self.global_scope);

                    Ok(self
                        .builder
                        .build_load(*var, name.as_str())
                        .into_float_value())
                }
                None => {
                    // println!("self.global_scope: {:?}", self.global_scope);
                    Err("Could not find a matching variable.")
                }
            },
            Expr::List(ref exprs) => {
                let (op, args) = extract_op_and_args(exprs)?;
                // println!("{op}({:?})", args);
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

                                let body = self.compile_expr(&args[1])?;

                                Ok(body)
                            } else {
                                Err("Function definition should start with a symbol for its name.")
                            }
                        } else {
                            // println!("defining a variable ");
                            // Handle the define variable case
                            if let Expr::Symbol(var_name) = &args[0] {
                                let value = self.compile_expr(&args[1])?;
                                let alloca = self.create_entry_block_alloca(var_name);
                                self.builder.build_store(alloca, value);
                                self.global_scope.insert(var_name.clone(), alloca);
                                Ok(value)
                            } else {
                                Err("define requires a variable name to be a symbol.")
                            }
                        }
                    }
                    // i think in all the cases below this we want to compile a prototype and anonymous function with zero args.
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
                                            None => Err(
                                                "Error: Addition requires at least one argument.",
                                            ),
                                        }
                                    }
                                    "-" => {
                                        match compiled_args.into_iter().reduce(|lhs, rhs| {
                                            self.builder.build_float_sub(lhs, rhs, "tmpsub")
                                        }) {
                                            Some(result) => Ok(result),
                                            None => Err("Error: Subtraction requires at least one argument."),
                                        }
                                    }
                                    "*" => {
                                        match compiled_args.into_iter().reduce(|lhs, rhs| {
                                            self.builder.build_float_mul(lhs, rhs, "tmpmul")
                                        }) {
                                            Some(result) => Ok(result),
                                            None => Err("Error: Multiplication requires at least one argument."),
                                        }
                                    }
                                    "/" => {
                                        match compiled_args.into_iter().reduce(|lhs, rhs| {
                                            self.builder.build_float_div(lhs, rhs, "tmpdiv")
                                        }) {
                                            Some(result) => Ok(result),
                                            None => Err(
                                                "Error: Division requires at least one argument.",
                                            ),
                                        }
                                    }
                                    _ => {
                                        match self.module.get_function(op) {
                                            Some(f) => {
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

                                                Ok(body)
                                            }
                                            None => {
                                                let intrinsic =
                                                    inkwell::intrinsics::Intrinsic::find(op);
                                                if let Some(found_intrinsic) = intrinsic {
                                                    // Get double type from the context
                                                    let double_type = self.context.f64_type();

                                                    // Get the function declaration from the intrinsic
                                                    let intrinsic_function = found_intrinsic
                                                        .get_declaration(
                                                            &self.module,
                                                            vec![double_type.into(); args.len()]
                                                                .as_slice(),
                                                        )
                                                        .unwrap();

                                                    // // Process the compiled arguments
                                                    let mut compiled_args = vec![];
                                                    for arg in args.iter() {
                                                        let foo =
                                                            BasicMetadataValueEnum::FloatValue(
                                                                self.compile_expr(arg).unwrap(),
                                                            );
                                                        compiled_args.push(foo);
                                                    }

                                                    // Perform the intrinsic function call
                                                    let body = self
                                                        .builder
                                                        .build_call(
                                                            intrinsic_function,
                                                            compiled_args.as_slice(),
                                                            "intrinsic_call",
                                                        )
                                                        .try_as_basic_value()
                                                        .left()
                                                        .unwrap()
                                                        .into_float_value();

                                                    Ok(body)
                                                } else {
                                                    Err("no function found with that name")
                                                }
                                            }
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

    /// Compiles the specified `Function` into an LLVM `FunctionValue`.
    fn compile_fn(&mut self) -> Result<FunctionValue<'ctx>, &'static str> {
        let (op, args) = match self.expr {
            // (define (square x) (* x x)))
            Expr::List(exs) => {
                let (op, args) = extract_op_and_args(exs)?;
                match op {
                    "define" => {
                        let sig = &args.clone()[0];
                        match sig { // (square x) or just x 
                            Expr::List(sig_exs) => { // this arm is for function definition (square x)
                                let (fn_name, fn_args) = extract_op_and_args(&sig_exs)?;
                                // Extract arg_names and validate
                                let mut arg_names: Vec<String> = vec![];
                                for arg in fn_args.iter() {
                                    match arg {
                                        Expr::Symbol(s) => arg_names.push(s.to_string()),
                                        _ => return Err("in define: all the elements in the argument list must be symbols"),
                                    }
                                }
                                (fn_name, arg_names)
                            }
                            Expr::Symbol(s) => ("anon", vec![].into()),
                            _ => panic!("in define: the first element in the argument list must be a symbol"),
                        }
                    }
                    _ => ("anon", vec![].into()),
                }
            }
            _ => ("anon", vec![].into()),
        };

        let function = self.compile_prototype(op, args.clone())?;

        let entry = self.context.append_basic_block(function, "entry");

        self.builder.position_at_end(entry);

        // update fn field
        self.fn_value_opt = Some(function);

        // build variables map
        self.global_scope.reserve(args.len());

        for (i, arg) in function.get_param_iter().enumerate() {
            let arg_name = args[i].as_str();
            let alloca = self.create_entry_block_alloca(arg_name);

            self.builder.build_store(alloca, arg);

            self.global_scope.insert(args[i].clone(), alloca);
        }

        // let tmp = self.global_scope.clear();

        // for (k, v) in self.global_scope.iter() {
        //     // let arg_name = args[i].as_str();
        //     // let alloca = self.create_entry_block_alloca(k);
        //     v.
        //     self.builder.build_store(alloca, v.into());
        // }

        // compile body
        let body = self.compile_expr(self.expr)?;

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

    /// Compiles the specified `Function` in the given `Context` and using the specified `Builder`, `PassManager`, and `Module`.
    pub fn compile(
        context: &'ctx Context,
        builder: &'a Builder<'ctx>,
        pass_manager: &'a PassManager<FunctionValue<'ctx>>,
        module: &'a Module<'ctx>,
        expr: &Expr,
        global_scope: &'a mut HashMap<String, PointerValue<'ctx>>,
    ) -> Result<FunctionValue<'ctx>, &'static str> {
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
        compiler.compile_fn()
    }
}
