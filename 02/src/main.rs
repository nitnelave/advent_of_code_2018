extern crate lib;
use std::io;
use std::io::BufRead;

/// Solution for the 2nd day of the advent of code 2018.
///
/// This solution assumes that all the ids are composed only of lowercase letters, and that they
/// are all the same length.

fn main() {
    let stdin = io::stdin();
    // Read the lines of the input.
    let lines: Vec<std::string::String> = stdin.lock().lines().map(Result::unwrap).collect();
    println!("checksum: {}", lib::checksum(&lines));
    println!("common id: {}", lib::find_matching_ids(&lines));
}
