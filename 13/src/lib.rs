#![feature(vec_remove_item)]
#![feature(drain_filter)]

#[macro_use]
extern crate nom;
#[macro_use]
extern crate derive_more;

#[allow(unused_imports)]
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

use std::cmp::{Ordering, PartialOrd};
use std::collections::{HashMap, HashSet};

/// The contents of one cell of the board, without carts.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Cell {
    Empty,
    Vertical,
    Horizontal,
    Intersection,
    CornerSlash,
    CornerBackSlash,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Direction {
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

/// Directions in clockwise order.
const DIRECTIONS: [Direction; 4] = [
    Direction::Up,
    Direction::Right,
    Direction::Down,
    Direction::Left,
];

/// Grid of cells and cart direction.
type Grid = Vec<Vec<(Cell, Option<Direction>)>>;
type GridSlice = [Vec<(Cell, Option<Direction>)>];

/// Parse a character into a cell and an optional cart direction.
named!(parse_cell <&str, (Cell, Option<Direction>)>,
       map!(alt!(
         char!(' ') |
         char!('|') |
         char!('-') |
         char!('+') |
         char!('/') |
         char!('\\') |
         char!('^') |
         char!('>') |
         char!('v') |
         char!('<')),
         |c| match c {
             ' ' => (Cell::Empty, None),
             '|' => (Cell::Vertical, None),
             '-' => (Cell::Horizontal, None),
             '+' => (Cell::Intersection, None),
             '/' => (Cell::CornerSlash, None),
             '\\' => (Cell::CornerBackSlash, None),
             '^' => (Cell::Vertical, Some(Direction::Up)),
             '>' => (Cell::Horizontal, Some(Direction::Right)),
             'v' => (Cell::Vertical, Some(Direction::Down)),
             '<' => (Cell::Horizontal, Some(Direction::Left)),
             _ => unreachable!(),
         })
);

named!(parse_line <&str, Vec<(Cell, Option<Direction>)>>,
    terminated!(many1!(parse_cell), char!('\n'))
);

named!(parse_lines <&str, Grid>,
    many1!(complete!(parse_line))
);

/// Parse the whole input into a grid.
fn parse_all(input: &str) -> Result<Grid, nom::Err<&str>> {
    parse_lines(input).map(|r| r.1)
}

/// Id of a node in the turn graph.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct NodeId(usize);

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Add, AddAssign, Neg, Sub, SubAssign)]
pub struct Coordinates {
    x: i32,
    y: i32,
}

/// Coordinates are ordered by row, col.
impl PartialOrd for Coordinates {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.y.cmp(&other.y) {
            Ordering::Equal => Some(self.x.cmp(&other.x)),
            ord => Some(ord),
        }
    }
}

