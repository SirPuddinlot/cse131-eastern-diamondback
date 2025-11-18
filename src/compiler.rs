// src/compiler.rs
use im::HashMap;
use crate::ast::*;
use crate::instr::*;

static mut LABEL_COUNTER: i32 = 0;
static mut HEAP_OFFSET: i32 = 0;
static mut INPUT_HEAP_OFFSET: Option<i32> = None;

fn new_label(prefix: &str) -> String {
    unsafe {
        LABEL_COUNTER += 1;
        format!("{}_{}", prefix, LABEL_COUNTER)
    }
}

fn alloc_heap_slot() -> i32 {
    unsafe {
        let offset = HEAP_OFFSET;
        HEAP_OFFSET += 8;
        offset
    }
}

pub fn get_input_heap_offset() -> i32 {
    unsafe {
        if INPUT_HEAP_OFFSET.is_none() {
            INPUT_HEAP_OFFSET = Some(alloc_heap_slot());
        }
        INPUT_HEAP_OFFSET.unwrap()
    }
}

pub struct FunContext {
    pub functions: HashMap<String, FunDefn>,
}

impl FunContext {
    pub fn new(defns: &[FunDefn]) -> Self {
        let mut functions = HashMap::new();
        let mut seen = std::collections::HashSet::new();
        
        for defn in defns {
            if seen.contains(&defn.name) {
                panic!("Duplicate function definition: {}", defn.name);
            }
            seen.insert(defn.name.clone());
            functions = functions.update(defn.name.clone(), defn.clone());
        }
        
        FunContext { functions }
    }
    
    pub fn check_function_exists(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }
    
    pub fn get_param_count(&self, name: &str) -> usize {
        self.functions.get(name).map(|f| f.params.len()).unwrap_or(0)
    }
}

const TRUE_VAL: i32 = 1;
const FALSE_VAL: i32 = 3;

