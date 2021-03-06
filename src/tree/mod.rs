use std::ops::Index;

use glam::Vec2;
use rand::{prelude::SliceRandom, Rng};
use slotmap::*;

use crate::Face;

pub use node::*;
pub use portal::*;
pub use portals::*;

mod node;
mod portal;
mod portals;

type Nodes = SlotMap<NodeIndex, BSPNode>;

new_key_type! {
    /// Represent the index of a [crate::BSPNode]
    pub struct NodeIndex;
}
/// Defines the tree used for navigation
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct BSPTree {
    nodes: Nodes,
    root: NodeIndex,
    // Bounds
    l: Vec2,
    r: Vec2,
}

impl BSPTree {
    /// Constructs a new tree.
    /// Returns None if there are not faces, and root construction was not possible
    pub fn new(faces: Vec<Face>) -> Option<Self> {
        Self::new_inner(faces)
    }

    pub fn new_shuffle(faces: impl Iterator<Item = Face>, rng: &mut impl Rng) -> Option<Self> {
        let mut faces: Vec<_> = faces.collect();
        faces.shuffle(rng);

        Self::new_inner(faces)
    }

    fn new_inner(faces: Vec<Face>) -> Option<Self> {
        let mut l = Vec2::new(f32::MAX, f32::MAX);
        let mut r = Vec2::new(f32::MIN, f32::MIN);

        faces.iter().flatten().for_each(|val| {
            l = l.min(val);
            r = r.max(val);
        });

        let mut nodes = SlotMap::with_key();
        let root = BSPNode::from_faces(&mut nodes, &faces, 0)?;

        Some(Self { nodes, root, l, r })
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

    pub fn descendants(&self) -> Descendants {
        BSPNode::descendants(self.root, &self.nodes)
    }

    /// Returns the containing node and if the point is covered
    pub fn locate(&self, point: Vec2) -> NodePayload {
        let mut index = self.root;

        loop {
            let node = &self.nodes[index];
            let rel = point - node.origin();
            let dot = rel.dot(node.normal());

            let (next, covered) = if dot >= 0.0 {
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
                    depth: if covered {
                        -node.normal() * dot
                    } else {
                        Vec2::ZERO
                    },
                };
            }
        }
    }

    /// Get a mutable reference to the bsptree's root.
    pub fn root_mut(&mut self) -> &mut NodeIndex {
        &mut self.root
    }

    /// Get a reference to the bsptree's nodes.
    pub fn nodes(&self) -> &Nodes {
        &self.nodes
    }

    /// Returns clipping planes which contain the scene
    pub fn clipping_planes(&self) -> [Face; 4] {
        [
            Face::new([Vec2::new(self.l.x, self.r.y), self.l]),
            Face::new([self.l, Vec2::new(self.r.x, self.l.y)]),
            Face::new([Vec2::new(self.r.x, self.l.y), self.r]),
            Face::new([self.r, Vec2::new(self.l.x, self.r.y)]),
        ]
    }

    pub fn generate_portals(&self) -> Vec<ClippedFace> {
        let clipping_planes = self.clipping_planes().into_iter().collect();

        let mut portals = Vec::new();
        BSPNode::generate_portals(self.root, &self.nodes, &clipping_planes, &mut portals);
        portals
    }
}

/// Represents the result of [crate::BSPTree::locate]
#[derive(Clone, Debug)]
pub struct NodePayload<'a> {
    pub index: NodeIndex,
    pub node: &'a BSPNode,
    pub covered: bool,
    pub depth: Vec2,
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

    /// Get the node payload's depth.
    pub fn depth(&self) -> Vec2 {
        self.depth
    }
}

impl Index<NodeIndex> for BSPTree {
    type Output = BSPNode;

    fn index(&self, index: NodeIndex) -> &Self::Output {
        self.node(index).unwrap()
    }
}