impl Ord for Coordinates {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Coordinates {
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

/// Node in the turn graph.
#[derive(Debug, PartialEq, Eq)]
struct Node {
    /// Can only be corner or intersection.
    cell: Cell,
    coords: Coordinates,
    /// Neighbors in the graph, and in which direction they are.
    neighbors: Vec<(Direction, NodeId)>,
}

/// Turn graph.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Board {
    nodes: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cart {
    direction: Direction,
    position: Coordinates,
    /// Which node in the turn graph it will hit next.
    target: NodeId,
    target_coord: Coordinates,
    /// How it will turn at the next intersection: -1: CCW, 0: straight, 1: CW.
    next_turn: i8,
    is_dead: bool,
}

impl Cart {
    fn new(
        direction: Direction,
        position: Coordinates,
        target: NodeId,
        target_coord: Coordinates,
    ) -> Self {
        Self {
            direction,
            position,
            target,
            target_coord,
            next_turn: -1,
            is_dead: false,
        }
    }
}

/// Is `cell` a turning point?
fn is_node_cell(cell: Cell) -> bool {
    match cell {
        Cell::CornerBackSlash | Cell::CornerSlash | Cell::Intersection => true,
        _ => false,
    }
}

/// Returns the directions you can go from a slash corner, if you are coming from the down
/// direction.
const SLASH_DOWN_DIRECTIONS: [Direction; 2] = [Direction::Down, Direction::Right];
const SLASH_UP_DIRECTIONS: [Direction; 2] = [Direction::Up, Direction::Left];
const BACK_SLASH_DOWN_DIRECTIONS: [Direction; 2] = [Direction::Down, Direction::Left];
const BACK_SLASH_UP_DIRECTIONS: [Direction; 2] = [Direction::Up, Direction::Right];
const INTERSECTION_DIRECTIONS: [Direction; 4] = DIRECTIONS;

/// Get the directions in which you can go after reaching a node, travelling in
/// `incoming_direction`.
fn get_directions_for(
    cell: Cell,
    incoming_direction: Direction,
) -> impl Iterator<Item = Direction> {
    let directions: &[Direction] = match cell {
        Cell::CornerSlash => match incoming_direction {
            Direction::Down | Direction::Right => &SLASH_UP_DIRECTIONS,
            Direction::Up | Direction::Left => &SLASH_DOWN_DIRECTIONS,
        },
        Cell::CornerBackSlash => match incoming_direction {
            Direction::Down | Direction::Left => &BACK_SLASH_UP_DIRECTIONS,
            Direction::Up | Direction::Right => &BACK_SLASH_DOWN_DIRECTIONS,
        },
        Cell::Intersection => &INTERSECTION_DIRECTIONS,
        _ => panic!("Not interesting node considered as a node"),
    };
    directions.iter().cloned()
}

/// Direction -> Coordinates.
fn coords_for(dir: Direction) -> Coordinates {
    match dir {
        Direction::Up => Coordinates::new((0, -1)),
        Direction::Right => Coordinates::new((1, 0)),
        Direction::Down => Coordinates::new((0, 1)),
        Direction::Left => Coordinates::new((-1, 0)),
    }
}

/// Visit nodes in DFS.
fn visit(
    coords: Coordinates,
    grid: &GridSlice,
    board: &mut Board,
    visited: &mut HashMap<Coordinates, NodeId>,
    seen_carts: &mut HashSet<Coordinates>,
    carts: &mut Vec<Cart>,
    incoming_direction: Direction,
) -> NodeId {
    if let Some(n) = visited.get(&coords) {
        return *n;
    }
    let cell = grid[coords.uy()][coords.ux()].0;
    // Insert the node.
    let node_id = NodeId(board.nodes.len());
    visited.insert(coords, node_id);
    board.nodes.push(Node {
        cell,
        coords,
        neighbors: Vec::new(),
    });
    // Explore the graph from the node.
    board.nodes[node_id.0].neighbors = get_directions_for(cell, incoming_direction)
        .map(|d| {
            let dir = coords_for(d);
            let mut local_carts: Vec<Coordinates> = Vec::new();
            // Go in the new direction until we find a node.
            let new_node_coords = itertools::iterate(coords + dir, |c| *c + dir)
                .find(|&iter_coords| {
                    let iter_cell = grid[iter_coords.uy()][iter_coords.ux()];
                    if is_node_cell(iter_cell.0) {
                        // Found the node.
                        return true;
                    }
                    if let Some(direction) = iter_cell.1 {
                        // Found a car.
                        if !seen_carts.contains(&iter_coords) {
                            seen_carts.insert(iter_coords);
                            if direction == d {
                                // If it's going the same direction, we don't yet know the target.
                                local_carts.push(iter_coords);
                            } else {
                                // If it's going the opposite direction, it's going where we're
                                // coming from.
                                carts.push(Cart::new(direction, iter_coords, node_id, coords));
                            }
                        }
                    }
                    // Not a node.
                    false
                })
                .unwrap();
            // Recurse into the new node.
            let new_node = visit(new_node_coords, grid, board, visited, seen_carts, carts, d);
            for c in local_carts {
                carts.push(Cart::new(d, c, new_node, new_node_coords))
            }
            (d, new_node)
        })
        .collect::<Vec<_>>();
    node_id
}

/// Parse the board structure and the list of carts from the input.
pub fn parse_board(input: &str) -> Result<(Board, Vec<Cart>), nom::Err<&str>> {
    let grid = parse_all(input)?;
    let mut board = Board::default();
    let mut visited = HashMap::<Coordinates, NodeId>::new();
    let mut seen_carts = HashSet::<Coordinates>::new();
    let mut carts = Vec::new();
    grid.iter().enumerate().for_each(|(y, v)| {
        v.iter().enumerate().for_each(|(x, (c, _))| {
            // The first node is always going to be a slash (top-left).
            if *c == Cell::CornerSlash {
                #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
                visit(
                    Coordinates::new((x as i32, y as i32)),
                    &grid,
                    &mut board,
                    &mut visited,
                    &mut seen_carts,
                    &mut carts,
                    // Assume we were going left when starting.
                    Direction::Left,
                );
            }
        })
    });
    Ok((board, carts))
}

struct CartTracker {
    /// List of carts.
    carts: Vec<Cart>,
    /// Map of position to cart.
    cart_positions: HashMap<Coordinates, usize>,
}

/// Returns the direction rotated by `rotate`, which is -1, 0, or 1.
fn rotate_direction(dir: Direction, rotate: i8) -> Direction {
    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap
    )]
    DIRECTIONS
        [((DIRECTIONS.iter().position(|d| *d == dir).unwrap() as i8 + 4 + rotate) % 4) as usize]
}

