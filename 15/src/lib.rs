#[macro_use]
extern crate nom;
#[macro_use]
extern crate derive_more;

extern crate boolinator;

mod path;

use boolinator::Boolinator;
use itertools::Itertools;
use ndarray::Array2;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Cell {
    Empty,
    Wall,
    Elf(u8, usize),
    Goblin(u8, usize),
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

/// Opposite direction.
impl std::ops::Neg for Direction {
    type Output = Self;
    fn neg(self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Right => Direction::Left,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
        }
    }
}

#[derive(
    Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Add, AddAssign, Neg, Sub, SubAssign,
)]
pub struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(c: (i32, i32)) -> Self {
        Self { x: c.0, y: c.1 }
    }
    #[allow(clippy::cast_sign_loss)]
    fn ux(self) -> usize {
        self.x as usize
    }
    #[allow(clippy::cast_sign_loss)]
    fn uy(self) -> usize {
        self.y as usize
    }
}

impl From<Direction> for Point {
    fn from(dir: Direction) -> Self {
        match dir {
            Direction::Up => Self::new((-1, 0)),
            Direction::Right => Self::new((0, 1)),
            Direction::Down => Self::new((1, 0)),
            Direction::Left => Self::new((0, -1)),
        }
    }
}

impl From<(usize, usize)> for Point {
    fn from(coords: (usize, usize)) -> Self {
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        Self {
            x: coords.0 as i32,
            y: coords.1 as i32,
        }
    }
}

impl std::ops::Add<Direction> for Point {
    type Output = Self;
    fn add(self, dir: Direction) -> Self::Output {
        self + Self::from(dir)
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    cells: Array2<Cell>,
}

impl std::ops::Index<Point> for Board {
    type Output = Cell;
    fn index(&self, p: Point) -> &Self::Output {
        &self.cells[(p.ux(), p.uy())]
    }
}

impl std::ops::IndexMut<Point> for Board {
    fn index_mut(&mut self, p: Point) -> &mut Self::Output {
        &mut self.cells[(p.ux(), p.uy())]
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use crate::Cell::*;
        write!(
            f,
            "{}",
            self.cells
                .outer_iter()
                .map(|row| row
                    .iter()
                    .map(|c| match c {
                        Elf(_, _) => "E",
                        Goblin(_, _) => "G",
                        Wall => "#",
                        Empty => " ",
                    })
                    .collect::<std::string::String>()
                    + &"   ".to_owned()
                    + &row
                        .iter()
                        .filter_map(|&c| is_unit(c).as_some_from(|| get_health(c).to_string()))
                        .join(","))
                .join("\n")
        )
    }
}

/// Directions in reading order.
const DIRECTIONS: [Direction; 4] = [
    Direction::Up,
    Direction::Left,
    Direction::Right,
    Direction::Down,
];

named!(parse_cell <&[u8], Cell>,
       map!(
           one_of!([b'#', b'E', b'G', b'.'].as_ref()),
           |c| match c {
               '.' => Cell::Empty,
               '#' => Cell::Wall,
               'E' => Cell::Elf(200, 0),
               'G' => Cell::Goblin(200, 0),
               _ => unreachable!(),
           }
        )
);

named!(parse_line <&[u8], Vec<Cell>>,
    terminated!(many1!(parse_cell), char!('\n'))
);

named!(parse_lines <&[u8], Vec<Vec<Cell>>>,
    many1!(complete!(parse_line))
);

fn assign_id(counter: &mut usize, c: Cell) -> Cell {
    use crate::Cell::*;
    match c {
        Elf(a, _) => {
            *counter += 1;
            Elf(a, *counter - 1)
        }
        Goblin(a, _) => {
            *counter += 1;
            Goblin(a, *counter - 1)
        }
        e => e,
    }
}

/// Parse the whole input into a grid.
pub fn parse_board(input: &[u8]) -> Result<Board, nom::Err<&[u8]>> {
    let lines = parse_lines(input)?.1;
    let mut counter = 0;
    Ok(Board {
        cells: Array2::from_shape_vec(
            (lines.len(), lines[0].len()),
            lines
                .into_iter()
                .flat_map(|row| row.into_iter())
                .map(|c| assign_id(&mut counter, c))
                .collect::<Vec<_>>(),
        )
        .unwrap(),
    })
}

fn is_unit(c: Cell) -> bool {
    match c {
        Cell::Elf(_, _) | Cell::Goblin(_, _) => true,
        _ => false,
    }
}

fn is_elf(c: Cell) -> bool {
    assert!(is_unit(c));
    match c {
        Cell::Elf(_, _) => true,
        _ => false,
    }
}

fn is_enemy(c1: Cell, c2: Cell) -> bool {
    is_unit(c1) && is_unit(c2) && is_elf(c1) != is_elf(c2)
}

fn find_targets(board: &Board, unit: Cell) -> impl Iterator<Item = Point> + '_ {
    assert!(is_unit(unit));
    board
        .cells
        .indexed_iter()
        .filter_map(move |(p, &c)| is_enemy(unit, c).as_some(Point::from(p)))
}

