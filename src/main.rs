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
    // fpm.add_instruction_combining_pass();
    // fpm.add_reassociate_pass();
    // fpm.add_gvn_pass();
    // fpm.add_cfg_simplification_pass();
    // fpm.add_basic_alias_analysis_pass();
    // fpm.add_promote_memory_to_register_pass();
    // fpm.add_instruction_combining_pass();
    // fpm.add_reassociate_pass();
    // fpm.initialize();
    let mut global_scope = HashMap::new();

    let mut previous_exprs: Vec<Expr> = Vec::new();
    
    let ee = module
    .create_jit_execution_engine(OptimizationLevel::None)
    .unwrap();

    // let result = Compiler::compile(
    //     &context,
    //     &builder,
    //     &fpm,
    //     &module,
    //     &read("(define (square x) (* x x))").unwrap(),
    //     &mut global_scope,
    // );
    // println!("{:?}\n\n", result);
    // println!("{}", module.print_to_string().to_string());

    // let result2 = Compiler::compile(
    //     &context,
    //     &builder,
    //     &fpm,
    //     &module,
    //     &read("(square 2)").unwrap(),
    //     &mut global_scope,
    // );

    // let result3 = Compiler::compile(
    //     &context,
    //     &builder,
    //     &fpm,
    //     &module,
    //     &read("(llvm.fabs -2.5)").unwrap(),
    //     &mut global_scope,
    // );

    // let result4 = Compiler::compile(
    //     &context,
    //     &builder,
    //     &fpm,
    //     &module,
    //     &read("(+ 2 2)").unwrap(),
    //     &mut global_scope,
    // )
    // .unwrap();
    // println!("r4{:?}\n\n", result4);



    // let sq = unsafe {
    //     ee.get_function::<unsafe extern "C" fn(f64) -> f64>("square")
    //         .ok()
    // }
    // .unwrap();
    // let maybe_fn = unsafe { ee.get_function::<unsafe extern "C" fn() -> f64>("anon") }.unwrap();

    // // // todo figure out the name mangling
    // let maybe_fn2 = unsafe { ee.get_function::<unsafe extern "C" fn() -> f64>("anon.1") }.unwrap();

    // unsafe {
    //     println!(
    //         "YO YO YO THE EXECUTION ENGINE GOT: {:?}\n\n and {:?}",
    //         maybe_fn.call(),
    //         maybe_fn2.call()
    //     );
    // }
    // println!("{:#?}", module.get_functions().collect::<Vec<_>>());

    // unsafe {
    //     println!("=> {}", sq.call(3.0));
    // }

    let mut loop_counter = 0; // used for module name

    loop {
        let prompt_str = format! {"\x1b[1;32mmylisp[HIST:{} | LOOP: {}]>>\x1b[0m ", rl.history().len().to_string(),loop_counter};

        let readline = rl.readline(prompt_str.as_str());
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                if line.is_empty() {
                    continue;
                }
                match read(&line) {
                    Ok(expr) => {
                        let mod_name = format!("repl_{}", loop_counter);
                        let module = context.create_module(&mod_name);
                        loop_counter += 1;

                        // recompile every previously parsed function into the new module
                        // this clears the anon so we dont get anon.1 anon.2 etc
                        for prev in &previous_exprs {
                            Compiler::compile(
                                &context,
                                &builder,
                                &fpm,
                                &module,
                                prev,
                                &mut global_scope,
                            )
                            .expect("Cannot re-add previously compiled function.");
                        }

                        if display_parser_output {
                            println!("{:?}", expr);
                        }

                        match Compiler::compile(
                            &context,
                            &builder,
                            &fpm,
                            &module,
                            &expr,
                            &mut global_scope,
                        ) {
                            Ok(result) => {
                                println!(
                                    "GLOBAL_SCOPE:\n\n {:?}\n\nMODULE CONTENTS: \n\n{}",
                                    global_scope,
                                    module.to_string()
                                );

                                let function_name = result.get_name().to_str().unwrap();
                                // I assume this returns a &str

                                if function_name.contains("anon") {
                                    // previous_exprs.push(expr);
                                    let ee = module
                                        .create_jit_execution_engine(OptimizationLevel::None)
                                        .unwrap();
                                    let maybe_fn = unsafe {
                                        ee.get_function::<unsafe extern "C" fn() -> f64>(
                                            function_name,
                                        )
                                    };

                                    let compiled_fn = match maybe_fn {
                                        Ok(f) => f,
                                        Err(err) => {
                                            println!("!> Error during execution: {:?}", err);
                                            continue;
                                        }
                                    };
                                    
                                    unsafe {
                                        println!("about to call ");
                                        println!("CALL=> {}", compiled_fn.call());
                                    }
                                } else {
                                    previous_exprs.push(expr);

                                    println!("NON_ANON");
                                    println!("{:?}\n\n", result);
                                }
                            }

                            Err(err) => {
                                println!(
                                    "GLOBAL_SCOPE:\n\n {:#?}\n\nMODULE CONTENTS: \n\n{}",
                                    global_scope,
                                    module.to_string()
                                );

                                println!("Error: {}", err)
                            }
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
// is_x86_feature_detected!("avx2");