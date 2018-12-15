extern crate lib;

use std::str::from_utf8;

fn main() {
    println!(
        "{}",
        from_utf8(
            &lib::find_score_after_steps(894_501)
                .iter()
                .map(|r| r + b'0')
                .collect::<Vec<_>>()
        )
        .unwrap()
    );
    println!("{}", lib::find_first_pattern("894501"));
}
