use std::ops::Deref;

use glam::Vec2;
use slotmap::{secondary::Iter, SecondaryMap};
use smallvec::SmallVec;

use crate::{util::face_intersect, BSPTree, Face, NodeIndex, Side, TOLERANCE};

#[derive(Copy, Debug, Clone, PartialEq)]
pub struct Portal {
    face: Face,
    // Used to determine if a face is completely inside
    pub(crate) sides: [Side; 2],

    pub src: NodeIndex,
    pub dst: NodeIndex,
}

impl Portal {
    pub fn new(vertices: [Vec2; 2], sides: [Side; 2], src: NodeIndex, dst: NodeIndex) -> Self {
        Self {
            face: Face::new(vertices),
            sides,
            src,
            dst,
        }
    }

    /// Split the face. The first face is in front
    pub fn split(&self, p: Vec2, normal: Vec2) -> [Self; 2] {
        let intersection = face_intersect(self.into_tuple(), p, normal);
        let a = (self.vertices[0] - p).dot(normal);

        // a is in front
        if a >= -TOLERANCE {
            [
                Self::new(
                    [self.vertices[0], intersection.point],
                    [self.sides[0], Side::Front],
                    self.src,
                    self.dst,
                ),
                Self::new(
                    [intersection.point, self.vertices[1]],
                    [Side::Back, self.sides[1]],
                    self.src,
                    self.dst,
                ),
            ]
        } else {
            // a is behind
            [
                Self::new(
                    [self.vertices[1], intersection.point],
                    [self.sides[1], Side::Front],
                    self.src,
                    self.dst,
                ),
                Self::new(
                    [self.vertices[0], intersection.point],
                    [self.sides[0], Side::Back],
                    self.src,
                    self.dst,
                ),
            ]
        }
    }

    /// Get the portal's src.
    pub fn src(&self) -> NodeIndex {
        self.src
    }

    /// Get the portal's dst.
    pub fn dst(&self) -> NodeIndex {
        self.dst
    }

    /// Get the portal's sides.
    pub fn sides(&self) -> [Side; 2] {
        self.sides
    }

    pub fn reverse(&self) -> Self {
        Self {
            face: self.face,
            sides: self.sides,
            src: self.dst,
            dst: self.src,
        }
    }
}

impl Deref for Portal {
    type Target = Face;

    fn deref(&self) -> &Self::Target {
        &self.face
    }
}

pub type NodePortals = SmallVec<[Portal; 4]>;

pub struct Portals {
    inner: SecondaryMap<NodeIndex, NodePortals>,
}

impl Portals {
    pub fn new() -> Self {
        Self {
            inner: SecondaryMap::new(),
        }
    }

    pub fn generate(&mut self, tree: &BSPTree) {
        for portal in tree.generate_portals() {
            self.add(portal)
        }
    }

    pub fn from_tree(tree: &BSPTree) -> Self {
        let mut portals = Self::new();
        portals.generate(tree);
        portals
    }

    pub fn add(&mut self, portal: Portal) {
        let reverse = portal.reverse();
        self.inner
            .entry(portal.src)
            .expect("Node was removed")
            .or_default()
            .push(portal);
        self.inner
            .entry(reverse.src)
            .expect("Node was removed")
            .or_default()
            .push(reverse);
    }

    pub fn get(&self, index: NodeIndex) -> &[Portal] {
        self.inner
            .get(index)
            .map(|val| val.as_ref())
            .unwrap_or_default()
    }

    pub fn iter(&self) -> PortalsIter {
        PortalsIter {
            inner: self.inner.iter(),
        }
    }
}

impl<'a> IntoIterator for &'a Portals {
    type Item = &'a [Portal];

    type IntoIter = PortalsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Default for Portals {
    fn default() -> Self {
        Self::new()
    }
}
pub struct PortalsIter<'a> {
    inner: Iter<'a, NodeIndex, NodePortals>,
}

impl<'a> Iterator for PortalsIter<'a> {
    type Item = &'a [Portal];

    fn next(&mut self) -> Option<Self::Item> {
        let (_, edges) = self.inner.next()?;
        Some(edges.as_ref())
    }
}
