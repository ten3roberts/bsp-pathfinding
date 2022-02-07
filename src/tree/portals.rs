use core::slice;
use std::ops::Deref;

use glam::Vec2;
use slotmap::{secondary::Iter, SecondaryMap};
use smallvec::SmallVec;

use crate::{util::face_intersect, BSPTree, Face, NodeIndex, Portal, PortalRef, Side};

#[derive(Copy, Debug, Clone, PartialEq)]
#[doc(hidden)]
pub struct ClippedFace {
    face: Face,
    // Used to determine if a face is completely inside
    pub(crate) sides: [Side; 2],
    pub(crate) adjacent: [bool; 2],

    pub src: NodeIndex,
    pub dst: NodeIndex,
}

impl ClippedFace {
    pub fn new(
        vertices: [Vec2; 2],
        sides: [Side; 2],
        adjacent: [bool; 2],
        src: NodeIndex,
        dst: NodeIndex,
    ) -> Self {
        Self {
            face: Face::new(vertices),
            sides,
            adjacent,
            src,
            dst,
        }
    }

    pub(crate) fn split(&self, p: Vec2, normal: Vec2) -> [Self; 2] {
        let intersection = face_intersect(self.into_tuple(), p, normal);

        // a is in front
        [
            Self::new(
                [self.vertices[0], intersection.point],
                [Side::Front, Side::Front],
                [self.adjacent[0], false],
                self.src,
                self.dst,
            ),
            Self::new(
                [intersection.point, self.vertices[1]],
                [Side::Front, Side::Front],
                [false, self.adjacent[1]],
                self.src,
                self.dst,
            ),
        ]
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

    /// Get the clipped face's face.
    pub fn face(&self) -> Face {
        self.face
    }
}

impl Deref for ClippedFace {
    type Target = Face;

    fn deref(&self) -> &Self::Target {
        &self.face
    }
}

type NodePortals = SmallVec<[PortalRef; 4]>;

/// Declares portals which are surfaces connecting two partitioning planes,
/// [crate::BSPNode].
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct Portals {
    inner: SecondaryMap<NodeIndex, NodePortals>,
    faces: Vec<Face>,
}

impl Portals {
    pub fn new() -> Self {
        Self {
            inner: SecondaryMap::new(),
            faces: Vec::new(),
        }
    }

    pub fn generate(&mut self, tree: &BSPTree) {
        self.extend(tree.generate_portals())
    }

    /// Adds a new portal for both src and dst
    pub fn push(&mut self, portal: ClippedFace) {
        let face = self.faces.len();
        self.faces.push(portal.face);

        assert_ne!(portal.src, portal.dst);

        self.inner
            .entry(portal.src)
            .expect("Node was removed")
            .or_default()
            .push(PortalRef {
                dst: portal.dst,
                src: portal.src,
                adjacent: portal.adjacent,
                normal: -portal.normal(),
                face,
            });
        self.inner
            .entry(portal.dst)
            .expect("Node was removed")
            .or_default()
            .push(PortalRef {
                dst: portal.src,
                src: portal.dst,
                adjacent: portal.adjacent,
                normal: portal.normal(),
                face,
            });
    }

    pub fn get(&self, index: NodeIndex) -> PortalIter {
        PortalIter {
            faces: &self.faces,
            iter: self
                .inner
                .get(index)
                .map(|val| val.as_ref())
                .unwrap_or_default()
                .iter(),
        }
    }

    pub fn iter(&self) -> PortalsIter {
        PortalsIter {
            faces: &self.faces,
            inner: self.inner.iter(),
        }
    }

    pub fn from_ref(&self, portal: PortalRef) -> Portal {
        Portal {
            face: &self.faces[portal.face],
            portal_ref: portal,
        }
    }
}

#[doc(hidden)]
pub struct PortalIter<'a> {
    faces: &'a [Face],
    iter: slice::Iter<'a, PortalRef>,
}

impl<'a> Iterator for PortalIter<'a> {
    type Item = Portal<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let portal = self.iter.next()?;

        Some(Portal {
            face: &self.faces[portal.face],
            portal_ref: *portal,
        })
    }
}

impl<'a> IntoIterator for &'a Portals {
    type Item = PortalIter<'a>;

    type IntoIter = PortalsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[doc(hidden)]
pub struct PortalsIter<'a> {
    faces: &'a [Face],
    inner: Iter<'a, NodeIndex, NodePortals>,
}

impl<'a> Iterator for PortalsIter<'a> {
    type Item = PortalIter<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let (_, portals) = self.inner.next()?;
        Some(PortalIter {
            faces: self.faces,
            iter: portals.iter(),
        })
    }
}

impl Default for Portals {
    fn default() -> Self {
        Self::new()
    }
}

impl Extend<ClippedFace> for Portals {
    fn extend<T: IntoIterator<Item = ClippedFace>>(&mut self, iter: T) {
        let iter = iter.into_iter();
        let cap = self.inner.len() + iter.size_hint().0;

        self.inner.set_capacity(cap);
        iter.for_each(|val| self.push(val))
    }
}
