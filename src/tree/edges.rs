use glam::Vec2;
use slotmap::{secondary::Iter, SecondaryMap};
use smallvec::SmallVec;

use crate::{BSPNode, NodeIndex, TOLERANCE};

use super::Nodes;

pub(crate) type NodeEdges = SmallVec<[Edge; 4]>;

pub struct Edges {
    inner: SecondaryMap<NodeIndex, NodeEdges>,
}

impl Edges {
    pub fn new(root: NodeIndex, nodes: &Nodes) -> Self {
        let inner = SecondaryMap::with_capacity(nodes.len());

        let mut edges = Self { inner };

        for (index, _) in BSPNode::descendants(root, nodes) {
            edges.construct_edges(index, nodes);
        }

        edges
    }

    fn construct_edges(&mut self, index: NodeIndex, nodes: &Nodes) {
        // Construct an edge between the node and transitive child

        let node = &nodes[index];

        if let Some(val) = node.front() {
            self.construct_for_node(index, node, val, nodes);
        }

        if let Some(val) = node.back() {
            self.construct_for_node(index, node, val, nodes);
        }
    }

    fn construct_for_node(
        &mut self,
        index: NodeIndex,
        node: &BSPNode,
        child_index: NodeIndex,
        nodes: &Nodes,
    ) {
        let child = &nodes[child_index];

        // Find shared vertex
        for a in node.vertices() {
            for b in child.vertices() {
                if (*a - *b).length_squared() > TOLERANCE * TOLERANCE {
                    continue;
                }

                let from = BSPNode::transitive_node(index, *a, nodes);
                let to = BSPNode::transitive_node(child_index, *a, nodes);

                self.add(
                    from,
                    to,
                    Edge::new(*a, [nodes[from].origin(), nodes[to].origin()]),
                );
            }
        }

        // self.construct_edges(child_index, nodes);

        match (child.front(), child.back()) {
            (None, None) => {}
            (None, Some(val)) => self.construct_for_node(index, node, val, nodes),
            (Some(val), None) => self.construct_for_node(index, node, val, nodes),
            (Some(a), Some(b)) => {
                let a_rel = nodes[a].origin() - node.origin();
                let b_rel = nodes[b].origin() - node.origin();

                if a_rel.dot(node.normal()) > b_rel.dot(node.normal()) {
                    self.construct_for_node(index, node, a, nodes)
                } else {
                    self.construct_for_node(index, node, b, nodes)
                }
            }
        }
    }

    /// Adds a two way edge
    pub fn add(&mut self, a: NodeIndex, b: NodeIndex, edge: Edge) {
        self.inner
            .entry(a)
            .expect("Node was removed")
            .or_default()
            .push(edge);
        self.inner
            .entry(b)
            .expect("Node was removed")
            .or_default()
            .push(edge);
    }

    /// Iterate all edges
    pub fn iter(&self) -> AllEdgesIter {
        AllEdgesIter {
            inner: self.inner.iter(),
        }
    }

    /// Get the edges for a specific node
    pub fn get(&self, index: NodeIndex) -> Option<&[Edge]> {
        self.inner.get(index).map(|val| val.as_ref())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Edge {
    pos: Vec2,
    origins: [Vec2; 2],
}

impl Edge {
    pub fn new(pos: Vec2, origins: [Vec2; 2]) -> Self {
        Self { pos, origins }
    }

    /// Get the edge's origins.
    /// Used for visualization purposes
    pub fn origins(&self) -> [Vec2; 2] {
        self.origins
    }

    /// Get the edge's pos.
    pub fn pos(&self) -> Vec2 {
        self.pos
    }
}

pub struct AllEdgesIter<'a> {
    inner: Iter<'a, NodeIndex, NodeEdges>,
}

impl<'a> Iterator for AllEdgesIter<'a> {
    type Item = &'a [Edge];

    fn next(&mut self) -> Option<Self::Item> {
        let (_, edges) = self.inner.next()?;
        Some(edges.as_ref())
    }
}
