#[macro_use]
extern crate nom;

use std::string::String;
use std::collections::HashSet;

use crate::parse::Claim;
use crate::vec2d::Vec2D;

mod parse;
mod vec2d;


/// Parse all the lines as claims.
fn parse_lines(lines: &Vec<String>) -> Vec<Claim> {
    lines.iter().map(parse::parse_claim).collect()
}

/// State of a square inch of the fabric.
#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    UNCLAIMED,
    CLAIMED,
    /// Claimed by more than 1 elf.
    OVERLAPPING,
}

/// Claim a cell. It updates the cell, the set of non-overlapping ids, the size of the overlapping
/// area, and whether the claim is overlapping anything.
fn claim_cell(
    cell: &mut (State, Vec<usize>),
    non_overlapping_ids: &mut HashSet<usize>,
    overlapping: &mut usize,
    is_overlapping: &mut bool,
) {
    cell.0 = match cell.0 {
        State::UNCLAIMED => State::CLAIMED,
        State::CLAIMED => {
            *is_overlapping = true;
            // Increase the overlapping area.
            *overlapping += 1;
            // Remove the claim that previously had the cell from the set of
            // non-overlapping claims, if it was there.
            non_overlapping_ids.remove(&cell.1[0]);
            State::OVERLAPPING
        }
        State::OVERLAPPING => {
            *is_overlapping = true;
            State::OVERLAPPING
        }
    }
}

pub fn find_overlapping_area(lines: &Vec<String>) -> (usize, Option<usize>) {
    let claims = parse_lines(lines);
    let max_x = claims
        .iter()
        .map(|c| c.coordinates.0 + c.size.0)
        .max()
        .unwrap_or(0);
    let max_y = claims
        .iter()
        .map(|c| c.coordinates.1 + c.size.1)
        .max()
        .unwrap_or(0);
    let mut board: Vec2D<(State, Vec<usize>)> =
        Vec2D::from_fn(max_x, max_y, &|| (State::UNCLAIMED, vec![]));
    let mut overlapping = 0;
    let mut non_overlapping_ids: HashSet<usize> = HashSet::new();
    for claim in claims {
        let mut is_overlapping = false;
        for x in claim.coordinates.0..(claim.coordinates.0 + claim.size.0) {
            for y in claim.coordinates.1..(claim.coordinates.1 + claim.size.1) {
                claim_cell(
                    &mut board[(x, y)],
                    &mut non_overlapping_ids,
                    &mut overlapping,
                    &mut is_overlapping,
                );
                board[(x, y)].1.push(claim.id);
            }
        }
        if !is_overlapping {
            non_overlapping_ids.insert(claim.id);
        }
    }
    let mut non_overlapping_claim = None;
    if non_overlapping_ids.len() == 1 {
        non_overlapping_claim = Some(
            *non_overlapping_ids.iter().next().unwrap()
        )
    }
    (overlapping, non_overlapping_claim)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int32() {
        let lines: Vec<String> = vec![
            "#1 @ 1,3: 4x4".to_string(),
            "#2 @ 3,1: 4x4".to_string(),
            "#3 @ 5,5: 2x2".to_string(),
        ];
        assert_eq!(
            parse_lines(&lines),
            vec![
                Claim {
                    id: 1,
                    coordinates: (1, 3),
                    size: (4, 4),
                },
                Claim {
                    id: 2,
                    coordinates: (3, 1),
                    size: (4, 4),
                },
                Claim {
                    id: 3,
                    coordinates: (5, 5),
                    size: (2, 2),
                },
            ]
        );
    }

    #[test]
    fn test_find_overlapping_area() {
        let lines: Vec<String> = vec![
            "#1 @ 1,3: 4x4".to_string(),
            "#2 @ 3,1: 4x4".to_string(),
            "#3 @ 5,5: 2x2".to_string(),
        ];
        assert_eq!(find_overlapping_area(&lines), 4);
    }
}
