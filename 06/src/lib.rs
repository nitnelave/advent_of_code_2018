#[macro_use]
extern crate nom;
#[macro_use]
extern crate derive_more;

extern crate itertools;

use std::str::FromStr;
use std::string::String;
use nom::digit;
use nom::types::CompleteStr;
use itertools::{iterate, Itertools};
use itertools::MinMaxResult::MinMax;

#[derive(Eq, PartialEq, Debug, Copy, Clone, Add, Sub, Display, Ord, PartialOrd, From, Into, Mul)]
struct Distance(i32);

#[derive(Eq, PartialEq, Debug, Copy, Clone, Add, Sub, Display, Constructor, From, Into)]
#[display(fmt = "{}, {}", "x", "y")]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn dist(self, other: Self) -> Distance {
        Distance((self.x - other.x).abs() + (self.y - other.y).abs())
    }
}

/// The 4 cardinal directions, in a random order.
const DIRECTIONS: [Point; 4] = [
    Point { x: 0, y: 1 },
    Point { x: 0, y: -1 },
    Point { x: 1, y: 0 },
    Point { x: -1, y: 0 },
];

/// Parse an int.
named!(i32 <&str, i32>,
   map!(map!(digit, FromStr::from_str), Result::unwrap)
);

/// Parse a point.
named!(point <&str, Point>,
   do_parse!(
       p: separated_pair!(i32, tag!(", "), i32) >>
       (Point::from(p))
));


/// Convenience function to parse a point.
fn parse_point(input: &str) -> Point {
    point(&CompleteStr(input)).unwrap().1
}

/// Checks if the point at (`origin_x`, `origin_y`) is the closest one to each of the points on the
/// half-ray going in the positive x position, or if (`point_x`, `point_y`) is going to be closer to a
/// point (at some point).
fn is_point_blocking_direction(point_x: i32, origin_x: i32, point_y: i32, origin_y: i32) -> bool {
    if origin_x >= point_x {
        return false;
    }
    (point_x - origin_x) >= (point_y - origin_y).abs()
}

/// Checks if `origin` is the closest point to the half-ray in direction `direction` or if `point` is
/// going to be closer to some of the points.
fn is_point_blocking(point: Point, origin: Point, direction: Point) -> bool {
    if direction.x == 0 {
        if direction.y > 0 {
            is_point_blocking_direction(point.y, origin.y, point.x, origin.x)
        } else {
            is_point_blocking_direction(origin.y, point.y, origin.x, point.x)
        }
    } else if direction.x > 0 {
        is_point_blocking_direction(point.x, origin.x, point.y, origin.y)
    } else {
        is_point_blocking_direction(origin.x, point.x, origin.y, point.y)
    }
}

/// Checks if `origin` is the closest point to the half-ray in direction `direction` or if there is a
/// point that is going to be closer to some of the points.
fn is_infinite_direction(
    origin: Point,
    dist_from_origin: &[(&Point, Distance)],
    direction: Point,
) -> bool {
    !dist_from_origin.iter().any(|(&p, _)| {
        is_point_blocking(p, origin, direction)
    })
}

/// Check if `origin` has an unbounded area where it is the closest point, by checking the 4 axis.
/// This is enough, because the distance is the Manhattan distance.
fn is_infinite_point(origin: Point, dist_from_origin: &[(&Point, Distance)]) -> bool {
    DIRECTIONS.iter().any(|d| {
        is_infinite_direction(origin, &dist_from_origin, *d)
    })
}

/// Checks whether `origin` is the closest point to `point`.
fn is_closest_point(
    point: Point,
    origin: Point,
    dist_from_origin: &[(&Point, Distance)],
) -> bool {
    let mut point_distances = dist_from_origin
        .iter()
        .take_while(|(_, d)| *d <= point.dist(origin) * 2)
        .map(|(p, _)| p.dist(point))
        .collect::<Vec<_>>();
    point_distances.sort();
    point_distances[0] == origin.dist(point) &&
        (point_distances.len() == 1 || point_distances[1] > point_distances[0])
}

/// Find the distance to the furthest point along a half-ray that is still closer to `origin` than to
/// any other point.
fn find_furthest_point(
    origin: Point,
    dist_from_origin: &[(&Point, Distance)],
    direction: Point,
) -> Distance {
    iterate(origin, |&p| p + direction)
        .take_while(|p| is_closest_point(*p, origin, dist_from_origin))
        .last()
        .unwrap()
        .dist(origin)
}

