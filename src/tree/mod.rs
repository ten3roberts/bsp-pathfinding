use slotmap::*;

use crate::Face;

pub use node::{BSPNode, DescendantsIter};

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
        let root = BSPNode::new(&mut nodes, &faces)?;

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
}
