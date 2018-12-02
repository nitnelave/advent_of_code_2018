use std::collections::HashSet;
use std::option::Option;
use std::string::String;

/// Size of the alphabet.
const ALPHABET_SIZE: usize = 26;

/// This structure helps to count the number of pairs and triplets in a string.
/// It assumes that the string is composed only of lowercase letters between 'a'
/// and 'z'.
struct LetterCounter {
    /// This is a histogram of how many times we've seen each letter, by their
    /// index in the alphabet.
    histogram: [i32; ALPHABET_SIZE],
    /// Number of (exact) pairs of characters.
    number_of_pairs: i32,
    /// Number of (exact) triplets of characters.
    number_of_triplets: i32,
}

impl LetterCounter {
    /// Create a new LetterCount with everything initialized to 0.
    fn new() -> LetterCounter {
        LetterCounter {
            histogram: [0; ALPHABET_SIZE],
            number_of_pairs: 0,
            number_of_triplets: 0,
        }
    }

    /// Add a letter to our histogram, and update the counts of pairs and
    /// triplets.
    fn add_letter(&mut self, letter: char) {
        // Index of the letter in the alphabet.
        let index = (letter as usize) - ('a' as usize);
        // Reference to the relevant cell in the histogram.
        let cell = &mut self.histogram[index];
        // If the letter was already a pair or a triple, it's not anymore (since
        // we're adding one count).
        match cell {
            2 => self.number_of_pairs -= 1,
            3 => self.number_of_triplets -= 1,
            _ => (),
        };
        *cell += 1;
        // If the letter is now a pair or a triple, add it to the count.
        match cell {
            2 => self.number_of_pairs += 1,
            3 => self.number_of_triplets += 1,
            _ => (),
        };
    }

    /// Return whether we have at least one (exact) pair of letters.
    fn has_exactly_two(&self) -> bool {
        self.number_of_pairs > 0
    }

    /// Return whether we have at least one (exact) triplet of letters.
    fn has_exactly_three(&self) -> bool {
        self.number_of_triplets > 0
    }
}

/// Check whether a string has pairs and/or triplets. It returns 2 values,
/// whether there are pairs, and whether there are triplets.
fn count_pairs_and_triplets(line: &String) -> (bool, bool) {
    let mut counter = LetterCounter::new();
    for letter in line.chars() {
        counter.add_letter(letter);
    }
    (counter.has_exactly_two(), counter.has_exactly_three())
}

/// Compute the checksum of a list of IDs: this is the number of IDs with an
/// (exact) pair times the number of IDs with an exact triplet.
pub fn checksum(lines: &Vec<String>) -> i32 {
    // Call count_pairs_and_triplets on every line. We get a list of pairs of
    // bool.
    let counts = lines.iter().map(&count_pairs_and_triplets);
    // Count the number of elements with true in the first spot.
    let number_of_pairs = counts.clone().filter(|t| t.0).count();
    // Count the number of elements with true in the second spot.
    let number_of_triplets = counts.filter(|t| t.1).count();
    (number_of_pairs * number_of_triplets) as i32
}

/// Consider each line without the nth letter (index), and try to find
/// duplicates. Returns the first duplicate line, or None if there is none.
fn find_matching_ids_without_letter(lines: &Vec<String>, index: usize) -> Option<String> {
    let mut seen: HashSet<String> = HashSet::new();
    for id in lines {
        let new_string: String = id.char_indices()
            .filter(|(i, _)| *i != index)
            .map(|(_, c)| c)
            .collect();
        if seen.contains(&new_string) {
            return Some(new_string);
        }
        seen.insert(new_string);
    }
    None
}

/// Find matching ids. Two IDs are matching if they differ only by 1 character.
pub fn find_matching_ids(lines: &Vec<String>) -> String {
    let length = lines[0].len();
    for i in 0..length {
        match find_matching_ids_without_letter(&lines, i) {
            Some(s) => return s,
            None => (),
        }

    }
    panic!("No matching id found!")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_letter_counter() {
        let mut counter = LetterCounter::new();
        assert_eq!(0, counter.histogram[0]);
        assert_eq!(0, counter.number_of_pairs);
        assert_eq!(0, counter.number_of_triplets);
        counter.add_letter('a');
        assert_eq!(1, counter.histogram[0]);
        assert_eq!(0, counter.number_of_pairs);
        counter.add_letter('a');
        assert_eq!(2, counter.histogram[0]);
        assert_eq!(1, counter.number_of_pairs);
        assert_eq!(true, counter.has_exactly_two());
        assert_eq!(false, counter.has_exactly_three());
        counter.add_letter('a');
        assert_eq!(3, counter.histogram[0]);
        assert_eq!(0, counter.number_of_pairs);
        assert_eq!(1, counter.number_of_triplets);
        assert_eq!(false, counter.has_exactly_two());
        assert_eq!(true, counter.has_exactly_three());
        counter.add_letter('a');
        assert_eq!(4, counter.histogram[0]);
        assert_eq!(0, counter.number_of_pairs);
        assert_eq!(0, counter.number_of_triplets);
        counter.add_letter('c');
        assert_eq!(1, counter.histogram[2]);
    }

    #[test]
    fn test_count_pairs_and_triplets() {
        assert_eq!(
            (false, false),
            count_pairs_and_triplets(&"abcdef".to_string())
        );
        assert_eq!(
            (true, true),
            count_pairs_and_triplets(&"bababc".to_string())
        );
        assert_eq!(
            (true, false),
            count_pairs_and_triplets(&"abbcde".to_string())
        );
        assert_eq!(
            (false, true),
            count_pairs_and_triplets(&"abcccd".to_string())
        );
        assert_eq!(
            (true, false),
            count_pairs_and_triplets(&"aabcdd".to_string())
        );
        assert_eq!(
            (true, false),
            count_pairs_and_triplets(&"abcdee".to_string())
        );
        assert_eq!(
            (false, true),
            count_pairs_and_triplets(&"ababab".to_string())
        );
    }

    #[test]
    fn test_checksum() {
        let lines: Vec<String> = vec![
            "abcdef",
            "bababc",
            "abbcde",
            "abcccd",
            "aabcdd",
            "abcdee",
            "ababab",
        ].into_iter()
            .map(str::to_string)
            .collect();
        assert_eq!(12, checksum(&lines));
    }

    #[test]
    fn test_find_matching_ids_without_letter() {
        let lines: Vec<String> = vec![
            "abcde",
            "fghij",
            "klmno",
            "pqrst",
            "fguij",
            "axcye",
            "wvxyz",
        ].into_iter()
            .map(str::to_string)
            .collect();
        assert_eq!(None, find_matching_ids_without_letter(&lines, 0));
        assert_eq!(None, find_matching_ids_without_letter(&lines, 1));
        assert_eq!(
            Some("fgij".to_string()),
            find_matching_ids_without_letter(&lines, 2)
        );
    }

    #[test]
    fn test_find_matching_ids() {
        let lines: Vec<String> = vec![
            "abcde",
            "fghij",
            "klmno",
            "pqrst",
            "fguij",
            "axcye",
            "wvxyz",
        ].into_iter()
            .map(str::to_string)
            .collect();
        assert_eq!("fgij", find_matching_ids(&lines));
    }
}
