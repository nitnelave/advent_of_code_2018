#[macro_use]
extern crate nom;

use ndarray::Array2;
use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cell {
    Empty,
    Tree,
    LumberYard,
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Empty
    }
}

named!(parse_cell <&str, Cell>,
       map!(
           one_of!([b'#', b'|', b'.'].as_ref()),
           |c| match c {
               '.' => Cell::Empty,
               '#' => Cell::LumberYard,
               '|' => Cell::Tree,
               _ => unreachable!(),
           }
        )
);

named!(parse_line <&str, Vec<Cell>>,
    terminated!(many1!(parse_cell), char!('\n'))
);

named!(parse_lines <&str, Vec<Vec<Cell>>>,
    many1!(complete!(parse_line))
);

#[derive(Clone, Hash)]
pub struct Board {
    cells: Array2<Cell>,
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.cells
                .outer_iter()
                .map(|row| row
                    .iter()
                    .map(|&c| match c {
                        Cell::Tree => '|',
                        Cell::LumberYard => '#',
                        Cell::Empty => '.',
                    })
                    .collect::<std::string::String>())
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

pub fn parse_input(input: &str) -> Result<Board, nom::Err<&str>> {
    let lines = parse_lines(input)?.1;
    let mut cells = Array2::default((lines.len() + 2, lines[0].len() + 2));
    for x in 0..lines.len() {
        for y in 0..lines[0].len() {
            cells[(x + 1, y + 1)] = lines[x][y];
        }
    }
    Ok(Board { cells })
}

fn update_cell(input: &Board, x: usize, y: usize) -> Cell {
    use crate::Cell::*;
    let mut trees = 0;
    let mut yards = 0;
    for ix in (x - 1)..=(x + 1) {
        for iy in (y - 1)..=(y + 1) {
            if ix == x && iy == y {
                continue;
            }
            match input.cells[(ix, iy)] {
                Tree => trees += 1,
                LumberYard => yards += 1,
                _ => (),
            }
        }
    }
    match input.cells[(x, y)] {
        Tree => {
            if yards >= 3 {
                LumberYard
            } else {
                Tree
            }
        }
        Empty => {
            if trees >= 3 {
                Tree
            } else {
                Empty
            }
        }
        LumberYard => {
            if yards > 0 && trees > 0 {
                LumberYard
            } else {
                Empty
            }
        }
    }
}

fn step(input: &Board, output: &mut Board) {
    assert_eq!(input.cells.shape(), output.cells.shape());
    for x in 1..(input.cells.shape()[0] - 1) {
        for y in 1..(input.cells.shape()[1] - 1) {
            output.cells[(x, y)] = update_cell(input, x, y);
        }
    }
}

fn calculate_hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

pub fn run_steps(mut input_board: Board, num_steps: usize) -> Board {
    let mut input = &mut input_board;
    let mut output = &mut input.clone();
    let mut seen_states = HashMap::new();
    #[cfg(test)]
    {
        println!("{}", input);
    }
    let mut i = 0;
    let mut cycled = false;
    while i < num_steps {
        step(input, output);
        std::mem::swap(&mut input, &mut output);
        if !cycled {
            let hash = calculate_hash(input);
            if let Some(first) = seen_states.get(&hash) {
                let cycle = i - first;
                let skip = ((num_steps - i) / cycle) * cycle;
                println!(
                    "Found cycle of length {} at step {}, skipping to {}",
                    cycle,
                    i,
                    i + skip
                );
                i += skip;
                cycled = true;
            }
            seen_states.insert(hash, i);
        }
        #[cfg(test)]
        {
            println!("{}", input);
        }
        i += 1;
    }
    input_board
}

fn count_cell(input: &Board, cell: Cell) -> usize {
    input.cells.iter().filter(|&&c| c == cell).count()
}

pub fn compute_score(input: &Board) -> usize {
    count_cell(input, Cell::Tree) * count_cell(input, Cell::LumberYard)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn score_for_test(input: &str) -> (usize, usize) {
        let board = parse_input(input).expect("Failed to parse: ");
        let new_board = run_steps(board, 10);
        let first_score = compute_score(&new_board);
        let final_board = run_steps(new_board, 1_000_000_000 - 10);
        (first_score, compute_score(&final_board))
    }

    #[test]
    fn parse_input_test() {
        assert_eq!(score_for_test(include_str!("../test_input")), (1147, 0));
    }
}
