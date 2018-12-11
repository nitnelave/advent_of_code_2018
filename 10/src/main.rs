extern crate lib;
use std::io;
use std::io::BufRead;
use std::string::String;

fn main() {
    let stdin = io::stdin();
    // Read the lines from stdin.
    let lines = stdin
        .lock()
        .lines()
        .collect::<std::io::Result<Vec<String>>>()
        .expect("Error reading lines: {}");
    let coordinates = lib::parse_lines(&lines);
    lib::print_message(&coordinates);
}
