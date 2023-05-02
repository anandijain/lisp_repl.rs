
                // println!("variables: {:?}", self.variables);
                    // let lhs = self.compile_expr(left)?;
                    // let rhs = self.compile_expr(right)?;
                    // "+" => compiled_args
                    //     .into_iter()
                    //     .reduce(|lhs, rhs| self.builder.build_float_add(lhs, rhs, "tmpadd")),
                    // "-" => compiled_args
                    //     .into_iter()
                    //     .reduce(|lhs, rhs| self.builder.build_float_sub(lhs, rhs, "tmpsub")),
                    // "*" => compiled_args
                    //     .into_iter()
                    //     .reduce(|lhs, rhs| self.builder.build_float_mul(lhs, rhs, "tmpmul")),
                    // "/" => compiled_args
                    //     .into_iter()
                    //     .reduce(|lhs, rhs| self.builder.build_float_div(lhs, rhs, "tmpdiv")),
                    // "%" => compiled_args
                    //     .into_iter()
                    //     .reduce(|lhs, rhs| self.builder.build_float_rem(lhs, rhs, "tmprem")),
                // let result = match args {
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
                        "%" => compiled_args
                            .into_iter()
                            .reduce(|lhs, rhs| self.builder.build_float_rem(lhs, rhs, "tmprem")),
                //         // "define" => {
                //         //     match &exprs[1] {
                //         //         Expr::List(fn_exprs) => {
                //         //             // Define a new function with the given arguments
                //         //             let fn_name = match &fn_exprs[0] {
                //         //                 Expr::Symbol(name) => name,
                //         //                 _ => return Err("Expected function name as a symbol"),
                //         //             };
                //         //             if let Some(old_fn) = self.functions.get(fn_name) {
                //         //                 return Err("Function already exists with that name");
                //         //             }
                //         //             let fn_args =
                //         //                 fn_exprs[1..]
                //         //                     .iter()
                //         //                     .enumerate()
                //         //                     .map(|(i, expr)| {
                //         //                         match expr {
                //         //                             Expr::Symbol(name) => {
                //         //                                 let variable = self.builder.build_alloca(
                //         //                                     self.context.f64_type(),
                //         //                                     &format!("arg_{}_{}", fn_name, i),
                //         //                                 );
                //         //                                 self.variables
                //         //                                     .insert(name.to_owned(), variable);
                //         //                                 (name.to_owned(), i)
                //         //                             }
                //         //                             _ => return Err(
                //         //                                 "Expected function argument to be a symbol",
                //         //                             ),
                //         //                         }
                //         //                     })
                //         //                     .collect::<Vec<_>>();
                //         //             let fn_body_expr = exprs[2].clone();
                //         //             let result = self.compile_expr(&fn_body_expr)?;
                //         //             for (arg_name, _) in fn_args {
                //         //                 self.variables.remove(&arg_name);
                //         //             }
                //         //             let func = compile_func(&self.context, fn_name, fn_args.len());
                //         //             self.functions.insert(fn_name.clone(), func);
                //         //             Some(self.context.f64_type().const_float(0.0))
                //         //         }
                //         //         _ => {
                //         //             // Define a new variable
                //         //             if let Expr::Symbol(var_name) = exprs[1] {
                //         //                 if self.variables.contains_key(var_name) {
                //         //                     return Err(
                //         //                         "A variable already exists with that name.",
                //         //                     );
                //         //                 }
                //         //                 let var_value = self.compile_expr(&exprs[2])?;
                //         //                 let var_ptr = var_ptr(
                //         //                     &mut self.builder,
                //         //                     self.context.f64_type(),
                //         //                     var_name,
                //         //                 );
                //         //                 self.builder.build_store(var_ptr, var_value);
                //         //                 self.variables.insert(var_name.to_owned(), var_ptr);
                //         //             }
                //         //             Some(self.context.f64_type().const_float(0.0))
                //         //         }
                //         //     }
                //         // }
                //         _ => {
                //             if let Some(intrinsic) = inkwell::intrinsics::Intrinsic::find(op) {
                //                 let arg_types: Vec<inkwell::types::BasicTypeEnum> = compiled_args
                //                     .clone()
                //                     .iter()
                //                     .map(|arg| arg.get_type().into())
                //                     .collect();

                //                 let args_metadata: Vec<inkwell::values::BasicMetadataValueEnum> =
                //                     compiled_args
                //                         .clone()
                //                         .into_iter()
                //                         .map(|arg| arg.into())
                //                         .collect();

                //                 let intrinsic_function = match intrinsic
                //                     .get_declaration(&self.module, arg_types.as_slice())
                //                 {
                //                     Some(function) => function,
                //                     None => {
                //                         return Err(
                //                             "Found Intrinsic, but could not create FunctionValue for the intrinsic.",
                //                         );
                //                     }
                //                 };

                //                 let ret_val = self
                //                     .builder
                //                     .build_call(
                //                         intrinsic_function,
                //                         args_metadata.as_slice(),
                //                         "call",
                //                     )
                //                     .try_as_basic_value()
                //                     .left()
                //                     .unwrap();

                //                 let fv = ret_val.into_float_value();
                //                 println!("yooooo");
                //                 fv.print_to_stderr();
                //                 Some(fv.into())
                //             } else {
                //                 return Err("Unknown operator/intrinsic");
                //             }
                //         }
                //     },
                //     Err(err) => {
                //         return Err(err);
                //     }
                // };
                // println!("result: {:?}", result);

                // match result {
                //     Some(res) => Ok(res),
                //     None => Err("Insufficient arguments for the operator."),
                // }


    /// Compiles the specified `Prototype` into an extern LLVM `FunctionValue`.
    // fn compile_prototype(&self, expr: &Expr) -> Result<(FunctionValue<'ctx>, bool), &'static str> {
    //     let ret_type = self.context.f64_type();

    //     let (fn_name, fn_args, is_anonymous) =
    //         match *expr {
    //             Expr::List(ref exprs) => {
    //                 let (op, args) = match exprs.split_first() {
    //                     Some((Expr::Symbol(op), args)) => (op.as_str(), args),
    //                     _ => return Err("expected operator"),
    //                 };

    //                 match op {
    //                     "define" => {
    //                         let (name_expr, rest_args) = match args.split_first() {
    //                             Some((name_expr, rest_args)) => (name_expr, rest_args),
    //                             _ => return Err("expected function name"),
    //                         };

    //                         match name_expr {
    //                             Expr::Symbol(name) => {
    //                                 // Variable definition case: (define x 5)
    //                                 ("anonymous".to_string(), vec![], true)
    //                             }
    //                             Expr::List(name_and_args) => {
    //                                 // Function definition case: (define (square x) (* x x))
    //                                 let (name_expr, args) = match name_and_args.split_first() {
    //                                     Some((name_expr, args)) => (name_expr, args),
    //                                     _ => return Err("expected function name and arguments"),
    //                                 };

    //                                 let fn_name = match name_expr {
    //                                     Expr::Symbol(name) => name.clone(),
    //                                     _ => return Err("expected function name to be a symbol"),
    //                                 };

    //                                 let fn_args: Result<Vec<String>, &'static str> = args
    //                                     .iter()
    //                                     .map(|arg| {
    //                                         if let Expr::Symbol(ref arg_name) = arg {
    //                                             Ok(arg_name.clone())
    //                                         } else {
    //                                             Err("expected function arguments to be symbols")
    //                                         }
    //                                     })
    //                                     .collect();

    //                                 let fn_args = match fn_args {
    //                                     Ok(args) => args,
    //                                     Err(err) => return Err(err),
    //                                 };

    //                                 (fn_name, fn_args, false)
    //                             }
    //                             _ => return Err(
    //                                 "expected a symbol or a list with function name and arguments",
    //                             ),
    //                         }
    //                     }
    //                     _ => ("anonymous".to_string(), vec![], true),
    //                 }
    //             }
    //             _ => ("anonymous".to_string(), vec![], true),
    //         };

    //     // Create argument types for the function
    //     let args_types = std::iter::repeat(ret_type)
    //         .take(fn_args.len())
    //         .map(|f| f.into())
    //         .collect::<Vec<BasicMetadataTypeEnum>>();
    //     let args_types = args_types.as_slice();

    //     // Create the function type and value
    //     let fn_type = self.context.f64_type().fn_type(args_types, false);
    //     let fn_val = self.module.add_function(fn_name.as_str(), fn_type, None);

    //     // Set the argument names
    //     for (i, arg) in fn_val.get_param_iter().enumerate() {
    //         arg.into_float_value().set_name(fn_args[i].as_str());
    //     }

    //     Ok((fn_val, is_anonymous))
    // }


    // /// Compiles the specified `Function` into an LLVM `FunctionValue`.

    // fn compile_fn(&mut self) -> Result<FunctionValue<'ctx>, &'static str> {
    //     let proto = self.expr;
    //     println!("in compile_fn compiling fn: {:?}", proto);
    //     println!("in compile_fn compiling fn: {:?}", proto);
    //     let (function, is_anonymous) = self.compile_prototype(proto)?;
    
    //     if is_anonymous {
    //         return Ok(function);
    //     }
    
    //     let entry = self.context.append_basic_block(function, "entry");
    
    //     self.builder.position_at_end(entry);
    
    //     // update fn field
    //     self.fn_value_opt = Some(function);
    
    //     let args_list = match proto {
    //         Expr::List(ref exprs) => {
    //             let (op, args) = extract_op_and_args(exprs)?;

    //         }
    //         _ => return Err("Invalid prototype. Expected a list with function name and arguments."),
    //     };
    
    //     // build variables map
    //     self.variables.reserve(args_list.len());
    
    //     for (i, arg) in function.get_param_iter().enumerate() {
    //         let arg_name = match args_list[i] {
    //             Expr::Symbol(ref name) => name,
    //             _ => return Err("Expected function arguments to be symbols."),
    //         };
    //         let alloca = self.create_entry_block_alloca(arg_name.as_str());
    
    //         self.builder.build_store(alloca, arg);
    
    //         self.variables.insert(arg_name.clone(), alloca);
    //     }
    
    //     // compile body

    //     let body = self.compile_expr(self.expr[2].as_ref().unwrap())?;
    
    //     self.builder.build_return(Some(&body));
    
    //     // return the whole function after verification and optimization
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
  