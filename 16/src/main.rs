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
    let (samples, program) = lib::parse_input(&line).expect("Could not parse input: ");
    println!("ambiguous: {}", lib::num_very_ambiguous_ops(&samples));
    println!("output: {}", lib::execute_program(&samples, &program));
}