/// When a cart comes up to a node, find the next direction, potentially updating its `next_turn`.
fn find_next_direction(
    turn_cell: Cell,
    init_direction: Direction,
    next_turn: &mut i8,
) -> Direction {
    match turn_cell {
        Cell::Intersection => {
            let dir = rotate_direction(init_direction, *next_turn);
            *next_turn = (*next_turn + 2) % 3 - 1;
            dir
        }
        Cell::CornerSlash => match init_direction {
            Direction::Up => Direction::Right,
            Direction::Down => Direction::Left,
            Direction::Right => Direction::Up,
            Direction::Left => Direction::Down,
        },
        Cell::CornerBackSlash => match init_direction {
            Direction::Up => Direction::Left,
            Direction::Down => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Left => Direction::Up,
        },
        e => panic!("Should not get there: {:?}", e),
    }
}

impl CartTracker {
    /// Move a cart by one tick, check for collisions and return a potential collision site.
    fn move_cart(&mut self, i: usize, board: &Board) -> Option<Coordinates> {
        if self.carts[i].is_dead {
            return None;
        }
        // Remove the cart from the position map.
        self.cart_positions.remove(&self.carts[i].position).unwrap();

        {
            let dir = coords_for(self.carts[i].direction);
            self.carts[i].position += dir;
        }

        let new_pos = self.carts[i].position;
        if let Some(&c) = self.cart_positions.get(&new_pos) {
            // Collision, mark both as dead.
            self.cart_positions.remove(&new_pos).unwrap();
            self.carts[c].is_dead = true;
            self.carts[i].is_dead = true;
            return Some(new_pos);
        }
        // No collision, we can proceed.
        self.cart_positions.insert(new_pos, i);
        let cart = &mut self.carts[i];
        // Update the cart.
        if cart.position == cart.target_coord {
            let target = &board.nodes[cart.target.0];
            cart.direction = find_next_direction(target.cell, cart.direction, &mut cart.next_turn);
            cart.target = target
                .neighbors
                .iter()
                .find(|(d, _)| *d == cart.direction)
                .unwrap()
                .1;
            cart.target_coord = board.nodes[cart.target.0].coords;
        }
        None
    }

    /// Get the index order in which the cart will be updated.
    fn get_cart_index(carts: &[Cart]) -> Vec<usize> {
        let mut carts_index: Vec<usize> = (0..carts.len()).collect();
        carts_index.sort_by(|&i, &j| carts[i].position.cmp(&carts[j].position));
        carts_index
    }
}

