use std::{
    collections::{BinaryHeap, HashSet},
    ops::Deref,
};

use glam::Vec2;
use slotmap::{secondary::Entry, SecondaryMap};

use crate::{BSPTree, NodeIndex, Portals};

#[derive(Debug, Clone)]
pub struct Path {
    points: Vec<Vec2>,
}

impl Path {
    /// Get a reference to the path's points.
    pub fn points(&self) -> &[Vec2] {
        self.points.as_ref()
    }
}

impl Deref for Path {
    type Target = [Vec2];

    fn deref(&self) -> &Self::Target {
        self.points.deref()
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Backtrace {
    node: NodeIndex,
    // The first side of the portal
    left: Vec2,
    // The other edge of the portal
    right: Vec2,
    prev: Option<NodeIndex>,
    start_cost: f32,
    total_cost: f32,
}

impl Backtrace {
    pub fn from_backtrace(
        node: NodeIndex,
        left: Vec2,
        right: Vec2,
        backtrace: &Backtrace,
        heuristic: f32,
    ) -> Self {
        let start_cost = backtrace.start_cost + left.distance(backtrace.left);
        Self {
            node,
            left,
            right,
            prev: Some(backtrace.node),
            start_cost,
            total_cost: start_cost + heuristic,
        }
    }

    pub fn new(
        node: NodeIndex,
        left: Vec2,
        right: Vec2,
        prev: Option<NodeIndex>,
        start_cost: f32,
        heuristic: f32,
    ) -> Self {
        Self {
            node,
            left,
            right,
            prev,
            start_cost,
            total_cost: start_cost + heuristic,
        }
    }
}

// Order by lowest total_cost
impl PartialOrd for Backtrace {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.total_cost.partial_cmp(&self.total_cost)
    }
}

impl Eq for Backtrace {}

impl Ord for Backtrace {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .total_cost
            .partial_cmp(&self.total_cost)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

pub fn astar<F: Fn(Vec2, Vec2) -> f32>(
    tree: &BSPTree,
    portals: &Portals,
    start: Vec2,
    end: Vec2,
    heuristic: F,
) -> Option<Path> {
    let mut open = BinaryHeap::new();
    let start_node = tree.locate(start);
    let end_node = tree.locate(end);

    // No path if start or end a covered
    if start_node.covered() || end_node.covered() {
        return None;
    }

    let start_node = start_node.index();
    let end_node = end_node.index();

    // Information of how a node was reached
    let mut backtraces: SecondaryMap<_, Backtrace> = SecondaryMap::new();
    let start = Backtrace::new(start_node, start, start, None, 0.0, (heuristic)(start, end));

    // Push the fist node
    open.push(start);
    backtraces.insert(start_node, start);

    let mut closed = HashSet::new();

    // Expand the node with the lowest total cost
    while let Some(current) = open.pop() {
        // Skip edge if dst has already been visited
        if closed.contains(&current.node) {
            continue;
        }

        // End found
        // Generate backtrace and terminate
        if current.node == end_node {
            return Some(backtrace(end, current.node, backtraces));
        }

        // Add all edges to the open list and update backtraces
        let portals = portals
            .get(current.node)
            .iter()
            .map(|edge| {
                assert_eq!(edge.src(), current.node);

                // Calculate cost based on current position
                // Try both ends of the portal
                [
                    Backtrace::from_backtrace(
                        edge.dst(),
                        edge.vertices[0],
                        edge.vertices[1],
                        &current,
                        (heuristic)(edge.vertices[0], end),
                    ),
                    Backtrace::from_backtrace(
                        edge.dst(),
                        edge.vertices[1],
                        edge.vertices[0],
                        &current,
                        (heuristic)(edge.vertices[1], end),
                    ),
                ]
            })
            .flatten()
            .map(|backtrace| {
                // Update backtrace
                // If the cost to this node is lower than previosuly found,
                // overwrite with the new backtrace.
                match backtraces.entry(backtrace.node).unwrap() {
                    Entry::Occupied(mut val) => {
                        if val.get().total_cost > backtrace.total_cost {
                            val.insert(backtrace);
                        }
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(backtrace);
                    }
                }

                backtrace
            });

        // Add the edges
        open.extend(portals);

        // The current node is now done and won't be revisited
        assert!(closed.insert(current.node))
    }

    None
}

fn backtrace(
    end: Vec2,
    mut current: NodeIndex,
    backtraces: SecondaryMap<NodeIndex, Backtrace>,
) -> Path {
    let mut backtrace = vec![end];
    loop {
        let node = backtraces[current];
        backtrace.push(node.left);

        // Continue up the backtrace
        if let Some(prev) = node.prev {
            current = prev;
        } else {
            break;
        }
    }

    backtrace.reverse();
    eprintln!("Backtraced");
    Path { points: backtrace }
}
