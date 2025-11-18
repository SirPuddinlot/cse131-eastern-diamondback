//typechecker.rs
use im::HashMap;
use crate::ast::*;

impl Type {
    pub fn union(&self, other: &Type) -> Type {
        match (self, other) {
            (Type::Any, _) | (_, Type::Any) => Type::Any,
            (t1, t2) if t1 == t2 => t1.clone(),
            (Type::Nothing, t) | (t, Type::Nothing) => t.clone(),
            (Type::Num, Type::Bool) | (Type::Bool, Type::Num) => Type::Any,
            (Type::Num, Type::Num) => Type::Num,
            (Type::Bool, Type::Bool) => Type::Bool,
        }
    }
    
    pub fn is_subtype(&self, other: &Type) -> bool {
        match (self, other) {
            (_, Type::Any) => true,
            (Type::Nothing, _) => true,
            (t1, t2) => t1 == t2,
        }
    }
}

pub fn typecheck_program(program: &Program, input_type: Option<Type>) -> Result<Type, String> {
    // Check all function definitions
    for defn in &program.defns {
        typecheck_defn(defn, &program.defns)?;
    }
    
    // Check main expression
    let mut env = HashMap::new();
    if let Some(t) = input_type {
        env = env.update("input".to_string(), t);
    } else {
        env = env.update("input".to_string(), Type::Any);
    }
    
    typecheck_expr(&program.main, &env, &program.defns)
}

// Make these functions public for REPL
pub fn typecheck_defn(defn: &FunDefn, all_defns: &[FunDefn]) -> Result<(), String> {
    let mut env = HashMap::new();
    
    // Build environment from parameters
    if let Some(ref param_types) = defn.param_types {
        for (param, typ) in defn.params.iter().zip(param_types.iter()) {
            env = env.update(param.clone(), typ.clone());
        }
    } 
    else {
        // Unannotated function - all params are Any
        for param in &defn.params {
            env = env.update(param.clone(), Type::Any);
        }
    }
    
    // Check body
    let body_type = typecheck_expr(&defn.body, &env, all_defns)?;
    
    // Check return type if annotated
    if let Some(ref ret_type) = defn.return_type {
        if !body_type.is_subtype(ret_type) {
            return Err(format!("Type error: function {} body has type {:?} but declared {:?}", 
                defn.name, body_type, ret_type));
        }
    }
    // For unannotated functions, we don't require the body to be Any
    // The body can have any type - it just means we treat calls as returning Any
    
    Ok(())
}