fn find_nearby_enemies(board: &Board, position: Point) -> Vec<Point> {
    let unit = board[position];
    DIRECTIONS
        .iter()
        .filter_map(|&d| {
            let neighbor = position + d;
            is_enemy(unit, board[neighbor]).as_some(neighbor)
        })
        .collect()
}

fn find_enemy_free_spot<'a>(
    board: &'a Board,
    targets: &'a [Point],
) -> impl Iterator<Item = Point> + 'a {
    targets.iter().flat_map(move |&t| {
        DIRECTIONS
            .iter()
            .filter_map(move |&d| (board[t + d] == Cell::Empty).as_some(t + d))
    })
}

fn next_step(board: &Board, targets: &[Point], start: Point) -> Option<Point> {
    path::shortest_path_step(board, start, targets)
}

fn attack(board: &mut Board, positions: &[Point], attack_power: u8) {
    use crate::Cell::*;
    let min_health = positions
        .iter()
        .map(|&p| get_health(board[p]))
        .min()
        .unwrap();
    let target_position = *positions
        .iter()
        .find(|&p| get_health(board[*p]) == min_health)
        .unwrap();
    let target = &mut board[target_position];
    assert!(is_unit(*target));
    *target = match target {
        Elf(a, id) => {
            if *a > 3 {
                Elf(*a - 3, *id)
            } else {
                Empty
            }
        }
        Goblin(a, id) => {
            if *a > attack_power {
                Goblin(*a - attack_power, *id)
            } else {
                Empty
            }
        }

        _ => unreachable!(),
    }
}

/// Returns whether the fight is over.
fn update_unit(board: &mut Board, position: Point, attack_power: u8) -> bool {
    let targets = find_targets(board, board[position])
        .peekable()
        .collect::<Vec<_>>();
    if targets.is_empty() {
        return true;
    }
    let mut nearby_ennemies = find_nearby_enemies(board, position);
    if nearby_ennemies.is_empty() {
        let goal_positions = find_enemy_free_spot(board, &targets).collect::<Vec<_>>();
        if let Some(new_pos) = next_step(board, &goal_positions, position) {
            board[new_pos] = board[position];
            board[position] = Cell::Empty;
            nearby_ennemies = find_nearby_enemies(board, new_pos);
        }
    }
    if !nearby_ennemies.is_empty() {
        attack(board, &nearby_ennemies, attack_power);
    }
    false
}

fn get_unit_id(c: Cell) -> Option<usize> {
    match c {
        Cell::Elf(_, id) | Cell::Goblin(_, id) => Some(id),
        _ => None,
    }
}

pub fn update_all_units(board: &mut Board, attack_power: u8) -> bool {
    let units_to_update = board
        .cells
        .indexed_iter()
        .filter_map(|(p, &c)| get_unit_id(c).map(|id| (Point::from(p), id)))
        .collect::<HashMap<_, _>>();
    let coords = board
        .cells
        .indexed_iter()
        .map(|(p, _)| Point::from(p))
        .collect::<Vec<_>>();
    coords.iter().any(|&c| {
        if let Some(&id) = units_to_update.get(&c) {
            if let Some(uid) = get_unit_id(board[c]) {
                if uid == id {
                    return update_unit(board, c, attack_power);
                }
            }
        }
        false
    })
}