pub fn compile_to_instrs(
    e: &Expr,
    si: i32,
    env: &HashMap<String, i32>,
    defines: &HashMap<String, i32>,
    fun_ctx: &FunContext,
    input: bool,
    loop_end: &Option<String>,
) -> (Vec<Instr>, i32) {
    let mut code: Vec<Instr> = Vec::new();
    let mut current_min = si;

    match e {
        Expr::Number(n) => {
            code.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(*n << 1)));
        }
        Expr::Boolean(b) => {
            let val = if *b { TRUE_VAL } else { FALSE_VAL };
            code.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(val)));
        }
        Expr::Input => {
            let input_heap_offset = get_input_heap_offset();
            code.push(Instr::IMov(Val::Reg(Reg::RAX), Val::RegOffset(Reg::R15, input_heap_offset)));
        }
        Expr::Id(name) => {
            if let Some(&offset) = env.get(name) {
                code.push(Instr::IMov(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RBP, offset)));
            } else if let Some(&heap_offset) = defines.get(name) {
                code.push(Instr::IMov(Val::Reg(Reg::RAX), Val::RegOffset(Reg::R15, heap_offset)));
            } else {
                panic!("Unbound variable identifier {}", name);
            }
        }
        Expr::UnOp(op, expr) => {
            let (mut expr_code, expr_min) = compile_to_instrs(expr, si, env, defines, fun_ctx, input, loop_end);
            current_min = current_min.min(expr_min);
            code.append(&mut expr_code);

            match op {
                Op1::Add1 => {
                    code.push(Instr::ITest(Val::Reg(Reg::RAX), Val::Imm(1)));
                    code.push(Instr::IJne("error_invalid_argument".to_string()));
                    code.push(Instr::IAdd(Val::Reg(Reg::RAX), Val::Imm(1 << 1)));
                    code.push(Instr::IJo("error_overflow".to_string()));
                }
                Op1::Sub1 => {
                    code.push(Instr::ITest(Val::Reg(Reg::RAX), Val::Imm(1)));
                    code.push(Instr::IJne("error_invalid_argument".to_string()));
                    code.push(Instr::ISub(Val::Reg(Reg::RAX), Val::Imm(1 << 1)));
                    code.push(Instr::IJo("error_overflow".to_string()));
                }
                Op1::IsNum => {
                    code.push(Instr::ITest(Val::Reg(Reg::RAX), Val::Imm(1)));
                    code.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(FALSE_VAL)));
                    code.push(Instr::IMov(Val::Reg(Reg::RCX), Val::Imm(TRUE_VAL)));
                    code.push(Instr::ICMovE(Val::Reg(Reg::RAX), Val::Reg(Reg::RCX)));
                }
                Op1::IsBool => {
                    code.push(Instr::ITest(Val::Reg(Reg::RAX), Val::Imm(1)));
                    code.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(TRUE_VAL)));
                    code.push(Instr::IMov(Val::Reg(Reg::RCX), Val::Imm(FALSE_VAL)));
                    code.push(Instr::ICMovE(Val::Reg(Reg::RAX), Val::Reg(Reg::RCX)));
                }
                Op1::Print => {
                    code.push(Instr::IPush(Val::Reg(Reg::RAX)));
                    code.push(Instr::IMov(Val::Reg(Reg::RDI), Val::Reg(Reg::RAX)));
                    code.push(Instr::ICall("_snek_print".to_string()));
                    code.push(Instr::IPop(Val::Reg(Reg::RAX)));
                }
            }
        }
        Expr::BinOp(op, left, right) => {
            let (mut left_code, left_min) = compile_to_instrs(left, si, env, defines, fun_ctx, input, loop_end);
            current_min = current_min.min(left_min);
            code.append(&mut left_code);

            code.push(Instr::IMov(Val::RegOffset(Reg::RBP, si), Val::Reg(Reg::RAX)));
            current_min = current_min.min(si);

            let (mut right_code, right_min) = compile_to_instrs(right, si - 8, env, defines, fun_ctx, input, loop_end);
            current_min = current_min.min(right_min);
            code.append(&mut right_code);

            match op {
                Op2::Plus | Op2::Minus | Op2::Times => {
                    code.push(Instr::IMov(Val::Reg(Reg::RCX), Val::Reg(Reg::RAX)));
                    code.push(Instr::IOr(Val::Reg(Reg::RCX), Val::RegOffset(Reg::RBP, si)));
                    code.push(Instr::ITest(Val::Reg(Reg::RCX), Val::Imm(1)));
                    code.push(Instr::IJne("error_invalid_argument".to_string()));

                    match op {
                        Op2::Plus => {
                            code.push(Instr::IAdd(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RBP, si)));
                            code.push(Instr::IJo("error_overflow".to_string()));
                        }
                        Op2::Minus => {
                            code.push(Instr::IMov(Val::Reg(Reg::RCX), Val::Reg(Reg::RAX)));
                            code.push(Instr::IMov(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RBP, si)));
                            code.push(Instr::ISub(Val::Reg(Reg::RAX), Val::Reg(Reg::RCX)));
                            code.push(Instr::IJo("error_overflow".to_string()));
                        }
                        Op2::Times => {
                            code.push(Instr::ISar(Val::Reg(Reg::RAX), Val::Imm(1)));
                            code.push(Instr::IMul(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RBP, si)));
                            code.push(Instr::IJo("error_overflow".to_string()));
                        }
                        _ => unreachable!(),
                    }
                }
                Op2::Less | Op2::Greater | Op2::LessEqual | Op2::GreaterEqual => {
                    code.push(Instr::IMov(Val::RegOffset(Reg::RBP, si - 8), Val::Reg(Reg::RAX)));
                    current_min = current_min.min(si - 8);
                    
                    code.push(Instr::IMov(Val::Reg(Reg::RCX), Val::Reg(Reg::RAX)));
                    code.push(Instr::IOr(Val::Reg(Reg::RCX), Val::RegOffset(Reg::RBP, si)));
                    code.push(Instr::ITest(Val::Reg(Reg::RCX), Val::Imm(1)));
                    code.push(Instr::IJne("error_invalid_argument".to_string()));
                    
                    code.push(Instr::IMov(Val::Reg(Reg::RCX), Val::RegOffset(Reg::RBP, si)));
                    code.push(Instr::IMov(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RBP, si - 8)));
                    code.push(Instr::ICmp(Val::Reg(Reg::RCX), Val::Reg(Reg::RAX)));
                    
                    code.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(FALSE_VAL)));
                    code.push(Instr::IMov(Val::Reg(Reg::RCX), Val::Imm(TRUE_VAL)));
                    
                    match op {
                        Op2::Less => code.push(Instr::ICMovL(Val::Reg(Reg::RAX), Val::Reg(Reg::RCX))),
                        Op2::Greater => code.push(Instr::ICMovG(Val::Reg(Reg::RAX), Val::Reg(Reg::RCX))),
                        Op2::LessEqual => code.push(Instr::ICMovLE(Val::Reg(Reg::RAX), Val::Reg(Reg::RCX))),
                        Op2::GreaterEqual => code.push(Instr::ICMovGE(Val::Reg(Reg::RAX), Val::Reg(Reg::RCX))),
                        _ => unreachable!(),
                    }
                }
                Op2::Equal => {
                    code.push(Instr::ICmp(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RBP, si)));
                    code.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(TRUE_VAL)));
                    code.push(Instr::IMov(Val::Reg(Reg::RCX), Val::Imm(FALSE_VAL)));
                    code.push(Instr::ICMovNE(Val::Reg(Reg::RAX), Val::Reg(Reg::RCX)));
                }
            }
        }
        Expr::Set(name, expr) => {
            let (mut expr_code, expr_min) = compile_to_instrs(expr, si, env, defines, fun_ctx, input, loop_end);
            current_min = current_min.min(expr_min);
            code.append(&mut expr_code);

            if let Some(offset) = env.get(name) {
                code.push(Instr::IMov(Val::RegOffset(Reg::RBP, *offset), Val::Reg(Reg::RAX)));
                current_min = current_min.min(*offset);
            } else if let Some(&heap_offset) = defines.get(name) {
                code.push(Instr::IMov(Val::RegOffset(Reg::R15, heap_offset), Val::Reg(Reg::RAX)));
            } else {
                panic!("Unbound variable identifier {}", name);
            }
        }
        Expr::If(cond, then_expr, else_expr) => {
            let else_label = new_label("else");
            let end_label = new_label("endif");

            let (mut cond_code, cond_min) = compile_to_instrs(cond, si, env, defines, fun_ctx, input, loop_end);
            current_min = current_min.min(cond_min);
            code.append(&mut cond_code);

            code.push(Instr::ICmp(Val::Reg(Reg::RAX), Val::Imm(FALSE_VAL)));
            code.push(Instr::IJe(else_label.clone()));

            let (mut then_code, then_min) = compile_to_instrs(then_expr, si, env, defines, fun_ctx, input, loop_end);
            current_min = current_min.min(then_min);
            code.append(&mut then_code);
            code.push(Instr::IJmp(end_label.clone()));

            code.push(Instr::ILabel(else_label));
            let (mut else_code, else_min) = compile_to_instrs(else_expr, si, env, defines, fun_ctx, input, loop_end);
            current_min = current_min.min(else_min);
            code.append(&mut else_code);

            code.push(Instr::ILabel(end_label));
        }
        Expr::Block(exprs) => {
            for expr in exprs {
                let (mut expr_code, expr_min) = compile_to_instrs(expr, si, env, defines, fun_ctx, input, loop_end);
                current_min = current_min.min(expr_min);
                code.append(&mut expr_code);
            }
        }
        Expr::Let(bindings, body) => {
            let mut new_env = env.clone();
            let mut current_si = si;
            let mut local_min = current_min;

            let mut seen_names = HashMap::new();
            for (name, _) in bindings {
                if seen_names.contains_key(name) {
                    panic!("Duplicate binding");
                }
                seen_names = seen_names.update(name.clone(), ());
            }

            for (name, expr) in bindings {
                let (mut expr_code, expr_min) = compile_to_instrs(expr, current_si - 8, &new_env, defines, fun_ctx, input, loop_end);
                local_min = local_min.min(expr_min);
                code.append(&mut expr_code);

                code.push(Instr::IMov(Val::RegOffset(Reg::RBP, current_si), Val::Reg(Reg::RAX)));
                new_env = new_env.update(name.clone(), current_si);
                local_min = local_min.min(current_si);

                current_si -= 8;
            }

            let (mut body_code, body_min) = compile_to_instrs(body, current_si, &new_env, defines, fun_ctx, input, loop_end);
            local_min = local_min.min(body_min);
            code.append(&mut body_code);

            current_min = current_min.min(local_min);
        }
        Expr::Loop(body) => {
            let loop_start = new_label("loop_start");
            let loop_end_label = new_label("loop_end");

            code.push(Instr::ILabel(loop_start.clone()));
            let (mut body_code, body_min) = compile_to_instrs(body, si, env, defines, fun_ctx, input, &Some(loop_end_label.clone()));
            current_min = current_min.min(body_min);
            code.append(&mut body_code);
            code.push(Instr::IJmp(loop_start));
            code.push(Instr::ILabel(loop_end_label));
        }
        Expr::Break(expr) => {
            let loop_end_label = loop_end.as_ref().expect("break outside of loop").clone();
            let (mut expr_code, expr_min) = compile_to_instrs(expr, si, env, defines, fun_ctx, input, loop_end);
            current_min = current_min.min(expr_min);
            code.append(&mut expr_code);
            code.push(Instr::IJmp(loop_end_label));
        }
        Expr::Call(name, args) => {
            if !fun_ctx.check_function_exists(name) {
                panic!("Undefined function: {}", name);
            }
            let expected = fun_ctx.get_param_count(name);
            if args.len() != expected {
                panic!("Wrong number of arguments for {}: expected {}, got {}", name, expected, args.len());
            }
            let arg_bytes = (args.len() * 8) as i32;

            let needs_pad = (arg_bytes + 8) % 16 != 0;
            if needs_pad {
                code.push(Instr::ISub(Val::Reg(Reg::RSP), Val::Imm(8)));
            }
            
            for arg in args.iter().rev() {
                let (mut arg_code, arg_min) = compile_to_instrs(arg, si, env, defines, fun_ctx, input, loop_end);
                current_min = current_min.min(arg_min);
                code.append(&mut arg_code);
                code.push(Instr::IPush(Val::Reg(Reg::RAX)));
            }
            
            code.push(Instr::ICall(format!("fun_{}", name)));
            
            if arg_bytes > 0 {
                code.push(Instr::IAdd(Val::Reg(Reg::RSP), Val::Imm(arg_bytes)));
            }
            
            if needs_pad {
                code.push(Instr::IAdd(Val::Reg(Reg::RSP), Val::Imm(8)));
            }
        }
        Expr::Cast(expr, target_type) => {
            let (mut expr_code, expr_min) = compile_to_instrs(expr, si, env, defines, fun_ctx, input, loop_end);
            current_min = current_min.min(expr_min);
            code.append(&mut expr_code);
            
            // Runtime type check for cast
            match target_type {
                Type::Num => {
                    code.push(Instr::ITest(Val::Reg(Reg::RAX), Val::Imm(1)));
                    code.push(Instr::IJne("error_bad_cast".to_string()));
                }
                Type::Bool => {
                    code.push(Instr::ITest(Val::Reg(Reg::RAX), Val::Imm(1)));
                    code.push(Instr::IJe("error_bad_cast".to_string()));
                }
                Type::Nothing => {
                    code.push(Instr::IJmp("error_bad_cast".to_string()));
                }
                Type::Any => {
                    // No check needed
                }
            }
        }
    }

    (code, current_min)
}

