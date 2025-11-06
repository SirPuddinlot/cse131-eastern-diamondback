// src/repl.rs
use crate::helpers::*;
use std::io::{self, Write, BufRead};
use im::HashMap;
use std::mem;
use dynasmrt::{DynasmApi, dynasm, DynasmLabelApi};
use crate::ast::*;
use crate::parser::*;
use crate::jit::*;
use crate::compiler::FunContext;
use std::collections::HashMap as StdHashMap;
use crate::instr::Instr;

pub fn run_repl() -> io::Result<()> {
    let mut ops = dynasmrt::x64::Assembler::new().unwrap();
    let mut defines: HashMap<String, i32> = HashMap::new();
    
    // Allocate heap for define'd variables
    let mut heap: Vec<i64> = vec![0; 128 * 1024];
    let heap_ptr = heap.as_mut_ptr() as i64;
    
    // Track function definitions and their labels (persistent across prompts)
    let mut fun_defns: Vec<FunDefn> = Vec::new();
    let mut global_labels: StdHashMap<String, dynasmrt::DynamicLabel> = StdHashMap::new();
    
    // Get error handler addresses once
    let snek_error_addr = crate::snek_error as *const () as i64;
    let snek_print_addr = crate::_snek_print as *const () as i64;
    
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    
    loop {
        print!("> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        reader.read_line(&mut input)?;
        
        let input = input.trim();
        
        // Check for exit commands
        if input == "exit" || input == "quit" {
            break;
        }
        
        if input.is_empty() {
            continue;
        }
        
        // Parse the input
        let sexp = match sexp::parse(input) {
            Ok(s) => s,
            Err(_e) => {
                println!("Invalid: parse error");
                continue;
            }
        };
        
        // Parse into ReplEntry - catch panics from parser
        let entry = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            parse_repl_entry(&sexp, 0)
        })) {
            Ok(Ok(e)) => e,
            Ok(Err(msg)) => {
                println!("{}", msg);
                continue;
            }
            Err(_) => {
                println!("Invalid");
                continue;
            }
        };
        
        match entry {
            ReplEntry::Define(name, expr) => {
                // Check for duplicate definition
                if defines.contains_key(&name) {
                    println!("Duplicate binding");
                    continue;
                }
                
                // Create FunContext with all known functions
                let fun_ctx = FunContext::new(&fun_defns);
                
                // Compile the define using the proper function
                let (heap_offset, instrs) = match std::panic::catch_unwind(
                    std::panic::AssertUnwindSafe(|| {
                        crate::compiler::compile_define(&name, &expr, &defines, &fun_ctx)
                    })
                ) {
                    Ok(result) => result,
                    Err(_) => {
                        println!("Invalid");
                        continue;
                    }
                };
                
                let start = ops.offset();
                
                // Set up R15 to point to heap at the start of this code
                dynasm!(ops
                    ; .arch x64
                    ; push r15
                    ; mov r15, QWORD heap_ptr as _
                );
                
                // Create local labels for this segment
                let mut local_labels = global_labels.clone();
                
                // Create fresh error handler labels for THIS segment
                let snek_print_local = ops.new_dynamic_label();
                let error_overflow_local = ops.new_dynamic_label();
                let error_invalid_arg_local = ops.new_dynamic_label();
                local_labels.insert("_snek_print".to_string(), snek_print_local);
                local_labels.insert("error_overflow".to_string(), error_overflow_local);
                local_labels.insert("error_invalid_argument".to_string(), error_invalid_arg_local);
                
                // Pre-create labels from instructions
                for instr in &instrs {
                    if let Instr::ILabel(label_name) = instr {
                        if !local_labels.contains_key(label_name) {
                            local_labels.insert(label_name.clone(), ops.new_dynamic_label());
                        }
                    }
                    match instr {
                        Instr::IJmp(l) | Instr::IJe(l) | Instr::IJne(l) | Instr::IJo(l) => {
                            if !local_labels.contains_key(l) {
                                local_labels.insert(l.clone(), ops.new_dynamic_label());
                            }
                        }
                        _ => {}
                    }
                }
                
                // Emit the instructions
                for instr in &instrs {
                    crate::jit::instr_to_dynasm(instr, &mut ops, &local_labels);
                }
                
                // Normal exit path
                dynasm!(ops 
                    ; .arch x64 
                    ; pop r15
                    ; ret
                );
                
                // Emit error handlers AFTER the return
                dynasm!(ops
                    ; .arch x64
                    ; =>snek_print_local
                    ; push rbp
                    ; mov rbp, rsp
                    ; mov rax, QWORD snek_print_addr as _
                    ; call rax
                    ; pop rbp
                    ; ret
                );
                
                dynasm!(ops
                    ; .arch x64
                    ; =>error_overflow_local
                    ; pop r15  // Restore R15 before error
                    ; mov rdi, 1
                    ; mov rax, QWORD snek_error_addr as _
                    ; call rax
                    ; ret
                    ; =>error_invalid_arg_local
                    ; pop r15  // Restore R15 before error
                    ; mov rdi, 2
                    ; mov rax, QWORD snek_error_addr as _
                    ; call rax
                    ; ret
                );
                
                match ops.commit() {
                    Ok(_) => {}
                    Err(_) => {
                        println!("Invalid");
                        continue;
                    }
                }
                
                let reader = ops.reader();
                let buf = reader.lock();
                let jitted_fn: extern "C" fn() -> i32 = unsafe { mem::transmute(buf.ptr(start)) };
                jitted_fn(); // Execute to store the value
                
                // Store the heap offset
                defines = defines.update(name, heap_offset);
            }
            
            ReplEntry::Fun(name, params, body) => {
                // Check for duplicate function name
                if fun_defns.iter().any(|f| f.name == name) {
                    println!("Duplicate binding");
                    continue;
                }
                
                // Create the function definition
                let fun_defn = FunDefn {
                    name: name.clone(),
                    params: params.clone(),
                    body: Box::new(body.clone()),
                };
                
                // Create a persistent label for this function
                let fun_label = ops.new_dynamic_label();
                global_labels.insert(format!("fun_{}", name), fun_label);
                
                // Create FunContext with all known functions (including the new one)
                let mut all_defns = fun_defns.clone();
                all_defns.push(fun_defn.clone());
                let fun_ctx = FunContext::new(&all_defns);
                
                // Build environment: parameters are on caller's stack at [rbp+16], [rbp+24], etc.
                let mut env = HashMap::new();
                for (i, param) in params.iter().enumerate() {
                    let offset = 16 + (i as i32 * 8);
                    env = env.update(param.clone(), offset);
                }
                
                // Compile function body
                let (instrs, min_offset) = match std::panic::catch_unwind(
                    std::panic::AssertUnwindSafe(|| {
                        crate::compiler::compile_to_instrs(
                            &body,
                            -8,
                            &env,
                            &defines,
                            &fun_ctx,
                            false,
                            &None,
                        )
                    })
                ) {
                    Ok(result) => result,
                    Err(_) => {
                        println!("Invalid");
                        continue;
                    }
                };
                
                // Emit function prologue
                dynasm!(ops
                    ; .arch x64
                    ; =>fun_label
                    ; push rbp
                    ; mov rbp, rsp
                    ; push r15
                    ; mov r15, QWORD heap_ptr as _
                );
                
                // Allocate stack space if needed
                if min_offset < 0 {
                    let needed = -min_offset;
                    let stack_space = ((needed + 15) / 16) * 16;
                    dynasm!(ops
                        ; .arch x64
                        ; sub rsp, stack_space as i32
                    );
                }
                
                // Create local labels for this function
                let mut local_labels = global_labels.clone();
                for instr in &instrs {
                    if let Instr::ILabel(label_name) = instr {
                        if !local_labels.contains_key(label_name) {
                            local_labels.insert(label_name.clone(), ops.new_dynamic_label());
                        }
                    }
                    match instr {
                        Instr::IJmp(l) | Instr::IJe(l) | Instr::IJne(l) | Instr::IJo(l) => {
                            if !local_labels.contains_key(l) {
                                local_labels.insert(l.clone(), ops.new_dynamic_label());
                            }
                        }
                        _ => {}
                    }
                }
                
                // Emit function body instructions
                for instr in &instrs {
                    crate::jit::instr_to_dynasm(instr, &mut ops, &local_labels);
                }
                
                // Function epilogue
                dynasm!(ops
                    ; .arch x64
                    ; pop r15
                    ; mov rsp, rbp
                    ; pop rbp
                    ; ret
                );
                
                match ops.commit() {
                    Ok(_) => {}
                    Err(_) => {
                        println!("Invalid");
                        continue;
                    }
                }
                
                // Add function to our list
                fun_defns.push(fun_defn);
                println!("Function {} defined", name);
            }
            
            ReplEntry::Expr(expr) => {
                let start = ops.offset();
                
                // Set up R15 to point to heap
                dynasm!(ops
                    ; .arch x64
                    ; push r15
                    ; mov r15, QWORD heap_ptr as _
                );
                
                // Create FunContext with all known functions
                let fun_ctx = FunContext::new(&fun_defns);
                
                // Compile the expression
                let (instrs, min_offset) = match std::panic::catch_unwind(
                    std::panic::AssertUnwindSafe(|| {
                        crate::compiler::compile_to_instrs(
                            &expr,
                            -8,
                            &HashMap::new(),
                            &defines,
                            &fun_ctx,
                            true, // is_main
                            &None,
                        )
                    })
                ) {
                    Ok(result) => result,
                    Err(_) => {
                        println!("Invalid");
                        continue;
                    }
                };
                
                // Allocate stack space if needed
                if min_offset <= -16 {
                    let needed = -min_offset - 8;
                    let stack_space = ((needed + 15) / 16) * 16;
                    dynasm!(ops
                        ; .arch x64
                        ; sub rsp, stack_space as i32
                    );
                }
                
                // Create local labels for this segment
                let mut local_labels = global_labels.clone();
                
                // Create fresh error handler labels for THIS segment
                let snek_print_local = ops.new_dynamic_label();
                let error_overflow_local = ops.new_dynamic_label();
                let error_invalid_arg_local = ops.new_dynamic_label();
                local_labels.insert("_snek_print".to_string(), snek_print_local);
                local_labels.insert("error_overflow".to_string(), error_overflow_local);
                local_labels.insert("error_invalid_argument".to_string(), error_invalid_arg_local);
                
                // Pre-create labels from instructions
                for instr in &instrs {
                    if let Instr::ILabel(label_name) = instr {
                        if !local_labels.contains_key(label_name) {
                            local_labels.insert(label_name.clone(), ops.new_dynamic_label());
                        }
                    }
                    match instr {
                        Instr::IJmp(l) | Instr::IJe(l) | Instr::IJne(l) | Instr::IJo(l) => {
                            if !local_labels.contains_key(l) {
                                local_labels.insert(l.clone(), ops.new_dynamic_label());
                            }
                        }
                        _ => {}
                    }
                }
                
                // Emit instructions
                for instr in &instrs {
                    crate::jit::instr_to_dynasm(instr, &mut ops, &local_labels);
                }
                
                // Normal exit path - restore R15 and return
                dynasm!(ops 
                    ; .arch x64 
                    ; pop r15
                    ; ret
                );
                
                // Emit error handlers AFTER the return
                dynasm!(ops
                    ; .arch x64
                    ; =>snek_print_local
                    ; push rbp
                    ; mov rbp, rsp
                    ; mov rax, QWORD snek_print_addr as _
                    ; call rax
                    ; pop rbp
                    ; ret
                );
                
                dynasm!(ops
                    ; .arch x64
                    ; =>error_overflow_local
                    ; pop r15  // Restore R15 before error
                    ; mov rdi, 1
                    ; mov rax, QWORD snek_error_addr as _
                    ; call rax
                    ; ret
                    ; =>error_invalid_arg_local
                    ; pop r15  // Restore R15 before error
                    ; mov rdi, 2
                    ; mov rax, QWORD snek_error_addr as _
                    ; call rax
                    ; ret
                );
                
                match ops.commit() {
                    Ok(_) => {}
                    Err(_) => {
                        println!("Invalid");
                        continue;
                    }
                }
                
                let reader = ops.reader();
                let buf = reader.lock();
                let jitted_fn: extern "C" fn() -> i64 = unsafe { mem::transmute(buf.ptr(start)) };
                let result = jitted_fn();
                
                print_result(result);
            }
        }
    }
    
    Ok(())
}