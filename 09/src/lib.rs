/// Initial implementation used a vector. However, insert and delete at random positions are very
/// expensive in a vector, so I switched to a linked list.
use linked_list::{LinkedList, Cursor};

/// The linked list implementation I used has a Cursor that sits between 2 elements. The list
/// itself is circular, with a "ghost" of None between the tail and the head. Here, we're not
/// interested in the ghost, so this method helps us seek backward while also skipping over the
/// ghost. The cursor never has the ghost as next.
fn seek_backward_no_ghost<T>(cursor: &mut Cursor<T>, skip: usize) {
    for _ in 0..skip {
        cursor.prev();
        if cursor.peek_next().is_none() {
            cursor.prev();
        }
    }
}

/// Same for forward.
fn seek_forward_no_ghost<T>(cursor: &mut Cursor<T>, skip: usize) {
    for _ in 0..skip {
        if cursor.next().is_none() {
            cursor.next();
        }
    }
}

/// Implement the game, and return the score of the winner.
pub fn get_winner(num_players: usize, last_marble: usize) -> usize {
    assert!(num_players > 0);
    assert!(last_marble > 1);
    let mut player_score = (0..num_players).map(|_| 0).collect::<Vec<_>>();
    // The marbles, in a circular linked list.
    let mut marbles = LinkedList::new();
    // Skip the initial turn because it's a special case.
    marbles.push_back(0);
    marbles.push_back(1);
    // The cursor will always be just before the current marble (and never before the ghost).
    let mut current_marble = marbles.cursor();
    // Point to 1.
    current_marble.next();
    // We start at turn 2, to the last turn inclusive.
    for turn in 2..=last_marble {
        if turn % 23 == 0 {
            let player = turn % num_players;
            seek_backward_no_ghost(&mut current_marble, 7);
            // It's safe to unwrap because the cursor never has the ghost as next.
            player_score[player] += turn + current_marble.remove().unwrap();
        } else {
            seek_forward_no_ghost(&mut current_marble, 2);
            current_marble.insert(turn);
        }
    }

    // It's safe to unwrap, because there is at least one player.
    *player_score.iter().max().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_winner() {
        assert_eq!(get_winner(9, 25), 32);
        assert_eq!(get_winner(10, 1618), 8317);
        assert_eq!(get_winner(13, 7999), 146373);
        assert_eq!(get_winner(17, 1104), 2764);
        assert_eq!(get_winner(21, 6111), 54718);
        assert_eq!(get_winner(30, 5807), 37305);
    }
}