/// Move carts by one tick, returning the first potential collision.
fn tick(board: &Board, tracker: &mut CartTracker) -> Option<Coordinates> {
    let carts_index = CartTracker::get_cart_index(&tracker.carts);
    carts_index
        .iter()
        .flat_map(|i| tracker.move_cart(*i, board))
        // Consume all the elements to make sure all the carts are moved.
        .collect::<Vec<_>>()
        .get(0)
        .cloned()
}

/// Find the first collision when running the carts. It returns the coordinates of the first
/// collision, and the state of all the carts.
pub fn find_first_collision(board: &Board, carts: Vec<Cart>) -> (Vec<Cart>, Coordinates) {
    let mut cart_tracker = CartTracker {
        cart_positions: carts
            .iter()
            .enumerate()
            .map(|(i, c)| (c.position, i))
            .collect(),
        carts,
    };
    loop {
        #[cfg(test)]
        {
            tests::print_board(board, &cart_tracker.carts);
        }
        if let Some(c) = tick(board, &mut cart_tracker) {
            return (cart_tracker.carts, c);
        }
    }
}

/// Find the last remaining cart after all others have collided.
pub fn find_remaining_cart(board: &Board, carts: Vec<Cart>) -> Coordinates {
    let mut cart_tracker = CartTracker {
        cart_positions: carts
            .iter()
            .enumerate()
            .filter_map(|(i, c)| {
                if c.is_dead {
                    None
                } else {
                    Some((c.position, i))
                }
            })
            .collect(),
        carts,
    };
    loop {
        // Remove dead carts.
        for _ in cart_tracker.carts.drain_filter(|c| c.is_dead) {}
        #[cfg(test)]
        {
            tests::print_board(board, &cart_tracker.carts);
        }
        if tick(board, &mut cart_tracker).is_some()
            && cart_tracker
                .carts
                .iter()
                .filter(|c| !c.is_dead)
                .enumerate()
                .last()
                .unwrap()
                .0
                == 0
        {
            #[cfg(test)]
            {
                tests::print_board(board, &cart_tracker.carts);
            }
            return cart_tracker
                .carts
                .iter()
                .find(|c| !c.is_dead)
                .unwrap()
                .position;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! nom_assert_eq {
        ($parse_expr:expr, $expected:expr) => {
            assert_eq!($parse_expr.unwrap().1, $expected);
        };
    }

    pub fn print_board(board: &Board, carts: &[Cart]) {
        let max_x = board.nodes.iter().map(|n| n.coords.x).max().unwrap();
        let max_y = board.nodes.iter().map(|n| n.coords.y).max().unwrap();
        for y in 0..=max_y {
            for x in 0..=max_x {
                let coords = Coordinates { x, y };
                if let Some(c) = carts.iter().find(|c| c.position == coords && !c.is_dead) {
                    print!(
                        "{}",
                        match c.direction {
                            Direction::Up => "^",
                            Direction::Down => "v",
                            Direction::Right => ">",
                            Direction::Left => "<",
                        }
                    );
                } else if let Some(n) = board.nodes.iter().find(|n| n.coords == coords) {
                    print!(
                        "{}",
                        match n.cell {
                            Cell::CornerBackSlash => "\\",
                            Cell::CornerSlash => "/",
                            Cell::Intersection => "+",
                            _ => panic!("What?"),
                        }
                    );
                } else {
                    print!(" ");
                }
            }
            println!();
        }
        println!();
    }

    #[test]
    fn parse_cell_test() {
        nom_assert_eq!(parse_cell(" "), (Cell::Empty, None));
        nom_assert_eq!(parse_cell("+"), (Cell::Intersection, None));
        nom_assert_eq!(parse_cell("^"), (Cell::Vertical, Some(Direction::Up)));
    }

    #[test]
    fn parse_line_test() {
        nom_assert_eq!(
            parse_line("/->^\\\n"),
            vec![
                (Cell::CornerSlash, None),
                (Cell::Horizontal, None),
                (Cell::Horizontal, Some(Direction::Right)),
                (Cell::Vertical, Some(Direction::Up)),
                (Cell::CornerBackSlash, None),
            ]
        );
    }
    const TEST_INPUT: &str = "
/<->-\\
|    |
|    ^
v/---/
\\/    \n";

    #[test]
    fn parse_board_test() {
        // Ignore the first newline, it's only there to align the text.
        assert_eq!(
            parse_board(&TEST_INPUT[1..]).unwrap(),
            (
                Board {
                    nodes: vec![
                        Node {
                            cell: Cell::CornerSlash,
                            coords: Coordinates { x: 0, y: 0 },
                            neighbors: vec![
                                (Direction::Down, NodeId(1)),
                                (Direction::Right, NodeId(5))
                            ],
                        },
                        Node {
                            cell: Cell::CornerBackSlash,
                            coords: Coordinates { x: 0, y: 4 },
                            neighbors: vec![
                                (Direction::Up, NodeId(0)),
                                (Direction::Right, NodeId(2))
                            ],
                        },
                        Node {
                            cell: Cell::CornerSlash,
                            coords: Coordinates { x: 1, y: 4 },
                            neighbors: vec![
                                (Direction::Up, NodeId(3)),
                                (Direction::Left, NodeId(1))
                            ],
                        },
                        Node {
                            cell: Cell::CornerSlash,
                            coords: Coordinates { x: 1, y: 3 },
                            neighbors: vec![
                                (Direction::Down, NodeId(2)),
                                (Direction::Right, NodeId(4))
                            ],
                        },
                        Node {
                            cell: Cell::CornerSlash,
                            coords: Coordinates { x: 5, y: 3 },
                            neighbors: vec![
                                (Direction::Up, NodeId(5)),
                                (Direction::Left, NodeId(3))
                            ],
                        },
                        Node {
                            cell: Cell::CornerBackSlash,
                            coords: Coordinates { x: 5, y: 0 },
                            neighbors: vec![
                                (Direction::Down, NodeId(4)),
                                (Direction::Left, NodeId(0))
                            ],
                        },
                    ]
                },
                vec![
                    Cart::new(
                        Direction::Right,
                        Coordinates { x: 3, y: 0 },
                        NodeId(5),
                        Coordinates { x: 5, y: 0 },
                    ),
                    Cart::new(
                        Direction::Left,
                        Coordinates { x: 1, y: 0 },
                        NodeId(0),
                        Coordinates { x: 0, y: 0 },
                    ),
                    Cart::new(
                        Direction::Up,
                        Coordinates { x: 5, y: 2 },
                        NodeId(5),
                        Coordinates { x: 5, y: 0 },
                    ),
                    Cart::new(
                        Direction::Down,
                        Coordinates { x: 0, y: 3 },
                        NodeId(1),
                        Coordinates { x: 0, y: 4 },
                    ),
                ]
            )
        );
    }
    #[test]
    fn find_first_collision_test() {
        // Ignore the first newline, it's only there to align the text.
        let (board, carts) = parse_board(&TEST_INPUT[1..]).unwrap();
        assert_eq!(
            find_first_collision(&board, carts).1,
            Coordinates { x: 5, y: 0 }
        );
    }

    #[test]
    fn full_test() {
        let input = [
            "/>--\\        ",
            "|   |  /----\\",
            "| /-+--+-\\  |",
            "| | |  | v  |",
            "\\-+-/  \\-+--/",
            "  \\------/   \n",
        ]
        .join("\n");
        let (board, carts) = parse_board(&input).unwrap();
        assert_eq!(
            find_first_collision(&board, carts).1,
            Coordinates { x: 7, y: 4 }
        );
    }

    #[test]
    fn find_remaining_car_test() {
        let input = [
            "/>-<\\  ",
            "|   |  ",
            "| /<+-\\",
            "| | | v",
            "\\>+</ |",
            "  |   ^",
            "  \\<->/\n",
        ]
        .join("\n");
        let (board, carts) = parse_board(&input).unwrap();
        let (carts_after, _) = find_first_collision(&board, carts);
        assert_eq!(
            find_remaining_cart(&board, carts_after),
            Coordinates { x: 6, y: 4 }
        );
    }
}
