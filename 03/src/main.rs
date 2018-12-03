extern crate lib;
use std::io;
use std::io::BufRead;
use std::string::String;

fn main() {
    let stdin = io::stdin();
    let lines: Vec<String> = stdin.lock().lines().map(|l| l.unwrap()).collect();
    let (area, non_overlapping_claim) = lib::find_overlapping_area(&lines);
    println!("Area: {}", area);
    match non_overlapping_claim {
        None => println!("No/several non-overlapping claims"),
        Some(c) => println!("Non-overlapping claim: {}", c),
    }
}
