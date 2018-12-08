extern crate lib;
use std::io;
use std::io::BufRead;
use std::string::String;

fn main() {
    let stdin = io::stdin();
    let lines: Vec<String> = stdin.lock().lines().map(|l| l.unwrap()).collect();
    println!(
        "Single worker build steps: {}",
        lib::find_build_order(&lines)
            .iter()
            .map(|&n| (u8::from(n) + 'A' as u8) as char)
            .collect::<String>()
    );
    println!(
        "5 worker build time: {}",
        lib::find_build_time_with_workers(&lines, 5)
    );

}
