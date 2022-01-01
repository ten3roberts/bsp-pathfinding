use glam::Vec2;
use slotmap::*;

use crate::Face;

pub use edges::*;
pub use node::*;

mod edges;
mod node;

type Nodes = SlotMap<NodeIndex, BSPNode>;

new_key_type! {
    pub struct NodeIndex;
}
/// Defines the tree used for navigation
pub struct BSPTree {
    nodes: Nodes,
    root: NodeIndex,
}

impl BSPTree {
    /// Constructs a new tree.
    /// Returns None if there are not faces, and root construction was not possible
    pub fn new(faces: impl Iterator<Item = Face>) -> Option<Self> {
        let faces: Vec<_> = faces.collect();

        let mut nodes = SlotMap::with_key();
        let root = BSPNode::new(&mut nodes, &faces, None, 0)?;

        Some(Self { nodes, root })
    }

    /// Constructs a new tree.
    /// Returns None if there are not faces, and root construction was not possible
    pub fn with_bounds(faces: impl Iterator<Item = Face>, bounds: &[Face]) -> Option<Self> {
        let faces: Vec<_> = faces.collect();

        let mut nodes = SlotMap::with_key();
        let root = BSPNode::new(&mut nodes, &faces, Some(bounds.to_vec()), 0)?;

        Some(Self { nodes, root })
    }

    pub fn node(&self, index: NodeIndex) -> Option<&BSPNode> {
        self.nodes.get(index)
    }

    /// Returns the root index
    pub fn root(&self) -> NodeIndex {
        self.root
    }

    /// Returns a reference to the root node
    pub fn root_node(&self) -> &BSPNode {
        self.node(self.root).expect("Root is always present")
    }

    pub fn descendants(&self) -> DescendantsIter {
        BSPNode::descendants(self.root, &self.nodes)
    }

    pub fn generate_edges(&self) -> Edges {
        Edges::new(self.root, &self.nodes)
    }

    /// Returns the containing node and if the point is covered
    pub fn containing_node(&self, point: Vec2) -> NodePayload {
        let mut index = self.root;

        loop {
            let node = &self.nodes[index];
            let rel = point - node.origin();

            let (next, covered) = if rel.dot(node.normal()) > 0.0 {
                (node.front(), false)
            } else {
                (node.back(), true)
            };

            if let Some(next) = next {
                index = next
            } else {
                return NodePayload {
                    index,
                    node,
                    covered,
                };
            }
        }
    }
}

pub struct NodePayload<'a> {
    index: NodeIndex,
    node: &'a BSPNode,
    covered: bool,
}

impl<'a> NodePayload<'a> {
    /// Get the node payload's node.
    pub fn node(&self) -> &BSPNode {
        self.node
    }

    /// Get the node payload's index.
    pub fn index(&self) -> NodeIndex {
        self.index
    }

    /// Get the node payload's covered.
    pub fn covered(&self) -> bool {
        self.covered
    }
}