fn get_health(c: Cell) -> u8 {
    assert!(is_unit(c));
    match c {
        Cell::Elf(a, _) | Cell::Goblin(a, _) => a,
        _ => unreachable!(),
    }
}

pub fn compute_battle_score(board: &mut Board, attack_power: u8) -> usize {
    for counter in 1.. {
        if update_all_units(board, attack_power) {
            println!("Turn {}: \n{}", counter, board);
            return board
                .cells
                .iter()
                .filter_map(|&c| is_unit(c).as_some_from(|| get_health(c) as usize))
                .sum::<usize>()
                * (counter - 1);
        }
    }
    panic!("Still not finished?");
}

fn count_elves(board: &Board) -> usize {
    board
        .cells
        .iter()
        .filter(|&&c| is_unit(c) && is_elf(c))
        .count()
}

pub fn find_min_attacking_power_score(board: &Board) -> usize {
    for power in 4.. {
        let mut new_board = board.clone();
        let elves_before = count_elves(&new_board);
        let battle_score = compute_battle_score(&mut new_board, power);
        if elves_before == count_elves(&new_board) {
            return battle_score;
        }
    }
    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! nom_assert_eq {
        ($parse_expr:expr, $expected:expr) => {
            assert_eq!($parse_expr.unwrap().1, $expected);
        };
    }

    #[test]
    fn parse_cell_test() {
        nom_assert_eq!(parse_cell(b"#"), Cell::Wall);
    }

    const TEST_INPUT: &[u8] = b"#######
#.G...#
#...EG#
#.#.#G#
#..G#E#
#.....#
#######\n";

    #[test]
    fn parse_test() {
        use super::Cell::*;
        nom_assert_eq!(
            parse_lines(TEST_INPUT),
            vec![
                vec![Wall, Wall, Wall, Wall, Wall, Wall, Wall],
                vec![Wall, Empty, Goblin(200, 0), Empty, Empty, Empty, Wall],
                vec![Wall, Empty, Empty, Empty, Elf(200, 0), Goblin(200, 0), Wall],
                vec![Wall, Empty, Wall, Empty, Wall, Goblin(200, 0), Wall],
                vec![Wall, Empty, Empty, Goblin(200, 0), Wall, Elf(200, 0), Wall],
                vec![Wall, Empty, Empty, Empty, Empty, Empty, Wall],
                vec![Wall, Wall, Wall, Wall, Wall, Wall, Wall]
            ]
        );
    }

    fn whole_fight(input: &[u8], score: usize) {
        let mut board = parse_board(input).expect("Parsing board failed: ");
        assert_eq!(compute_battle_score(&mut board, 3), score);
    }

    #[test]
    fn whole_fight_test() {
        whole_fight(TEST_INPUT, 27730);
        whole_fight(
            b"#######
#G..#E#
#E#E.E#
#G.##.#
#...#E#
#...E.#
#######\n",
            36334,
        );
        whole_fight(
            b"#######
#E..EG#
#.#G.E#
#E.##E#
#G..#.#
#..E#.#
#######\n",
            39514,
        );

        whole_fight(
            b"#######
#E.G#.#
#.#G..#
#G.#.G#
#G..#.#
#...E.#
#######\n",
            27755,
        );
        whole_fight(
            b"#######
#.E...#
#.#..G#
#.###.#
#E#G#G#
#...#G#
#######\n",
            28944,
        );
        whole_fight(
            b"#########
#G......#
#.E.#...#
#..##..G#
#...##..#
#...#...#
#.G...G.#
#.....G.#
#########\n",
            18740,
        );
    }

    #[test]
    fn attack_power_test() {
        let board = parse_board(TEST_INPUT).expect("Parsing board failed: ");
        assert_eq!(find_min_attacking_power_score(&board), 4988);
    }
}
