#![allow(clippy::cast_possible_truncation)]

#[macro_use]
extern crate nom;
extern crate itertools;

use nom::digit;
use itertools::Itertools;

#[derive(Copy, Clone)]
struct Point {
    x: i64,
    y: i64,
}

pub struct Coordinates {
    pos: Point,
    speed: Point,
}

/// Parse an int.
named!(i64 <&str, i64>,
       map!(
           ws!(pair!(
                   opt!(char!('-')),
                   digit)),
           |(s, d)| d.parse::<i64>().unwrap() *
                    if s.is_some() { -1} else {1}
           )
);

named!(
    coordinate <&str, Coordinates>,
    do_parse!(
        tag_s!("position=<") >> pos_x: i64 >> char!(',') >> pos_y: i64 >>
            tag_s!("> velocity=<") >> vel_x: i64 >> char!(',') >> vel_y: i64 >>
            char!('>') >>
            (Coordinates {
                 pos: Point { x: pos_x, y: pos_y },
                 speed: Point { x: vel_x, y: vel_y },
             })
    )
);

pub fn parse_lines(lines: &[String]) -> Vec<Coordinates> {
    lines.iter().map(|l| coordinate(&l).unwrap().1).collect()
}

fn get_intersection_axis(c1: &Coordinates, c2: &Coordinates, axis: &Fn(Point) -> f64) -> f64 {
    (axis(c1.pos) - axis(c2.pos)) / (axis(c2.speed) - axis(c1.speed))
}

fn find_intersection(c: (&Coordinates, &Coordinates)) -> Option<i64> {
    let intersection_x = get_intersection_axis(c.0, c.1, &|p| f64::from(p.x as i32));
    let intersection_y = get_intersection_axis(c.0, c.1, &|p| f64::from(p.y as i32));
    if intersection_x.is_normal() {
        if intersection_y.is_normal() {
            Some((intersection_x + intersection_y) / 2.0)
        } else {
            Some(intersection_x)
        }
    } else if intersection_y.is_normal() {
            Some(intersection_y)
    } else {
        None
    }.map(|f| f.round() as i64)
}

fn find_median_intersection(coordinates: &[Coordinates]) -> i64 {
    let (first_half, second_half) = coordinates.split_at(coordinates.len() / 2);
    let mut intersections = first_half
        .iter()
        .zip(second_half)
        .flat_map(find_intersection)
        .collect::<Vec<_>>();
    intersections.sort();
    intersections[intersections.len() / 2]
}

fn move_point(c: &Coordinates, time: i64) -> Option<Point>{
    Some(Point{
        x: c.pos.x.checked_add(time.checked_mul(c.speed.x)?)?,
        y: c.pos.y.checked_add(time.checked_mul(c.speed.y)?)?,
    })
}

fn print_message_at(coordinates: &[Coordinates], time: i64) {
    assert!(time > 0);
    println!("time: {}", time);
    let points = coordinates.iter().flat_map(|c| move_point(c, time));
    if let itertools::MinMaxResult::MinMax(min_x, max_x) = points.clone().map(|p| p.x).minmax() {
        if let itertools::MinMaxResult::MinMax(min_y, max_y) = points.clone().map(|p| p.y).minmax() {
            let width = max_x - min_x + 1;
            let mut board = (min_x..=max_x)
                .cartesian_product(min_y..=max_y)
                .map(|_| false)
                .collect::<Vec<_>>();
            // We're not looking in the past, the intersection is guaranteed to be in the future.
            #[allow(clippy::cast_sign_loss)]
            let x_y_to_index = |x, y| (x - min_x + width * (y - min_y)) as usize;
            points.for_each(|p| board[x_y_to_index(p.x, p.y)] = true);
            (min_y..=max_y).for_each(|y|
                {
                    (min_x..=max_x).for_each(|x|
                        print!("{}",
                               if board[x_y_to_index(x, y)] {
                                   "#"
                               } else {
                                   " "
                               }));
                    println!()
                }
            );
    }}
}

pub fn print_message(coordinates: &[Coordinates]) {
    let t = find_median_intersection(coordinates);
    print_message_at(coordinates, t);
}
