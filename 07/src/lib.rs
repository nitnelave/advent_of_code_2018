#![feature(existential_type)]
#[macro_use]
extern crate nom;
#[macro_use]
extern crate derive_more;

extern crate boolinator;

use std::iter::Iterator;
use std::fmt::Debug;
use std::collections::BinaryHeap;
use boolinator::Boolinator;

const ALPHABET_SIZE: usize = 26;

#[derive(Eq, PartialEq, Debug, Into, Copy, Clone)]
pub struct NodeId(u8);

/// Reverse order: max returns the smallest element.
impl std::cmp::PartialOrd for NodeId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.0.partial_cmp(&self.0)
    }
}
/// Reverse order: max returns the smallest element.
impl std::cmp::Ord for NodeId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.0.cmp(&self.0)
    }
}

/// Parse a NodeId: Map 'A' to 0, 'B' to 1, etc...
named!(node_id <&str, NodeId>,
       map!(take!(1), |c| NodeId(c.bytes().next().unwrap() - b'A'))
);

/// A constraint is an edge in our directed graph: the first one must come before the second one.
type Constraint = (NodeId, NodeId);

/// Parse a constraint from a sentence.
/// ```
/// assert_eq!(constraint("Step C must be finished before step A can begin.").unwrap().1,
///            (NodeId(2), NodeId(0)));
/// ```
named!(constraint <&str, Constraint>,
    do_parse!(
        tag!("Step ") >>
        from: node_id >>
        tag!(" must be finished before step ") >>
        to: node_id >>
        tag!(" can begin.") >>
        (from, to)
));

/// Parse a list of constraints from a list of strings.
fn parse_constraints(lines: &[String]) -> Vec<Constraint> {
    lines.iter().map(|l| constraint(l).unwrap().1).collect()
}

/// A node in our graph.
#[derive(Debug, Clone)]
struct Node {
    /// Links to the children nodes.
    children: Vec<NodeId>,
    /// Number of parent nodes.
    num_parents: usize,
}

/// A simple graph is just the structure, with each node identified by a `NodeId`.
///
/// Since the `NodeId`s may not be dense, we only store Options in the vector.
#[derive(Debug)]
struct SimpleGraph {
    nodes: Vec<Option<Node>>,
}

impl SimpleGraph {
    /// Mutable access to a node. If the node doesn't exist, it will be created.
    fn get_mut_node(&mut self, key: NodeId) -> &mut Node {
        let cell = &mut self.nodes[key.0 as usize];
        if cell.is_none() {
            *cell = Some(Node {
                children: vec![],
                num_parents: 0,
            });
        }
        self.nodes[key.0 as usize].as_mut().unwrap()
    }
    /// Read-only access to a node. If the node doesn't exist, panic.
    fn get_node(&self, key: NodeId) -> &Node {
        self.nodes[key.0 as usize].as_ref().unwrap()
    }

    /// Iterate over all the (defined) nodes in the graph.
    fn iter(&self) -> impl Iterator<Item = &Node> {
        self.nodes.iter().flatten()
    }
}

/// Build a graph from a list of constraints.
fn build_graph(constraints: &[Constraint]) -> SimpleGraph {
    let mut graph = SimpleGraph { nodes: vec![None; ALPHABET_SIZE] };
    for c in constraints {
        graph.get_mut_node(c.0).children.push(c.1);
        graph.get_mut_node(c.1).num_parents += 1;
    }
    graph
}

/// A graph with other data attached to each node. It is a wrapper over a `SimpleGraph`.
#[derive(Debug)]
struct AnnotatedGraph<T>
where
    T: Default + Copy + Debug,
{
    graph: SimpleGraph,
    annotations: Vec<T>,
}

impl<T> AnnotatedGraph<T>
where
    T: Default + Copy + Debug,
{
    fn new(graph: SimpleGraph) -> Self {
        let num_nodes = graph.nodes.len();
        Self {
            graph,
            annotations: (0..num_nodes).map(|_| T::default()).collect(),
        }
    }
    /// Get the node of the original graph, with the mutable associated metadata.W
    /// The original node is not mutable.
    fn get_mut_node(&mut self, key: NodeId) -> (&Node, &mut T) {
        (
            &self.graph.get_node(key),
            &mut self.annotations[key.0 as usize],
        )
    }
    /// Get the node of the original graph, with the associated metadata.
    fn get_node(&self, key: NodeId) -> (&Node, &T) {
        (&self.graph.get_node(key), &self.annotations[key.0 as usize])
    }
}

