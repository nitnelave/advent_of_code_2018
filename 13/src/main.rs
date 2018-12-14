extern crate lib;
use std::io;
use std::io::Read;
use std::string::String;

fn main() {
    let stdin = io::stdin();
    let mut line = String::new();
    // Read the lines from stdin.
    stdin
        .lock()
        .read_to_string(&mut line)
        .expect("Error reading input: ");
    let (board, carts) = lib::parse_board(&line).expect("Could not parse input: ");
    let (carts_after, first_collision) = lib::find_first_collision(&board, carts);
    println!("First collision: {:?}", first_collision);
    println!(
        "Last car: {:?}",
        lib::find_remaining_cart(&board, carts_after)
    );
}
