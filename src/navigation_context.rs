use crate::{
    astar::{astar, Path, SearchInfo},
    BSPNode, BSPTree, NodeIndex, NodePayload, PortalIter,
};
use glam::Vec2;
use itertools::Itertools;
use rand::Rng;

use crate::{Face, Portals};

/// Contains the graph and edges necessary for path finding
#[derive(Default)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct NavigationContext {
    tree: Option<BSPTree>,
    portals: Portals,
}

impl NavigationContext {
    /// Creates a new navigation context
    pub fn new(faces: impl IntoIterator<Item = Face>) -> Self {
        let tree = BSPTree::new(faces.into_iter().collect_vec());
        let mut portals = Portals::new();
        if let Some(tree) = tree.as_ref() {
            portals.generate(&tree);
        }

        Self { tree, portals }
    }

    /// Creates a new navigation context.
    /// Shuffles the input which usually reduces the depth of the final tree.
    pub fn new_shuffle(faces: impl IntoIterator<Item = Face>, rng: &mut impl Rng) -> Self {
        let tree = BSPTree::new_shuffle(faces.into_iter(), rng);
        let mut portals = Portals::new();
        if let Some(tree) = tree.as_ref() {
            portals.generate(&tree);
        }

        Self { tree, portals }
    }
    pub fn node(&self, index: NodeIndex) -> Option<&BSPNode> {
        self.tree.as_ref()?.node(index)
    }

    /// Locate a position in the tree.
    /// Return None if there are no faces in the scene
    pub fn locate(&self, point: Vec2) -> Option<NodePayload> {
        match &self.tree {
            Some(tree) => Some(tree.locate(point)),
            None => None,
        }
    }

    /// Get a reference to the navigation context's tree.
    pub fn tree(&self) -> Option<&BSPTree> {
        self.tree.as_ref()
    }

    /// Get a reference to the navigation context's portals.
    pub fn portals(&self) -> &Portals {
        &self.portals
    }

    /// Get the portals associated to a node
    pub fn get(&self, index: NodeIndex) -> PortalIter {
        self.portals.get(index)
    }

    /// Find a path from `start` to `end`
    /// Returns None if no path was found.
    /// If there are no faces in the scene, a straight path will be returned.
    pub fn find_path(
        &self,
        start: Vec2,
        end: Vec2,
        heuristic: impl Fn(Vec2, Vec2) -> f32,
        info: SearchInfo,
    ) -> Option<Path> {
        let mut path = None;
        match &self.tree {
            Some(tree) => {
                astar(&tree, &self.portals, start, end, heuristic, info, &mut path);
                path
            }
            None => Some(Path::euclidian(start, end)),
        }
    }

    /// Find a path from `start` to `end`
    /// Returns None if no path was found.
    /// If there are no faces in the scene, a straight path will be returned.
    /// Uses an already allocated path to fill and will attempt to only update
    /// parts of the path
    pub fn find_path_inc<'a>(
        &self,
        start: Vec2,
        end: Vec2,
        heuristic: impl Fn(Vec2, Vec2) -> f32,
        info: SearchInfo,
        path: &'a mut Option<Path>,
    ) -> Option<&'a mut Path> {
        match &self.tree {
            Some(tree) => astar(&tree, &self.portals, start, end, heuristic, info, path),
            None => {
                *path = Some(Path::euclidian(start, end));
                path.as_mut()
            }
        }
    }
}
