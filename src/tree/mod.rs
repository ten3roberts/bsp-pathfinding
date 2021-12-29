use slotmap::*;

use crate::World;

use self::node::BSPNode;

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
    pub fn new(world: &World) -> Option<Self> {
        let faces: Vec<_> = world.shapes().flat_map(|shape| shape.faces()).collect();

        let mut nodes = SlotMap::with_key();
        let root = BSPNode::new(&mut nodes, &faces)?;

        Some(Self { nodes, root })
    }

    /// Draws the tree
    pub fn draw(&self, thickness: f32) {
        BSPNode::draw(self.root, &self.nodes, thickness, 0);
    }
}
