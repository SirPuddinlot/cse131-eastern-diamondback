// helpers.rs
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::atomic::{AtomicI64};

pub const TRUE_VAL: i64 = 1;
pub const FALSE_VAL: i64 = 3;

pub static REPL: AtomicBool = AtomicBool::new(false);

pub static LAST_ERR: AtomicI64 = AtomicI64::new(0);

pub fn parse_input(input: &str) -> i64 {
    match input {
        "true" => TRUE_VAL,
        "false" => FALSE_VAL,
        _ => {
            let num = input.parse::<i64>().unwrap_or_else(|_| {
                eprintln!("Invalid input: {}", input);
                std::process::exit(1);
            });
            
            if num < -4611686018427387904 || num > 4611686018427387903 {
                eprintln!("Input number out of range");
                std::process::exit(1);
            }
            num << 1  
        }
    }
}

pub fn print_result(val: i64) {
    if val & 1 == 1 {
        if val == 1 {              // TRUE = 1
            println!("true");
        } else if val == 3 {       // FALSE = 3
            println!("false");
        } else {
            eprintln!("Invalid boolean value: {}", val);
            std::process::exit(1);
        }
    } else {
        println!("{}", val >> 1);
    }
}
// Export snek_error for JIT to call
#[export_name = "\x01snek_error"]
pub extern "C" fn snek_error(errcode: i64) {
    match errcode {
        1 => eprintln!("overflow"),
        2 => eprintln!("invalid argument"),
        3 => eprintln!("bad cast"),
        _ => eprintln!("unknown error code: {}", errcode),
    }
    if REPL.load(Ordering::SeqCst) {
        print!("Runtime error");
    } 
    else {
        std::process::exit(1);
    }
}

#[export_name = "_snek_print"]
pub extern "C" fn _snek_print(val: i64) -> i64 {
    println!("{}", if val & 1 == 0 {
        format!("{}", val >> 1)
    } else if val == 1 {           
        "true".to_string()
    } else if val == 3 {          
        "false".to_string()
    } else {
        format!("Unknown value: {}", val)
    });
    val
}
