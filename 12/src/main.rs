extern crate lib;
use std::io;
use std::io::Read;
use std::string::String;

fn main() {
    let stdin = io::stdin();
    let mut line = String::new();
    // Read the lines from stdin.
    stdin.lock().read_to_string(&mut line).expect(
        "Error reading input: ",
    );
    println!("{}", lib::count_pots_from_input(&line, 20));
    println!("{}", lib::count_pots_from_input(&line, 50_000_000_000));
}
