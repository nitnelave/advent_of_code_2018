use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use std::usize;

use crate::{is_unit, Board, Cell, Direction, Point, DIRECTIONS};
use ndarray::Array2;

#[derive(Copy, Clone, Eq, PartialEq)]
struct State {
    cost: usize,
    position: Point,
}

// The priority queue depends on `Ord`.
// Explicitly implement the trait so the queue becomes a min-heap
// instead of a max-heap.
impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        // Notice that the we flip the ordering on costs.
        // In case of a tie we compare positions - this step is necessary
        // to make implementations of `PartialEq` and `Ord` consistent.
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.position.cmp(&other.position))
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn into_coords(p: Point) -> (usize, usize) {
    (p.ux(), p.uy())
}

// Dijkstra's shortest path algorithm.

// Start at `start` and use `dist` to track the current shortest distance
// to each node. This implementation isn't memory-efficient as it may leave duplicate
// nodes in the queue. It also uses `usize::max_value()` as a sentinel value,
// for a simpler implementation.
pub fn shortest_path_step(board: &Board, start: Point, goal: &[Point]) -> Option<Point> {
    assert!(is_unit(board[start]));
    // dist[node] = current shortest distance from `start` to `node`
    let mut dist = Array2::from_elem(board.cells.dim(), usize::max_value());
    let mut prev: Array2<Option<Direction>> = Array2::from_elem(board.cells.dim(), None);

    let goals = goal.iter().collect::<HashSet<_>>();

    let mut heap = BinaryHeap::new();

    // We're at `start`, with a zero cost
    dist[into_coords(start)] = 0;
    heap.push(State {
        cost: 0,
        position: start,
    });

    let mut goals_reached = HashSet::new();
    let mut goal_distance = usize::max_value();

    // Examine the frontier with lower cost nodes first (min-heap)
    while let Some(State { cost, position }) = heap.pop() {
        assert!(board[position] != Cell::Wall);
        if cost > goal_distance {
            break;
        }
        // Important as we may have already found a better way
        if cost > dist[into_coords(position)] {
            continue;
        }
        // Continue until we have found all the goals tied for the closest distance.
        if goals.contains(&position) {
            goals_reached.insert(position);
            goal_distance = cost;
        }

        // For each node we can reach, see if we can find a way with
        // a lower cost going through this node
        for &d in &DIRECTIONS {
            let next = State {
                cost: cost + 1,
                position: position + d,
            };
            if board[next.position] != Cell::Empty {
                continue;
            }

            let next_coords = into_coords(next.position);
            // If so, add it to the frontier and continue
            if next.cost < dist[next_coords] {
                heap.push(next);
                // Relaxation, we have now found a better way
                dist[next_coords] = next.cost;
                if next.position != start {
                    prev[next_coords] = Some(-d);
                }
            }
        }
    }
    let mut sorted_goals = goals_reached.iter().collect::<Vec<_>>();
    if sorted_goals.is_empty() {
        return None;
    }
    sorted_goals.sort();
    let mut front = HashSet::new();
    front.insert(*sorted_goals[0]);
    let mut front_cost = dist[into_coords(*sorted_goals[0])];
    while front_cost > 1 {
        front_cost -= 1;
        let mut new_front = HashSet::new();
        for n in front {
            for &d in &DIRECTIONS {
                let position = n + d;
                if dist[into_coords(position)] == front_cost {
                    new_front.insert(position);
                }
            }
        }
        front = new_front;
    }

    for &d in &DIRECTIONS {
        if front.contains(&(start + d)) {
            return Some(start + d);
        }
    }
    panic!("No nearby node?");
}
