use inkwell::context::Context;
use inkwell::intrinsics::Intrinsic;
use inkwell::passes::PassManager;
use inkwell::OptimizationLevel;
use lisp_repl::*;
use rustyline::error::ReadlineError;
use rustyline::history::History;
use rustyline::validate::MatchingBracketValidator;
use rustyline::{Cmd, Editor, EventHandler, KeyCode, KeyEvent, Modifiers};
use rustyline::{Completer, Helper, Highlighter, Hinter, Validator};
use std::collections::HashMap;
use std::path::Path;

#[derive(Completer, Helper, Highlighter, Hinter, Validator)]
struct InputValidator {
    #[rustyline(Validator)]
    brackets: MatchingBracketValidator,
}

fn main() -> Result<(), ReadlineError> {
    let mut display_parser_output = true;
    let mut display_compiler_output = false;

    for arg in std::env::args() {
        match arg.as_str() {
            "--dp" => display_parser_output = true,
            "--dc" => display_compiler_output = true,
            _ => (),
        }
    }

    println!("p{} c{}", display_parser_output, display_compiler_output);

    let h = InputValidator {
        brackets: MatchingBracketValidator::new(),
    };

    let mut rl = Editor::new()?;
    rl.set_helper(Some(h));

    rl.bind_sequence(
        KeyEvent(KeyCode::Enter, Modifiers::ALT),
        EventHandler::Simple(Cmd::Newline),
    );

    let history_path = "history.txt";

    // Load history from the history file if it exists
    if Path::new(history_path).exists() {
        rl.load_history(history_path)?;
    }

    let context = Context::create();
    let module = context.create_module("repl");
    let builder = context.create_builder();
    let fpm = PassManager::create(&module);
    fpm.add_instruction_combining_pass();
    fpm.add_reassociate_pass();
    fpm.add_gvn_pass();
    fpm.add_cfg_simplification_pass();
    fpm.add_basic_alias_analysis_pass();
    fpm.add_promote_memory_to_register_pass();
    fpm.add_instruction_combining_pass();
    fpm.add_reassociate_pass();
    fpm.initialize();
    let mut global_scope = HashMap::new();

    let mut previous_exprs = Vec::new();

    let result = Compiler::compile(
        &context,
        &builder,
        &fpm,
        &module,
        &read("(define (square x) (* x x))").unwrap(),
        &mut global_scope,
    );
    println!("{:?}\n\n", result);
    println!("{}", module.print_to_string().to_string());

    let result2 = Compiler::compile(
        &context,
        &builder,
        &fpm,
        &module,
        &read("(square 2)").unwrap(),
        &mut global_scope,
    );
    
    let result3 = Compiler::compile(
        &context,
        &builder,
        &fpm,
        &module,
        &read("(llvm.abs -2)").unwrap(),
        &mut global_scope,
    );

    let ee = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .unwrap();

    println!("ee: {:#?}", ee);
    let sq = unsafe {
        ee.get_function::<unsafe extern "C" fn(f64) -> f64>("square")
            .ok()
    }
    .unwrap();
    let maybe_fn = unsafe { ee.get_function::<unsafe extern "C" fn() -> f64>("anon") }.unwrap();

    // todo figure out the name mangling
    let maybe_fn2 = unsafe { ee.get_function::<unsafe extern "C" fn() -> f64>("anon.1") }.unwrap();

    unsafe {
        println!(
            "YO YO YO THE EXECUTION ENGINE GOT: {:?}\n\n and {:?}",
            maybe_fn.call(),
            maybe_fn2.call()
        );
    }
    println!("{:#?}", module.get_functions().collect::<Vec<_>>());

    // let x = Intrinsic::find("llvm.abs");


    // let maybe_fn = unsafe { ee.get_function::<unsafe extern "C" fn(f64) -> f64>("name") };
    // let compiled_fn = match maybe_fn {
    // Ok(f) => f,
    // Err(err) => panic!()
    // Err("")
    // println!("!> Error during execution: {:?}", err);
    // continue;
    // }
    // };

    unsafe {
        println!("=> {}", sq.call(3.0));
    }

    // println!(r"{}", module.to_string());
    // println!("{:#?}", module
    // module.
    // let module_string = "; ModuleID = 'repl'\nsource_filename = \"repl\"\n\ndefine double @square(double %x) {\nentry:\n  %x1 = alloca double, align 8\n  store double %x, double* %x1, align 8\n}\n";

    // println!("{}", module_string);
    loop {
        let prompt_str = format! {"mylisp[%{}]>> ", rl.history().len().to_string()};

        let readline = rl.readline(prompt_str.as_str());
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                match read(&line) {
                    Ok(expr) => {
                        // for (index, prev) in previous_exprs.iter().enumerate() {
                        //     println!("{}: {:?}", index, prev);

                        //     Compiler::compile(&context, &builder, &module, prev)
                        //         .expect("Cannot re-add previously compiled function.");
                        // }
                        // compiler.expr = &expr.clone();
                        // compiler.compile_expr(&expr);

                        previous_exprs.push(expr.clone());
                        if display_parser_output {
                            println!("{:?}", expr);
                        }
                        // println!("gs{:?}", compiler.global_scope.clone());

                        // match compiler.compile_expr(&expr.to_owned()) {
                        // match Compiler::compile(
                        //     &context,
                        //     &builder,
                        //     &fpm,
                        //     &module,
                        //     &expr,
                        //     &mut compiler.global_scope.clone(),
                        // )
                        match Compiler::compile(
                            &context,
                            &builder,
                            &fpm,
                            &module,
                            &expr,
                            &mut global_scope,
                        ) {
                            Ok(result) => {
                                // if is_anonymous {
                                // let ee = module
                                //     .create_jit_execution_engine(OptimizationLevel::None)
                                //     .unwrap();

                                // let maybe_fn = unsafe {
                                //     ee.get_function::<unsafe extern "C" fn() -> f64>("anonymous")
                                // };

                                // let compiled_fn = match maybe_fn {
                                //     Ok(f) => f,
                                //     Err(err) => {
                                //         println!("!> Error during execution: {:?}", err);
                                //         continue;
                                //     }
                                // };

                                // unsafe {
                                //     println!("CALL=> {}", compiled_fn.call());
                                // }
                                // }
                                // println!("{}\n\n{}", result, module.print_to_string());
                                println!("{}", module.to_string());
                                println!("{:?}\n\n", result);
                            }

                            Err(err) => println!("Error: {}", err),
                        }
                    }
                    Err(err) => println!("Error: {}", err),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
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
