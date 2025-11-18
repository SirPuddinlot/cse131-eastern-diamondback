// src/main.rs
mod ast;
mod instr;
mod parser;
mod compiler;
mod jit;
mod repl;
mod helpers;
mod typechecker;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use im::HashMap;
use dynasmrt::*;
use capstone::prelude::*;

use crate::compiler::FunContext;
use crate::compiler::get_input_heap_offset;
use crate::instr::Instr;
use crate::jit::compile_functions_only;
use crate::compiler::compile;
use crate::parser::parse_program;
use crate::helpers::*;
use crate::repl::run_repl;
use crate::typechecker::*;
use crate::ast::*;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <flag> <input.snek> [output.s/input]", args[0]);
        eprintln!("Flags:");
        eprintln!("  -c: Compile to assembly file (requires output file)");
        eprintln!("  -e: Execute directly using JIT compilation");
        eprintln!("  -g: Do both - execute and generate assembly");
        eprintln!("  -i: Interactive REPL mode");
        eprintln!("  -t: Typecheck only and print type");
        eprintln!("  -tc: Typecheck and compile to assembly");
        eprintln!("  -te: Typecheck and execute with JIT");
        eprintln!("  -tg: Typecheck and do both (execute + generate)");
        eprintln!("  -ti: Interactive REPL with typechecking");
        std::process::exit(1);
    }

    let flag = &args[1];
    
    // Handle REPL modes
    match flag.as_str() {
        "-i" => {
            return run_repl(false); // No typechecking
        }
        "-ti" => {
            return run_repl(true); // With typechecking
        }
        _ => {}
    }

    if args.len() < 3 {
        eprintln!("Usage: {} <flag> <input.snek> [output.s/input]", args[0]);
        std::process::exit(1);
    }

    let in_name = &args[2];
    
    let mut in_file = File::open(in_name)?;
    let mut in_contents = String::new();
    in_file.read_to_string(&mut in_contents)?;
    
    // Trim leading/trailing whitespace
    let trimmed = in_contents.trim();

    // Detect if it starts and ends with '(' … ')' (i.e., already a top-level list)
    let wrapped_source = 
        if trimmed.starts_with("((") && trimmed.ends_with("))") {
            trimmed.to_string()
        } 
        else {
            format!("({})", trimmed)
        };    

    let sexp = sexp::parse(&wrapped_source).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Parse error: {}", e))
    })?;

    let prog = parse_program(&sexp);

    // Determine if we're in typecheck mode and get input type if needed
    let typecheck_mode = flag.starts_with("-t");
    
    if typecheck_mode {
        if typecheck_mode {
            let input_type = match flag.as_str() {
                "-t" | "-tc" => {
                    // No input provided, input has type Any
                    None
                }
                "-te" => {
                    // For -te: input is at args[3]
                    let input_str = if args.len() > 3 { &args[3] } else { "false" };
                    let input = parse_input(input_str);
                    Some(if input & 1 == 0 { Type::Num } else { Type::Bool })
                }
                "-tg" => {
                    // For -tg: format is -tg <prog>.snek <prog>.s <input>
                    // So input is at args[4]
                    let input_str = if args.len() > 4 { &args[4] } else { "false" };
                    let input = parse_input(input_str);
                    Some(if input & 1 == 0 { Type::Num } else { Type::Bool })
                }
                _ => None
            };
        
        // Run typechecker
        match typecheck_program(&prog, input_type) {
            Ok(t) => {
                if flag == "-t" {
                    // Just print the type and exit
                    println!("{:?}", t);
                    return Ok(());
                }
                // For other -t* flags, continue to compilation/execution
            }
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
       }
    }
    
    match flag.as_str() {
        "-c" | "-tc" => {
            // AOT compilation only
            if args.len() < 4 {
                eprintln!("Error: output file required for -c/-tc");
                std::process::exit(1);
            }
            let out_name = &args[3];
            let result = compile(&prog);
            let asm_program = format!(
                "section .text
global our_code_starts_here
extern snek_error
extern _snek_print

{}",
                result
            );
            let mut out_file = File::create(out_name)?;
            out_file.write_all(asm_program.as_bytes())?;
        }
        "-e" | "-te" => {
            // JIT execution
            let input_str = if args.len() > 3 { &args[3] } else { "false" };
            let input = parse_input(input_str);
            
            let mut __ops__ = dynasmrt::x64::Assembler::new().unwrap();
            let heap: Vec<i64> = vec![0; 128 * 1024];
            let heap_ptr = heap.as_ptr() as i64;
            
            let mut __fun_ctx__ = FunContext::new(&prog.defns);
            let mut __label_map__ = std::collections::HashMap::new();
            
            // Compile function definitions and error handlers
            compile_functions_only(&prog, &mut __ops__, &mut HashMap::new(), &mut __fun_ctx__, &mut __label_map__);
            
            // Capture the offset - this is where main starts
            let start = __ops__.offset();
            
            // Set up heap pointer
            dynasm!(__ops__
                ; .arch x64
                ; push rbp
                ; mov rbp, rsp
                ; mov r15, QWORD heap_ptr as _
            );
            
            // Store input
            let input_heap_offset = get_input_heap_offset();
            dynasm!(__ops__
                ; .arch x64
                ; mov [r15 + input_heap_offset], rdi
            );
            
            // Compile main
            let (main_instrs, min_offset) = crate::compiler::compile_to_instrs(
                &prog.main,
                -8,
                &HashMap::new(),
                &mut HashMap::new(),
                &mut __fun_ctx__,
                true,
                &None,
            );
            
            // Allocate stack if needed
            if min_offset <= -16 {
                let needed = -min_offset - 8;
                let stack_space = ((needed + 15) / 16) * 16;
                dynasm!(__ops__; .arch x64; sub rsp, stack_space as i32);
            }
            
            // Pre-create labels for main
            for instr in &main_instrs {
                if let Instr::ILabel(label_name) = instr {
                    if !__label_map__.contains_key(label_name) {
                        __label_map__.insert(label_name.clone(), __ops__.new_dynamic_label());
                    }
                }
                match instr {
                    Instr::IJmp(l) | Instr::IJe(l) | Instr::IJne(l) | Instr::IJo(l) => {
                        if !__label_map__.contains_key(l) {
                            __label_map__.insert(l.clone(), __ops__.new_dynamic_label());
                        }
                    }
                    _ => {}
                }
            }
            
            // Emit main instructions
            for instr in &main_instrs {
                crate::jit::instr_to_dynasm(instr, &mut __ops__, &__label_map__);
            }
            
            // Epilogue
            dynasm!(__ops__
                ; .arch x64
                ; mov rsp, rbp
                ; pop rbp
                ; ret
            );
            
            let buf = __ops__.finalize().unwrap();
            
            let jitted_fn: extern "C" fn(i64) -> i64 = unsafe { std::mem::transmute(buf.ptr(start)) };
            let result_val = jitted_fn(input);
            
            std::mem::forget(heap);
            print_result(result_val);
        }
        "-g" | "-tg" => {
            // Both: JIT execution and write assembly to file
            // Format: -g/-tg <prog>.snek <prog>.s <input>
            if args.len() < 4 {
                eprintln!("Error: output file required for -g/-tg");
                std::process::exit(1);
            }

            let out_name = &args[3];
            let input_str = if args.len() > 4 { &args[4] } else { "false" };
            let input = parse_input(input_str);

            // === JIT COMPILATION AND EXECUTION ===
            let mut __ops__ = dynasmrt::x64::Assembler::new().unwrap();
            let heap: Vec<i64> = vec![0; 128 * 1024];
            let heap_ptr = heap.as_ptr() as i64;

            let mut __fun_ctx__ = FunContext::new(&prog.defns);
            let mut __label_map__ = std::collections::HashMap::new();

            // Compile function definitions and error handlers
            compile_functions_only(&prog, &mut __ops__, &mut HashMap::new(), &mut __fun_ctx__, &mut __label_map__);

            // Capture the offset - this is where main starts
            let start = __ops__.offset();

            // Set up heap pointer and function prologue
            dynasm!(__ops__
                ; .arch x64
                ; push rbp
                ; mov rbp, rsp
                ; mov r15, QWORD heap_ptr as _
            );

            // Store input to heap
            let input_heap_offset = get_input_heap_offset();
            dynasm!(__ops__
                ; .arch x64
                ; mov [r15 + input_heap_offset], rdi
            );

            // Compile main expression
            let (main_instrs, min_offset) = crate::compiler::compile_to_instrs(
                &prog.main,
                -8,
                &HashMap::new(),
                &mut HashMap::new(),
                &mut __fun_ctx__,
                true,
                &None,
            );

            // Allocate stack space if needed
            if min_offset <= -16 {
                let needed = -min_offset - 8;
                let stack_space = ((needed + 15) / 16) * 16;
                dynasm!(__ops__; .arch x64; sub rsp, stack_space as i32);
            }

            // Pre-create labels for main
            for instr in &main_instrs {
                if let Instr::ILabel(label_name) = instr {
                    if !__label_map__.contains_key(label_name) {
                        __label_map__.insert(label_name.clone(), __ops__.new_dynamic_label());
                    }
                }
                match instr {
                    Instr::IJmp(l) | Instr::IJe(l) | Instr::IJne(l) | Instr::IJo(l) => {
                        if !__label_map__.contains_key(l) {
                            __label_map__.insert(l.clone(), __ops__.new_dynamic_label());
                        }
                    }
                    _ => {}
                }
            }

            // Emit main instructions
            for instr in &main_instrs {
                crate::jit::instr_to_dynasm(instr, &mut __ops__, &__label_map__);
            }

            // Main epilogue
            dynasm!(__ops__
                ; .arch x64
                ; mov rsp, rbp
                ; pop rbp
                ; ret
            );

            // Execute JIT
            let buf = __ops__.finalize().unwrap();
            let jitted_fn: extern "C" fn(i64) -> i64 = unsafe { std::mem::transmute(buf.ptr(start)) };
            let result_val = jitted_fn(input);

            std::mem::forget(heap);

            println!("JIT Result: ");
            print_result(result_val);

            // === AOT COMPILATION OUTPUT ===
            let result = compile(&prog);
            let asm_program = format!(
                "section .text
global our_code_starts_here
extern snek_error
extern _snek_print

{}",
                result
            );
            println!("\n=== Generated Assembly ===");
            println!("{}", asm_program);
            let mut out_file = File::create(out_name)?;
            out_file.write_all(asm_program.as_bytes())?;

            println!("Assembly written to: {}", out_name);
        }
        _ => {
            eprintln!("Error: Unknown flag '{}'", flag);
            std::process::exit(1);
        }
    }

    Ok(())
}