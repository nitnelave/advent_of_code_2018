extern crate lib;
use std::io;
use std::io::Read;
use std::string::String;

fn main() {
    let stdin = io::stdin();
    let mut line = String::new();
    // Read the lines from stdin.
    if let Err(err) = stdin.lock().read_to_string(&mut line) {
        println!("Error parsing list: {}", err);
        std::process::exit(2);
    };
    let specification = lib::parse_tree_specification(&line).expect("Error parsing specification");
    let tree = lib::parse_tree(&specification).expect("Error parsing tree");
    println!("Metadata sum: {}", lib::count_metadata(&tree));
    println!("Root value: {}", lib::compute_root_value(&tree));
}
