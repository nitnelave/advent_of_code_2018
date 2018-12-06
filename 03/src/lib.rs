#[macro_use]
extern crate nom;

use std::string::String;
use std::collections::HashSet;

use itertools::Itertools;

use crate::parse::Claim;
use crate::vec2d::Vec2D;

mod parse;
mod vec2d;


/// Parse all the lines as claims.
fn parse_lines(lines: &[String]) -> Vec<Claim> {
    lines.iter().map(|l| parse::claim(l)).collect()
}

/// State of a square inch of the fabric.
#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    UNCLAIMED,
    CLAIMED,
    /// Claimed by more than 1 elf.
    OVERLAPPING,
}

/// Claim a cell. It returns the new state of the cell, whether the cell is overlapping, and
/// whether it started overlapping.
fn claim_cell(state: State) -> (State, bool, bool) {
    match state {
        State::UNCLAIMED => (State::CLAIMED, false, true),
        State::CLAIMED => (State::OVERLAPPING, true, true),
        State::OVERLAPPING => (State::OVERLAPPING, true, false),
    }
}

fn x_axis(coord: (usize, usize)) -> usize {
    coord.0
}

fn y_axis(coord: (usize, usize)) -> usize {
    coord.1
}

fn far_corner(claim: &Claim, axis: &Fn((usize, usize)) -> usize) -> usize {
    axis(claim.coordinates) + axis(claim.size)
}

/// Find the maximum coordinate of the `claims`, along the dimension provided by `axis`.
fn find_max_coordinate(claims: &[Claim], axis: &Fn((usize, usize)) -> usize) -> usize {
    claims.iter().map(|c| far_corner(c, axis)).max().unwrap_or(
        0,
    )
}

struct Board {
    cells: Vec2D<(State, Vec<usize>)>,
    overlapping: usize,
    non_overlapping_ids: HashSet<usize>,
}

impl Board {
    fn new(max_x: usize, max_y: usize) -> Self {
        Self {
            cells: Vec2D::from_fn(max_x, max_y, &|| (State::UNCLAIMED, vec![])),
            overlapping: 0,
            non_overlapping_ids: HashSet::new(),
        }
    }

    fn process_cell(&mut self, coord: (usize, usize)) -> bool {
        let (status, is_overlapping, starts_overlapping) = claim_cell(self.cells[coord].0);
        self.cells[coord].0 = status;
        if starts_overlapping {
            self.overlapping += 1;
            self.non_overlapping_ids.remove(&self.cells[coord].1[0]);
        }
        is_overlapping
    }

    fn process_claim(&mut self, claim: &Claim) {
        let mut is_overlapping = false;
        (x_axis(claim.coordinates)..far_corner(&claim, &x_axis))
            .cartesian_product(y_axis(claim.coordinates)..far_corner(&claim, &y_axis))
            .for_each(|coord| {
                is_overlapping = self.process_cell(coord);
                self.cells[coord].1.push(claim.id);
            });
        if !is_overlapping {
            self.non_overlapping_ids.insert(claim.id);
        }
    }

    fn get_result(self) -> (usize, Option<usize>) {
        (
            self.overlapping,
            self.non_overlapping_ids.into_iter().next(),
        )
    }
}


/// Find the area of overlap between the claims defined by `lines`, as well as the ID of the first
/// claim that doesn't overlap with any other.
pub fn find_overlapping_area(lines: &[String]) -> (usize, Option<usize>) {
    let claims = parse_lines(lines);
    let max_x = find_max_coordinate(&claims, &x_axis);
    let max_y = find_max_coordinate(&claims, &y_axis);
    let mut board = Board::new(max_x, max_y);
    claims.iter().for_each(|c| board.process_claim(c));
    board.get_result()
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