/// Find all the roots (no parent) of a simple graph.
#[allow(clippy::cast_possible_truncation)]
fn find_roots(graph: &SimpleGraph) -> Vec<NodeId> {
    graph
        .iter()
        .enumerate()
        .filter_map(|(i, n)| (n.num_parents == 0).as_some(NodeId(i as u8)))
        .collect()
}

/// Update all the children of the node by increasing the number of `done_parents`.
/// If all the parents are done, push the node on the heap.
fn mark_node_as_finished(
    node: NodeId,
    graph: &mut AnnotatedGraph<usize>,
    node_heap: &mut BinaryHeap<NodeId>,
) {
    // Update the children of the node, incrementing the number of parents done.
    let children: Vec<NodeId> = graph.get_node(node).0.children.clone();
    for n in children {
        let num_parents = graph.get_node(n).0.num_parents;
        let done_parents = graph.get_mut_node(n).1;
        *done_parents += 1;
        // If all their parents are done, they can be added to the nodes without dependencies.
        if num_parents == *done_parents {
            node_heap.push(n);
        }
    }
}

/// From an annotated graph, find the build order: for each step, find the node with no dependency
/// with the lowest value (i.e. 'A' before 'B').
fn find_build_order_from_graph(mut graph: AnnotatedGraph<usize>) -> Vec<NodeId> {
    // The BinaryHeap returns the max. Since Ord for NodeId returns the opposite order, this is a
    // min heap, initialized with all the nodes without dependencies.
    let mut node_heap = BinaryHeap::from(find_roots(&graph.graph));
    // The list of steps, in order.
    let mut result = Vec::new();
    while let Some(node) = node_heap.pop() {
        result.push(node);
        mark_node_as_finished(node, &mut graph, &mut node_heap);
    }
    result
}

/// Returns the time it takes to complete a step. It is defined as 60 + the position of the letter,
/// so 61 for 'A', 62 for 'B', etc.
fn get_node_time(node: NodeId) -> usize {
    node.0 as usize + 61
}

/// Given that we have several workers that can process steps in parallel, this returns the total
/// time to process the graph.
fn find_build_time_from_graph_with_workers(
    mut graph: AnnotatedGraph<usize>,
    num_workers: usize,
) -> usize {
    // Nodes without dependencies.
    let mut node_heap = BinaryHeap::from(find_roots(&graph.graph));
    // Total time taken so far.
    let mut total_time = 0;

    // A worker is either None (idle) or contains the node that it is currently processing, and the
    // time (from the start of the build) at which it will be done.
    // Initialize the workers with the top n steps from the heap (as much as possible).
    let mut workers: Vec<Option<(usize, NodeId)>> = (0..num_workers)
        .map(|_| node_heap.pop().map(|n| (get_node_time(n), n)))
        .collect();

    // While the workers are still doing something, find the next step completion time.
    while let Some((time, _)) = workers.iter().flatten().min_by_key(|(t, _)| t) {
        // Completion time of the steps so far.
        total_time = *time;
        // Update all nodes that finished at the current step.
        workers
            .iter()
            .flatten()
            .filter_map(|(t, n)| if *t == total_time { Some(n) } else { None })
            .for_each(|node| {
                mark_node_as_finished(*node, &mut graph, &mut node_heap)
            });
        workers = workers
            .iter()
            .map(|o| if o.is_some() && o.unwrap().0 != total_time {
                *o
            } else {
                node_heap.pop().map(|n| (total_time + get_node_time(n), n))
            })
            .collect();
    }
    total_time
}

pub fn find_build_order(lines: &[String]) -> Vec<NodeId> {
    let graph = build_graph(&parse_constraints(lines));
    let annotated_graph = AnnotatedGraph::new(graph);
    find_build_order_from_graph(annotated_graph)
}

pub fn find_build_time_with_workers(lines: &[String], num_workers: usize) -> usize {
    let graph = build_graph(&parse_constraints(lines));
    let annotated_graph = AnnotatedGraph::new(graph);
    find_build_time_from_graph_with_workers(annotated_graph, num_workers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id() {
        assert_eq!(node_id("A").unwrap().1, NodeId(0));
        assert_eq!(node_id("B").unwrap().1, NodeId(1));
        assert_eq!(node_id("Z").unwrap().1, NodeId(25));
    }
    #[test]
    fn test_constraint() {
        assert_eq!(constraint("Step F must be finished before step E can begin.").unwrap().1,
                   (NodeId(5), NodeId(4)));
    }
}