pub fn compile(program: &Program) -> String {
    unsafe {
        LABEL_COUNTER = 0;
        HEAP_OFFSET = 0;
        INPUT_HEAP_OFFSET = None;
    }
    
    let fun_ctx = FunContext::new(&program.defns);
    let mut asm_code = String::new();
    
    for defn in &program.defns {
        check_no_input(&defn.body);
    }
    
    for defn in &program.defns {
        asm_code.push_str(&compile_function(defn, &fun_ctx));
    }
    
    asm_code.push_str("our_code_starts_here:\n");
    asm_code.push_str("  push rbp\n");
    asm_code.push_str("  mov rbp, rsp\n");
    
    let input_heap_offset = get_input_heap_offset();
    asm_code.push_str(&format!("  mov [r15 + {}], rdi\n", input_heap_offset));
    
    let (instrs, min_offset) = compile_to_instrs(
        &program.main, 
        -8, 
        &HashMap::new(), 
        &HashMap::new(),
        &fun_ctx,
        true, 
        &None
    );

    // Allocate stack space for local variables if needed
    // Only allocate if we've actually used stack slots (min_offset < -8 means we used [rbp-16] or lower)
    if min_offset <= -16 {
        let needed = -min_offset - 8;  // Subtract 8 because -8 is just the starting point
        let stack_space = ((needed + 15) / 16) * 16;
        asm_code.push_str(&format!("  sub rsp, {}\n", stack_space));
    }

    for instr in instrs {
        asm_code.push_str(&instr_to_str(&instr));
        asm_code.push('\n');
    }
    
    asm_code.push_str("  mov rsp, rbp\n");
    asm_code.push_str("  pop rbp\n");
    asm_code.push_str("  ret\n");
    
    asm_code.push_str("\nerror_overflow:\n");
    asm_code.push_str("  mov rdi, 1\n");
    asm_code.push_str("  call snek_error\n");
    asm_code.push_str("  ret\n");
    
    asm_code.push_str("\nerror_invalid_argument:\n");
    asm_code.push_str("  mov rdi, 2\n");
    asm_code.push_str("  call snek_error\n");
    asm_code.push_str("  ret\n");

    asm_code.push_str("\nerror_bad_cast:\n");
    asm_code.push_str("  mov rdi, 3\n");
    asm_code.push_str("  call snek_error\n");
    asm_code.push_str("  ret\n");
    
    asm_code
}