/// Get the distance of each of the `points` to `origin`.
fn get_dist_from_origin(points: &[Point], origin: Point) -> Vec<(&Point, Distance)> {
    let mut dist_from_origin = points.iter().map(|p| (p, p.dist(origin))).collect::<Vec<_>>();
    dist_from_origin.sort_by_key(|(_, dist)| *dist);
    dist_from_origin
}

/// Find the area of the points that are closer to `origin` than to any other `points`. If the area is
/// unbounded, return None.
fn find_close_area(points: &[Point], origin: Point) -> Option<usize> {
    let dist_from_origin = get_dist_from_origin(points, origin);
    if is_infinite_point(origin, &dist_from_origin) {
        return None;
    }
    // Edges of the rectangle in which all the points are contained. These are distances in the
    // corresponding DIRECTIONS.
    let edges: Vec<_> = DIRECTIONS
        .iter()
        .map(|d| find_furthest_point(origin, &dist_from_origin, *d))
        .collect();
    Some(
        ((origin.x - i32::from(edges[3]))..=(origin.x + i32::from(edges[2])))
            .cartesian_product(
                (origin.y - i32::from(edges[1]))..=(origin.y + i32::from(edges[0])),
            )
            .map(|(x, y)| Point { x, y })
            .filter(|&p| is_closest_point(p, origin, &dist_from_origin))
            .count(),
    )
}

/// Find the point with the largest area in which it is the closest point from the list `lines`.
pub fn find_largest_close_area(lines: &[String]) -> usize {
    let points = lines.iter().map(|s| parse_point(s)).collect::<Vec<_>>();
    points
        .iter()
        .filter_map(|&p| find_close_area(&points, p))
        .max()
        .unwrap()
}

/// Check that the sum of the distances of all the pointst to `location` is under `max_distance`.
fn all_points_within(max_distance: Distance, location: Point, points: &[Point]) -> bool {
    points.iter().map(|&p| location.dist(p)).fold(
        Distance(0),
        std::ops::Add::add,
    ) <= max_distance
}

/// Get the min and the max from an iterator (assumed to have at least 2 different elements).
fn get_min_max<I>(iter: I) -> (i32, i32)
where
    I: Iterator<Item = i32>,
{
    match iter.minmax() {
        MinMax(min, max) => (min, max),
        _ => panic!("Not enough points!"),
    }
}

/// Find the area of the points for which the sum of the distances to each of the locations given
/// in `lines` are under `max_distance`.
pub fn find_area_close_to_points(lines: &[String], max_distance: i32) -> usize {
    let points = lines.iter().map(|s| parse_point(s)).collect::<Vec<_>>();
    let (min_x, max_x) = get_min_max(points.iter().map(|p| p.x));
    let (min_y, max_y) = get_min_max(points.iter().map(|p| p.y));
    (min_x..=max_x)
        .cartesian_product(min_y..=max_y)
        .map(Point::from)
        .filter(|&p| all_points_within(Distance(max_distance), p, &points))
        .count()
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_point() {
        assert_eq!(parse_point(&"1, 32".to_string()), Point { x: 1, y: 32 });
    }

    /// Test util to check the is_infinite_direction function.
    fn test_infinite_direction(origin: Point, other_points: Vec<Point>, expected_blocks: &[usize]) {
        let mut points = other_points.clone();
        points.push(origin);
        let dist_from_origin = get_dist_from_origin(&points, origin);
        for (i, d) in DIRECTIONS.iter().enumerate() {
            let expected = expected_blocks.iter().all(|&e| e != i);
            assert_eq!(
                is_infinite_direction(origin, &dist_from_origin, *d),
                expected,
                "Expected direction ({}) from ({}) with other points {:?} to be {}, got {}",
                d,
                origin,
                other_points,
                expected,
                !expected
            );
        }
    }

    #[test]
    fn test_is_infinite_direction() {
        test_infinite_direction(Point::new(5, 5), vec![Point::new(0, 1)], &[3]);
        test_infinite_direction(Point::new(5, 5), vec![Point::new(10, 0)], &[1, 2]);
        test_infinite_direction(Point::new(5, 5), vec![Point::new(1, 0)], &[1]);
        test_infinite_direction(
            Point::new(5, 5),
            vec![
                Point::new(1, 0),
                Point::new(0, 6),
                Point::new(10, 6),
                Point::new(6, 10),
            ],
            &[0, 1, 2, 3],
        );
    }
}
