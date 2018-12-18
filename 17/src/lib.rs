#[macro_use]
extern crate nom;

use ndarray::Array2;
use nom::digit;

named!(usize <&str, usize>,
       map!(digit, |d| d.parse::<usize>().unwrap())
);

/// Parse x..y
named!(range <&str, (usize, usize)>,
       separated_pair!(usize, tag_s!(".."), usize)
);

/// Parse 'x' or 'y'.
named!(is_x <&str, bool>,
       map!(alt!(char!('x') | char!('y')), |c| c == 'x')
);

#[derive(Debug, PartialEq, Eq)]
pub struct Line {
    /// If true, the structure represents a vertical line, with short_axis being the x coordinate,
    /// and long_axis the y coordinates.
    /// Otherwise, it's the other way around.
    is_vertical: bool,
    short_axis: usize,
    long_axis: (usize, usize),
}

/// Parse "x=123, y=45..67".
named!(line <&str, Line>,
       do_parse!(
           is_vertical: is_x >>
           char!('=') >>
           short_axis: usize >>
           tag_s!(", ") >>
           is_horizontal: is_x >>
           char!('=') >>
           long_axis: range >>
           ({ assert!(is_vertical != is_horizontal);
               Line { is_vertical, short_axis, long_axis }})
        )
);

named!(lines <&str, Vec<Line>>,
       many1!(complete!(terminated!(line, opt!(char!('\n')))))
);

pub fn parse_input(input: &str) -> Result<Vec<Line>, nom::Err<&str>> {
    lines(input).map(|r| r.1)
}

