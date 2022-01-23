use std::ops::{Deref, Div};

use glam::Vec2;

use crate::{util::face_intersect, Face, NodeIndex};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Portal<'a> {
    pub(crate) face: &'a Face,

    pub(crate) portal_ref: PortalRef,
}

impl<'a> Portal<'a> {
    /// Get the portal's portal ref.
    pub fn portal_ref(&self) -> PortalRef {
        self.portal_ref
    }

    pub fn dst(&self) -> NodeIndex {
        self.portal_ref.dst
    }

    /// Get the portal's src.
    pub fn src(&self) -> NodeIndex {
        self.portal_ref.src
    }

    /// Get the portal's face.
    pub fn face(&self) -> &Face {
        self.face
    }

    // Returns true if the line is contained on the surface of the portal
    pub(crate) fn try_clip(&self, start: Vec2, end: Vec2) -> Option<Vec2> {
        let p = face_intersect(self.into_tuple(), start, (end - start).perp());

        // let rel = (p - self.vertices[0]).dot(self.vertices[1] - self.vertices[0]);
        dbg!(p, (self.vertices[1] - self.vertices[0]).length());
        if p.distance > 0.0 && p.distance < 1.0 {
            Some(p.point)
        } else {
            None
        }
    }
}

impl<'a> Deref for Portal<'a> {
    type Target = Face;

    fn deref(&self) -> &Self::Target {
        &self.face
    }
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PortalRef {
    pub(crate) src: NodeIndex,
    pub(crate) dst: NodeIndex,
    pub(crate) face: usize,
}
