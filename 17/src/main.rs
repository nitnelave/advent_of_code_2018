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
    let lines = lib::parse_input(&line).expect("Failed to parse: ");
    let mut grid = lib::make_grid(&lines);
    lib::fill_grid(&mut grid);
    println!("Water count: {}", lib::count_all_water(&grid));
    println!("Resting water count: {}", lib::count_resting_water(&grid));
    //println!("Grid:\n{}", grid);
}
