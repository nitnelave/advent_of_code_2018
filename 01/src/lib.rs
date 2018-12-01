use std::io;
use std::io::BufRead;
use std::collections::HashSet;

/// Parse one line into an int, positive or negative.
fn parse_int(s: std::string::String) -> i64 {
    return s.parse::<i64>().unwrap();
}

/// Read the list of numbers from the standard input.
pub fn get_list_input() -> Vec<i64> {
    // Open standard input.
    let stdin = io::stdin();
    // Get a handle on it.
    return stdin.lock()
        // Read the input line by line.
        .lines()
        // Unwrap each line (makes the program crash if there is an error).
        .map(|l| l.unwrap())
        // Parse each line as an int64 (crash on failure).
        .map(parse_int)
        // Collect everything into a vector.
        .collect();
}

/// Sum the contents of the vector.
///
/// For example:
/// ```
/// assert_eq!(3, lib::sum_vector(&vec![1, 1, 1]));
/// assert_eq!(0, lib::sum_vector(&vec![1, 1, -2]));
/// assert_eq!(-6, lib::sum_vector(&vec![-1, -2, -3]));
/// ```
pub fn sum_vector(numbers: &Vec<i64>) -> i64 {
    return numbers.iter().sum();
}

/// Given a list of numbers, find the first partial sum that is reached more than once.
pub fn find_first_repeated(numbers: &Vec<i64>) -> i64 {
    // Sum of the numbers seen up to now.
    let mut sum = 0;
    // Set of frequencies (partial sums) we have seen so far.
    let mut seen = HashSet::new();
    // We have seen the value 0 (initial frequency).
    seen.insert(sum);
    // Iterate over the numbers in the list, in an infinite cycle.
    for number in numbers.iter().cycle() {
        sum += number;
        // If we have seen this new frequency before, we found it!
        if seen.contains(&sum) {
            return sum;
        }
        // Add the frequency to the list of those we have seen.
        seen.insert(sum);
    }
    // The loop above is infinite, we can't get here.
    unreachable!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_repeated() {
        assert_eq!(0, find_first_repeated(&vec![1, -1]));
        assert_eq!(10, find_first_repeated(&vec![3, 3, 4, -2, -4]));
        assert_eq!(5, find_first_repeated(&vec![-6, 3, 8, 5, -6]));
        assert_eq!(14, find_first_repeated(&vec![7, 7, -2, -7, -4]));
    }

    #[test]
    fn test_parse_int() {
        assert_eq!(3, parse_int("+3".to_string()));
        assert_eq!(-44, parse_int("-44".to_string()));
    }
}
