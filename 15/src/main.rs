extern crate lib;
use std::io;
use std::io::Read;

fn main() {
    let stdin = io::stdin();
    let mut line = Vec::new();
    // Read the lines from stdin.
    if let Err(err) = stdin.lock().read_to_end(&mut line) {
        println!("Error reading input: {}", err);
        std::process::exit(2);
    };
    let board = lib::parse_board(&line).unwrap();
    let mut simple_board = board.clone();
    println!("{}", lib::compute_battle_score(&mut simple_board, 3));
    println!("{}", lib::find_min_attacking_power_score(&board));
}
