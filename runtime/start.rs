use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
// #[link(name = "our_code")]
// extern "C" {
//     // The \x01 here is an undocumented feature of LLVM (which Rust uses) that ensures
//     // it does not add an underscore in front of the name, which happens on OSX
//     // Courtesy of Max New
//     // (https://maxsnew.com/teaching/eecs-483-fa22/hw_adder_assignment.html)
//     #[link_name = "\x01our_code_starts_here"]
//     fn our_code_starts_here() -> i64;
// }

// fn main() {
//   let i : i64 = unsafe {
//     our_code_starts_here()
//   };
//   println!("{i}");
// }

#[link(name = "our_code")]
extern "C" {
    // The \x01 here is an undocumented feature of LLVM (which Rust uses) that ensures
    // it does not add an underscore in front of the name, which happens on OSX
    // Courtesy of Max New
    // (https://maxsnew.com/teaching/eecs-483-fa22/hw_adder_assignment.html)
    #[link_name = "\x01our_code_starts_here"]
    fn our_code_starts_here(input: i64) -> i64;
}

// #[export_name = "\x01snek_error"]
// pub extern "C" fn snek_error(errcode: i64) {
//     if errcode == 1 {
//         eprintln!("overflow");
//     } else if errcode == 2 {
//         eprintln!("invalid argument");
//     } else {
//         eprintln!("error code: {}", errcode);
//     }
//     std::process::exit(1);
// }

pub static REPL: AtomicBool = AtomicBool::new(false);

#[export_name = "\x01snek_error"]
pub extern "C" fn snek_error(errcode: i64) {
    match errcode {
        1 => eprintln!("overflow"),
        2 => eprintln!("invalid argument"),
        3 => eprintln!("bad cast"),
        _ => eprintln!("unknown error code: {}", errcode),
    }
    if REPL.load(Ordering::SeqCst) {
        panic!("Runtime error");
    } 
    else {
        std::process::exit(1);
    }
}

#[export_name = "\x01snek_print"]
pub extern "C" fn snek_print(val: i64) -> i64 {
    // Print the value and return it
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

const TRUE_VAL: i64 = 1;   // 0b01
const FALSE_VAL: i64 = 3;  // 0b11

fn parse_input(input: &str) -> i64 {
    match input {
        "true" => TRUE_VAL,
        "false" => FALSE_VAL,
        _ => {
            let num = input.parse::<i64>().unwrap_or_else(|_| {
                eprintln!("Invalid input: {}", input);
                std::process::exit(1);
            });
            
            // Check 63-bit bounds
            if num < -4611686018427387904 || num > 4611686018427387903 {
                eprintln!("Input number out of range");
                std::process::exit(1);
            }
            
            num << 1  // Tag as number
        }
    }
}

fn print_result(val: i64) {
    if val & 1 == 1 {
        // It's a boolean
        if val == TRUE_VAL {
            println!("true");
        } else if val == FALSE_VAL {
            println!("false");
        } else {
            eprintln!("Invalid boolean value: {}", val);
            std::process::exit(1);
        }
    } else {
        // It's a number - shift right to get actual value
        println!("{}", val >> 1);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    let input = if args.len() > 1 {
        parse_input(&args[1])
    } 
    else {
        FALSE_VAL 
    };
    
    let result: i64 = unsafe { our_code_starts_here(input) };
    print_result(result);
}