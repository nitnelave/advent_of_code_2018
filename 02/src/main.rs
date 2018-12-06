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
    let lines = stdin.lock().lines().map(Result::unwrap).collect::<Vec<_>>();
    println!("checksum: {}", lib::checksum(&lines));
    if let Some(id) = lib::find_matching_ids(&lines) {
        println!("common id: {}", id);
    } else {
        println!("No common id found");
    }
}
