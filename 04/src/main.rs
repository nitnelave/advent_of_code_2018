extern crate lib;
use std::io;
use std::io::BufRead;
use std::string::String;

fn main() {
    let stdin = io::stdin();
    // Read the lines from stdin.
    let lines: Vec<String> = stdin.lock().lines().map(|l| l.unwrap()).collect();
    // Print the result of the two strategies.
    println!("{:?}", lib::find_guard_and_time(&lines));
}