pub fn typecheck_expr(expr: &Expr, env: &HashMap<String, Type>, defns: &[FunDefn]) -> Result<Type, String> {
    match expr {
        Expr::Number(_) => Ok(Type::Num),
        Expr::Boolean(_) => Ok(Type::Bool),
        Expr::Input => {
            env.get("input")
                .cloned()
                .ok_or_else(|| "Type error: input not in environment".to_string())
        }
        Expr::Id(name) => {
            env.get(name)
                .cloned()
                .ok_or_else(|| format!("Type error: unbound variable {}", name))
        }
        Expr::UnOp(Op1::Add1 | Op1::Sub1, e) => {
            let t = typecheck_expr(e, env, defns)?;
            if !t.is_subtype(&Type::Num) {
                return Err(format!("Type error: add1/sub1 requires Num, got {:?}", t));
            }
            Ok(Type::Num)
        }
        Expr::UnOp(Op1::IsNum | Op1::IsBool, e) => {
            typecheck_expr(e, env, defns)?;
            Ok(Type::Bool)
        }
        Expr::UnOp(Op1::Print, e) => {
            typecheck_expr(e, env, defns)
        }
        Expr::BinOp(Op2::Plus | Op2::Minus | Op2::Times, e1, e2) => {
            let t1 = typecheck_expr(e1, env, defns)?;
            let t2 = typecheck_expr(e2, env, defns)?;
            if !t1.is_subtype(&Type::Num) {
                return Err(format!("Type error: arithmetic requires Num, got {:?}", t1));
            }
            if !t2.is_subtype(&Type::Num) {
                return Err(format!("Type error: arithmetic requires Num, got {:?}", t2));
            }
            Ok(Type::Num)
        }
        Expr::BinOp(Op2::Less | Op2::Greater | Op2::LessEqual | Op2::GreaterEqual, e1, e2) => {
            let t1 = typecheck_expr(e1, env, defns)?;
            let t2 = typecheck_expr(e2, env, defns)?;
            if !t1.is_subtype(&Type::Num) || !t2.is_subtype(&Type::Num) {
                return Err("Type error: comparison requires Num".to_string());
            }
            Ok(Type::Bool)
        }
        Expr::BinOp(Op2::Equal, e1, e2) => {
            let t1 = typecheck_expr(e1, env, defns)?;
            let t2 = typecheck_expr(e2, env, defns)?;
            if (t1.is_subtype(&Type::Num) && t2.is_subtype(&Type::Num)) ||
               (t1.is_subtype(&Type::Bool) && t2.is_subtype(&Type::Bool)) {
                Ok(Type::Bool)
            } else {
                Err("Type error: = requires both Num or both Bool".to_string())
            }
        }
        Expr::Let(bindings, body) => {
            let mut new_env = env.clone();
            for (name, e) in bindings {
                let t = typecheck_expr(e, &new_env, defns)?;
                new_env = new_env.update(name.clone(), t);
            }
            typecheck_expr(body, &new_env, defns)
        }
        Expr::If(cond, then_e, else_e) => {
            let cond_t = typecheck_expr(cond, env, defns)?;
            if !cond_t.is_subtype(&Type::Bool) {
                return Err(format!("Type error: if condition must be Bool, got {:?}", cond_t));
            }
            let then_t = typecheck_expr(then_e, env, defns)?;
            let else_t = typecheck_expr(else_e, env, defns)?;
            Ok(then_t.union(&else_t))
        }
        Expr::Block(exprs) => {
            let mut last_type = Type::Any;
            for e in exprs {
                last_type = typecheck_expr(e, env, defns)?;
            }
            Ok(last_type)
        }
        Expr::Set(name, e) => {
            let val_type = typecheck_expr(e, env, defns)?;
            let var_type = env.get(name)
                .ok_or_else(|| format!("Type error: unbound variable {}", name))?;
            if !val_type.is_subtype(var_type) {
                return Err(format!("Type error: cannot assign {:?} to {:?}", val_type, var_type));
            }
            Ok(val_type)
        }
        Expr::Loop(body) => {
            collect_break_types(body, env, defns)
        }
        Expr::Break(e) => {
            typecheck_expr(e, env, defns)?;
            Ok(Type::Nothing)
        }
        Expr::Call(fname, args) => {
            let defn = defns.iter().find(|d| d.name == *fname)
                .ok_or_else(|| format!("Type error: undefined function {}", fname))?;
            
            if args.len() != defn.params.len() {
                return Err(format!("Type error: wrong number of arguments"));
            }
            
            // Check arguments
            if let Some(ref param_types) = defn.param_types {
                for (arg, expected_type) in args.iter().zip(param_types.iter()) {
                    let arg_type = typecheck_expr(arg, env, defns)?;
                    if !arg_type.is_subtype(expected_type) {
                        return Err(format!("Type error: argument has type {:?}, expected {:?}", 
                            arg_type, expected_type));
                    }
                }
                Ok(defn.return_type.clone().unwrap())
            } 
            else {
                // Unannotated function - check args typecheck but return Any
                for arg in args {
                    typecheck_expr(arg, env, defns)?;
                }
                Ok(Type::Any)
            }
        }
        Expr::Cast(e, target_type) => {
            typecheck_expr(e, env, defns)?;
            Ok(target_type.clone())
        }
    }
}

fn collect_break_types(expr: &Expr, env: &HashMap<String, Type>, defns: &[FunDefn]) -> Result<Type, String> {
    let mut result = Type::Nothing;
    collect_break_types_helper(expr, env, defns, &mut result, false)?;
    Ok(result)
}

fn collect_break_types_helper(
    expr: &Expr, 
    env: &HashMap<String, Type>, 
    defns: &[FunDefn],
    result: &mut Type,
    in_nested_loop: bool
) -> Result<(), String> {
    match expr {
        Expr::Break(e) if !in_nested_loop => {
            let t = typecheck_expr(e, env, defns)?;
            *result = result.union(&t);
            Ok(())
        }
        Expr::Loop(_) => {
            // Don't recurse into nested loops
            Ok(())
        }
        Expr::UnOp(_, e) => collect_break_types_helper(e, env, defns, result, in_nested_loop),
        Expr::BinOp(_, e1, e2) => {
            collect_break_types_helper(e1, env, defns, result, in_nested_loop)?;
            collect_break_types_helper(e2, env, defns, result, in_nested_loop)
        }
        Expr::If(e1, e2, e3) => {
            collect_break_types_helper(e1, env, defns, result, in_nested_loop)?;
            collect_break_types_helper(e2, env, defns, result, in_nested_loop)?;
            collect_break_types_helper(e3, env, defns, result, in_nested_loop)
        }
        Expr::Let(bindings, body) => {
            let mut new_env = env.clone();
            for (name, e) in bindings {
                collect_break_types_helper(e, &new_env, defns, result, in_nested_loop)?;
                let t = typecheck_expr(e, &new_env, defns)?;
                new_env = new_env.update(name.clone(), t);
            }
            collect_break_types_helper(body, &new_env, defns, result, in_nested_loop)
        }
        Expr::Block(exprs) => {
            for e in exprs {
                collect_break_types_helper(e, env, defns, result, in_nested_loop)?;
            }
            Ok(())
        }
        Expr::Set(_, e) => collect_break_types_helper(e, env, defns, result, in_nested_loop),
        Expr::Call(_, args) => {
            for arg in args {
                collect_break_types_helper(arg, env, defns, result, in_nested_loop)?;
            }
            Ok(())
        }
        Expr::Cast(e, _) => collect_break_types_helper(e, env, defns, result, in_nested_loop),
        _ => Ok(()),
    }
}