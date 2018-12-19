extern crate lib;
use std::io;
use std::io::Read;

fn main() {
    let stdin = io::stdin();
    let mut line = String::new();
    // Read the lines from stdin.
    if let Err(err) = stdin.lock().read_to_string(&mut line) {
        println!("Error reading input: {}", err);
        std::process::exit(2);
    };
    let program = lib::parse_input(&line).expect("Failed to parse: ");
    println!("Register 0: {}", lib::run_program(&program, 0));
    println!("Second run, register 0: {}", lib::run_program(&program, 1));
}