fn compile_function(defn: &FunDefn, fun_ctx: &FunContext) -> String {
    let mut code = String::new();
    code.push_str(&format!("fun_{}:\n", defn.name));

    code.push_str("  push rbp\n");
    code.push_str("  mov rbp, rsp\n");

    let mut env = HashMap::new();
    for (i, param) in defn.params.iter().enumerate() {
        let offset = 16 + (i as i32 * 8);
        env.insert(param.clone(), offset);
    }

    let (instrs, min_offset) = compile_to_instrs(
        &defn.body,
        -8,
        &env,
        &HashMap::new(),
        fun_ctx,
        false,
        &None,
    );

    // Allocate stack space for local variables if needed
    if min_offset < 0 {
        let needed = -min_offset;
        let stack_space = ((needed + 15) / 16) * 16;
        code.push_str(&format!("  sub rsp, {}\n", stack_space));
    }

    for instr in instrs {
        code.push_str(&instr_to_str(&instr));
        code.push('\n');
    }

    code.push_str("  mov rsp, rbp\n");
    code.push_str("  pop rbp\n");
    code.push_str("  ret\n");

    code
}

fn check_no_input(expr: &Expr) {
    if let Expr::Input = expr {
        panic!("input not allowed in function definitions");
    }
}

pub fn compile_define(name: &str, expr: &Expr, defines: &HashMap<String, i32>, fun_ctx: &FunContext) -> (i32, Vec<Instr>) {
    let heap_offset = alloc_heap_slot();
    let (mut code, _) = compile_to_instrs(expr, -8, &HashMap::new(), defines, fun_ctx, false, &None);
    code.push(Instr::IMov(Val::RegOffset(Reg::R15, heap_offset), Val::Reg(Reg::RAX)));
    (heap_offset, code)
}
