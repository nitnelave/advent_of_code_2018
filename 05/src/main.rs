use std::io;
use std::io::Read;


fn main() {
    let stdin = io::stdin();
    let mut buffer = Vec::new();
    // Read the input as a vector of bytes.
    stdin.lock().read_to_end(&mut buffer).unwrap();
    println!(
        "Characters left in the list: {}",
        lib::remove_pairs(&buffer).len()
    );
    println!(
        "Characters left without worst unit: {}",
        lib::try_remove_pairs(&buffer)
    );
}