/// Returns the x bounds and the y lower bound.
fn find_bounds(lines: &[Line]) -> ((usize, usize), (usize, usize)) {
    let min_x = lines
        .iter()
        .map(|l| {
            if l.is_vertical {
                l.short_axis
            } else {
                l.long_axis.0
            }
        })
        .min()
        .unwrap();
    let max_x = lines
        .iter()
        .map(|l| {
            if l.is_vertical {
                l.short_axis
            } else {
                l.long_axis.1
            }
        })
        .max()
        .unwrap();
    let min_y = lines
        .iter()
        .map(|l| {
            if l.is_vertical {
                l.long_axis.0
            } else {
                l.short_axis
            }
        })
        .min()
        .unwrap();
    let max_y = lines
        .iter()
        .map(|l| {
            if l.is_vertical {
                l.long_axis.1
            } else {
                l.short_axis
            }
        })
        .max()
        .unwrap();
    ((min_x, max_x), (min_y, max_y))
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Cell {
    Sand,
    Clay,
    FlowingWater,
    RestingWater,
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Sand
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Point {
    x: usize,
    y: usize,
}

impl Point {
    fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

#[derive(Copy, Clone)]
struct Direction {
    x: i32,
    y: i32,
}

const DOWN: Direction = Direction { x: 0, y: 1 };
const RIGHT: Direction = Direction { x: 1, y: 0 };
const LEFT: Direction = Direction { x: -1, y: 0 };
const UP: Direction = Direction { x: 0, y: -1 };

#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
fn add_signed(a: usize, b: i32) -> usize {
    (a as i32 + b) as usize
}

impl std::ops::Add<Direction> for Point {
    type Output = Self;
    fn add(self, other: Direction) -> Self {
        Self {
            x: add_signed(self.x, other.x),
            y: add_signed(self.y, other.y),
        }
    }
}

impl std::ops::AddAssign<Direction> for Point {
    fn add_assign(&mut self, other: Direction) {
        self.x = add_signed(self.x, other.x);
        self.y = add_signed(self.y, other.y);
    }
}

pub struct Grid {
    cells: Array2<Cell>,
    /// Left-most clay block.
    min_x: usize,
    /// Top clay block.
    min_y: usize,
    /// Bottom clay block.
    max_y: usize,
}

impl std::ops::Index<Point> for Grid {
    type Output = Cell;
    fn index(&self, pt: Point) -> &Self::Output {
        &self.cells[(pt.x, pt.y)]
    }
}

impl std::ops::IndexMut<Point> for Grid {
    fn index_mut(&mut self, pt: Point) -> &mut Self::Output {
        &mut self.cells[(pt.x, pt.y)]
    }
}

impl std::fmt::Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.cells
                .axis_iter(ndarray::Axis(1))
                .map(|row| row
                    .iter()
                    .map(|c| match c {
                        Cell::Sand => ".",
                        Cell::Clay => "#",
                        Cell::FlowingWater => "|",
                        Cell::RestingWater => "~",
                    })
                    .collect::<std::string::String>())
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

const HORIZONTAL_MARGIN: usize = 2;
const VERTICAL_MARGIN: usize = 2;

/// Make a grid from a list of clay block coordinates.
/// The grid is slightly larger than needed to account for water flowing out on the edges, and at
/// the bottom.
pub fn make_grid(lines: &[Line]) -> Grid {
    let ((min_x, max_x), (min_y, max_y)) = find_bounds(lines);
    let mut grid = Array2::default((
        max_x - min_x + 2 * HORIZONTAL_MARGIN + 1,
        max_y + VERTICAL_MARGIN,
    ));
    for l in lines {
        for c in l.long_axis.0..=l.long_axis.1 {
            if l.is_vertical {
                grid[(l.short_axis - min_x + HORIZONTAL_MARGIN, c)] = Cell::Clay;
            } else {
                grid[(c - min_x + HORIZONTAL_MARGIN, l.short_axis)] = Cell::Clay;
            }
        }
    }
    Grid {
        cells: grid,
        min_x,
        min_y: min_y,
        max_y: max_y,
    }
}

/// Move the current_point down as far as it can go, until it reaches clay or resting water. Fill
/// the intermediate cells with flowing water.
fn flow_down(grid: &mut Grid, current_point: &mut Point) {
    assert!(grid[*current_point] != Cell::RestingWater);
    while current_point.y <= grid.max_y && can_flow_through(grid[*current_point + DOWN]) {
        grid[*current_point] = Cell::FlowingWater;
        *current_point += DOWN;
    }
    assert!(grid[*current_point] != Cell::RestingWater);
}

/// Returns whether water can flow through this cell, or whether it will spread.
fn can_flow_through(c: Cell) -> bool {
    use crate::Cell::*;
    match c {
        Sand | FlowingWater => true,
        Clay | RestingWater => false,
    }
}

/// Find the edge of spreading water (wall or cliff), in the direction `dir`. If the edge is a
/// cliff and the water will flow down, add it to the flow_points.
fn find_one_edge(
    grid: &Grid,
    mut current_point: Point,
    dir: Direction,
    flow_points: &mut Vec<Point>,
) -> usize {
    while grid[current_point + dir] != Cell::Clay && !can_flow_through(grid[current_point + DOWN]) {
        assert!(grid[current_point] != Cell::RestingWater);
        current_point += dir;
    }
    if can_flow_through(grid[current_point + DOWN]) {
        flow_points.push(current_point);
    }
    current_point.x
}

/// Find the x coordinate of the edges of the spreading water from `current_point`. Returns the x
/// coordinates, and the list of points where the water flows out.
fn find_edges(grid: &Grid, current_point: Point) -> ((usize, usize), Vec<Point>) {
    let mut flow_points = Vec::new();
    (
        (
            find_one_edge(grid, current_point, LEFT, &mut flow_points),
            find_one_edge(grid, current_point, RIGHT, &mut flow_points),
        ),
        flow_points,
    )
}

/// Fill the grid, given a source.
fn fill_grid_from(grid: &mut Grid, source: Point) {
    if grid[source] == Cell::RestingWater {
        return;
    }
    let mut current_point = source;
    flow_down(grid, &mut current_point);
    if current_point.y > grid.max_y {
        // We reached the bottom.
        return;
    }
    let mut flow_points = Vec::new();
    while flow_points.is_empty() {
        // We're filling a reservoir, keep filling until we reach the top.
        let ((x_left, x_right), fp) = find_edges(grid, current_point);
        flow_points = fp;
        if flow_points.is_empty() {
            // We found clay on both sides, we can't flow out.
            assert_eq!(grid[Point::new(x_left - 1, current_point.y)], Cell::Clay);
            assert_eq!(grid[Point::new(x_right + 1, current_point.y)], Cell::Clay);
        }
        // Fill the spreading water region with either resting water if we're surrounded by walls,
        // or flowing water if there is at least one cliff.
        let cell_content = if flow_points.is_empty() {
            Cell::RestingWater
        } else {
            Cell::FlowingWater
        };
        for x in x_left..=x_right {
            let cell = &mut grid.cells[(x, current_point.y)];
            assert!(*cell == Cell::FlowingWater || *cell == Cell::Sand);
            *cell = cell_content;
        }
        // Go back up until we're either out of the part we already filled, or we're above the
        // source.
        current_point += UP;
        while grid[current_point] == Cell::RestingWater {
            current_point += UP;
            if current_point.y <= source.y {
                return;
            }
        }
    }
    for p in flow_points {
        // Recurse for each flow point.
        fill_grid_from(grid, p + DOWN);
    }
}

/// Fill the grid, from the original source at (500, 0).
pub fn fill_grid(grid: &mut Grid) {
    fill_grid_from(grid, Point::new(500 + HORIZONTAL_MARGIN - grid.min_x, 0))
}

/// Count the amount of cells in the grid that match `filter`, between `min_y` and `max_y`.
fn count_cells(grid: &Grid, filter: &Fn(&&Cell) -> bool) -> usize {
    grid.cells
        .axis_iter(ndarray::Axis(1))
        .skip(grid.min_y)
        .take(grid.max_y - grid.min_y + 1)
        .flat_map(|row| row.iter().filter(filter).cloned().collect::<Vec<Cell>>())
        .count()
}

/// Count all flowing and resting water in the grid.
pub fn count_all_water(grid: &Grid) -> usize {
    count_cells(grid, &|&&c| {
        c == Cell::RestingWater || c == Cell::FlowingWater
    })
}

/// Count all resting water in the grid.
pub fn count_resting_water(grid: &Grid) -> usize {
    count_cells(grid, &|&&c| c == Cell::RestingWater)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_lines_test() {
        assert_eq!(
            parse_input(include_str!("../test_input")).expect("Failed to parse: "),
            vec![
                Line {
                    is_vertical: true,
                    short_axis: 495,
                    long_axis: (2, 7)
                },
                Line {
                    is_vertical: false,
                    short_axis: 7,
                    long_axis: (495, 501)
                },
                Line {
                    is_vertical: true,
                    short_axis: 501,
                    long_axis: (3, 7)
                },
                Line {
                    is_vertical: true,
                    short_axis: 498,
                    long_axis: (2, 4)
                },
                Line {
                    is_vertical: true,
                    short_axis: 506,
                    long_axis: (1, 2)
                },
                Line {
                    is_vertical: true,
                    short_axis: 498,
                    long_axis: (10, 13)
                },
                Line {
                    is_vertical: true,
                    short_axis: 504,
                    long_axis: (10, 13)
                },
                Line {
                    is_vertical: false,
                    short_axis: 13,
                    long_axis: (498, 504)
                }
            ]
        );
    }

    fn whole_test(input: &str) -> usize {
        let lines = parse_input(input).expect("Failed to parse: ");
        let mut grid = make_grid(&lines);
        fill_grid(&mut grid);
        let count = count_all_water(&grid);
        println!("bounds: {}..{}", grid.min_y, grid.max_y);
        println!("Grid:\n{}", grid);
        count
    }

    #[test]
    fn fill_grid_test() {
        assert_eq!(whole_test(include_str!("../test_input")), 57);
        assert_eq!(whole_test(include_str!("../test_input2")), 20);
    }
}
