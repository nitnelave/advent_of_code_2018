extern crate lib;

fn main() {
    println!("{}", lib::get_winner(439, 71307));
    println!("{}", lib::get_winner(439, 100 * 71307));
}
