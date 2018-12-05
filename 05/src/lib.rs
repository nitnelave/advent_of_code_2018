extern crate linked_list;

use linked_list::{LinkedList, Cursor};

const CASE_DIFF: u8 = ('a' as u8) - ('A' as u8);

/// Transform the (ascii) input into a list, optionally skipping a unit.
/// The letters to be skipped are the one in except (must be within A-Z) and the
/// lowercase version of it.
fn to_linked_list_without(bytes: &Vec<u8>, except: Option<u8>) -> LinkedList<u8> {
    let mut list = LinkedList::new();
    for byte in bytes {
        if *byte != '\n' as u8 && Some(*byte) != except && Some(*byte - CASE_DIFF) != except {
            list.push_back(byte.clone());
        }
    }
    list
}

/// Check if v1 is the lowercase version of v2.
fn is_pair_checked(v1: &u8, v2: &u8) -> bool {
    match v1.checked_sub(*v2) {
        None => false,
        Some(v) => v == CASE_DIFF,
    }
}

/// Check if e1 and e2 are valid and e1 is the othercase version of e2.
fn is_pair(e1: Option<u8>, e2: Option<u8>) -> bool {
    match (e1, e2) {
        (Some(v1), Some(v2)) => is_pair_checked(&v1, &v2) || is_pair_checked(&v2, &v1),
        _ => false,
    }
}

/// Clone a mutable element option into an owning version.
fn clone<T: Clone>(element: Option<&mut T>) -> Option<T> {
    match element {
        None => None,
        Some(e) => Some(e.clone()),
    }
}

/// Meat of the algorithm. This takes a cursor (that sits between 2 elements)
/// that is before the first element of the list. It will go through the list
/// and remove all pairs (e.g. "aA" or "Pp").
fn remove_pairs_from_list<'a>(cursor: &mut Cursor<'a, u8>) {
    // Advance through the list one by one.
    while cursor.next() != None {
        // If the cursor is in the middle of a pair, remove the pair, and check
        // whether the cursor is in the middle of a pair again in case we
        // created a pair.
        while is_pair(clone(cursor.peek_prev()), clone(cursor.peek_next())) {
            // Remove the element after the cursor.
            cursor.remove();
            // Step back one element.
            cursor.seek_backward(1);
            // Remove the first element of the pair.
            cursor.remove();
        }
    }
}

/// Transform a list of bytes into a string.
fn string_from_list(list: &LinkedList<u8>) -> std::string::String {
    let vec: Vec<u8> = list.iter().map(|c| c.clone()).collect();
    std::str::from_utf8(&vec).unwrap().to_string()
}

/// Remove all the pairs in the given input, ignoring the letters specified by
/// unit: if None, then don't ignore letters, if Some(e), then ignore e and
/// lowercase(e). e must be an uppercase letter.
/// It returns the resulting string, after removal.
fn remove_pairs_without(bytes: &Vec<u8>, unit: Option<u8>) -> std::string::String {
    // Construct the list without the forbidden letters.
    let mut list = to_linked_list_without(bytes, unit);
    // Sanity check: the list has an even length.
    assert_eq!(list.len() % 2, 0);
    // Remove the adjacent pairs of letters.
    remove_pairs_from_list(&mut list.cursor());
    string_from_list(&list)
}

/// Remove the pairs from the input, returns the minimal string.
pub fn remove_pairs(bytes: &Vec<u8>) -> std::string::String {
    remove_pairs_without(bytes, None)
}

/// For each letter, try to remove all the pairs from the input without that
/// letter. Returns the size of the minimal string.
pub fn try_remove_pairs(bytes: &Vec<u8>) -> usize {
    (('A' as u8)..('Z' as u8 + 1))
        .map(|c| remove_pairs_without(bytes, Some(c)).len())
        .min()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_pair() {
        assert_eq!(is_pair(None, Some(32)), false);
        assert_eq!(is_pair(Some('a' as u8), Some('A' as u8)), true);
        assert_eq!(is_pair(Some('A' as u8), Some('a' as u8)), true);
        assert_eq!(is_pair(Some('a' as u8), Some('B' as u8)), false);
    }

    fn prepare_input(string: &str) -> Vec<u8> {
        string.bytes().collect()
    }

    #[test]
    fn test_remove_pairs_from_list() {
        assert_eq!(remove_pairs(&prepare_input("aA")), "".to_string());
        assert_eq!(remove_pairs(&prepare_input("abBA")), "".to_string());
        assert_eq!(remove_pairs(&prepare_input("abAB")), "abAB".to_string());
        assert_eq!(remove_pairs(&prepare_input("aabAAB")), "aabAAB".to_string());
        assert_eq!(
            remove_pairs(&prepare_input("dabAcCaCBAcCcaDA")),
            "dabCBAcaDA".to_string()
        );
        assert_eq!(remove_pairs(&prepare_input("abcdDCBA")), "".to_string());
    }
    #[test]
    fn test_try_remove_pairs_from_list() {
        assert_eq!(try_remove_pairs(&prepare_input("dabAcCaCBAcCcaDA")), 4);
    }
}
