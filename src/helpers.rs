// helpers.rs
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::atomic::AtomicI64;
use std::panic;

pub const TRUE_VAL: i64 = 1;
pub const FALSE_VAL: i64 = 3;

pub static REPL: AtomicBool = AtomicBool::new(false);
pub static HAS_ERROR: AtomicBool = AtomicBool::new(false);
pub static ERROR_CODE: AtomicI64 = AtomicI64::new(0);

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
        if val == 1 {
            println!("true");
        } else if val == 3 {
            println!("false");
        } else {
            eprintln!("Invalid boolean value: {}", val);
            std::process::exit(1);
        }
    } else {
        println!("{}", val >> 1);
    }
}

#[export_name = "\x01snek_error"]
pub extern "C" fn snek_error(errcode: i64) {
    // Use catch_unwind to handle panics at the FFI boundary
    let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        // Store error information
        ERROR_CODE.store(errcode, Ordering::SeqCst);
        HAS_ERROR.store(true, Ordering::SeqCst);
        
        if !REPL.load(Ordering::SeqCst) {
            // Only exit in non-REPL mode
            match errcode {
                1 => eprintln!("overflow"),
                2 => eprintln!("invalid argument"),
                3 => eprintln!("bad cast"),
                _ => eprintln!("unknown error code: {}", errcode),
            }
            std::process::exit(1);
        }
        
        // In REPL mode, panic to unwind back to the REPL loop
        // This panic will be caught by catch_unwind, preventing it from crossing FFI
        panic!("Runtime error in REPL");
    }));
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

pub fn check_error() -> Option<String> {
    if HAS_ERROR.load(Ordering::SeqCst) {
        HAS_ERROR.store(false, Ordering::SeqCst);
        let code = ERROR_CODE.load(Ordering::SeqCst);
        Some(match code {
            1 => "overflow".to_string(),
            2 => "invalid argument".to_string(),
            3 => "bad cast".to_string(),
            _ => format!("unknown error code: {}", code),
        })
    } 
    else {
        None
    }
}