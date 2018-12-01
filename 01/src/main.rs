extern crate lib;
use lib::get_list_input;

fn main() {
    // Read the list of numbers from the standard input.
    let numbers = get_list_input();
    // Compute the sum.
    println!("Sum: {}", lib::sum_vector(&numbers));
    println!(
        "First repeated frequency: {}",
        lib::find_first_repeated(&numbers)
    );
}
