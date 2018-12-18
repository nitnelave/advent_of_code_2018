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
    let board = lib::parse_input(&line).expect("Failed to parse: ");
    let new_board = lib::run_steps(board, 10);
    println!("score: {}", lib::compute_score(&new_board));
    // Keep running, up to 1 billion steps.
    let final_board = lib::run_steps(new_board, 1_000_000_000 - 10);
    println!("final score: {}", lib::compute_score(&final_board));
}
