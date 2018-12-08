#[macro_use]
extern crate nom;

use std::str::FromStr;
use std::string::String;
use nom::digit;
use nom::types::CompleteStr;

/// Parse an int.
named!(usize <&str, usize>,
   map!(map!(digit, FromStr::from_str), Result::unwrap));

/// Parse a list of int, separated by spaces.
named!(usize_list <&str, Vec<usize>>,
       separated_list!(char!(' '), usize));

/// Parse the tree specification as a list of ints.
pub fn parse_tree_specification(text: &str) -> Result<Vec<usize>, nom::Err<&str>> {
    usize_list(&CompleteStr(text)).map(|r| r.1)
}

/// Node in our generalized tree.
struct Node {
    children: Vec<Node>,
    metadata: Vec<usize>,
}

/// Root of the tree.
pub struct Tree(Node);

/// Parse a subtree by consuming elements from the input iterator, leaving it to point after the
/// definition of this node.
fn parse_tree_node<Iter>(spec: &mut Iter) -> Result<Node, String>
where
    Iter: Iterator<Item = usize>,
{
    // First element is the number of children.
    let num_children = spec.next().ok_or_else(
        || "Missing children number".to_owned(),
    )?;
    // Second element is the number of metadata.
    let num_metadata = spec.next().ok_or_else(
        || "Missing metadata number".to_owned(),
    )?;
    Ok(Node {
        // Recursively parse the children.
        children: (0..num_children)
            .map(|_| parse_tree_node(spec))
            .collect::<Result<Vec<_>, _>>()?,
        // Parse the metadata.
        metadata: (0..num_metadata)
            .map(|_| spec.next().ok_or("Missing metadata value"))
            .collect::<Result<Vec<_>, _>>()?,
    })
}

/// Parse a tree from the given specification.
pub fn parse_tree(spec: &[usize]) -> Result<Tree, String> {
    parse_tree_node(&mut spec.iter().cloned()).map(Tree)
}

/// Recursively sum the metadata of all the nodes.
fn count_metadata_rec(node: &Node) -> usize {
    node.metadata.iter().sum::<usize>() +
        node.children.iter().map(count_metadata_rec).sum::<usize>()
}

/// Get the sum of the metadata of all the nodes.
pub fn count_metadata(tree: &Tree) -> usize {
    count_metadata_rec(&tree.0)
}

/// Recursively compute the value of a node, as defined in the subject.
fn compute_node_value_rec(node: &Node) -> usize {
    if node.children.is_empty() {
        node.metadata.iter().sum()
    } else {
        node.metadata
            .iter()
            .flat_map(|&i| node.children.get(i - 1))
            .map(compute_node_value_rec)
            .sum()
    }
}

/// Compute the value of the root node.
pub fn compute_root_value(tree: &Tree) -> usize {
    compute_node_value_rec(&tree.0)
}
