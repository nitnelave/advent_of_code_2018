extern crate lib;
use std::io;
use std::io::BufRead;
use std::string::String;

fn main() {
    let stdin = io::stdin();
    // Read the lines from stdin.
    let lines: Vec<String> = stdin.lock().lines().map(Result::unwrap).collect();
    println!(
        "Largest area: {}",
        lib::find_largest_close_area(lines.as_slice())
    );
    println!(
        "Largest safe area: {}",
        lib::find_area_close_to_points(lines.as_slice(), 10000)
    );
}
