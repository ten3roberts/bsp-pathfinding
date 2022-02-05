use std::ops::Deref;

use glam::Vec2;

use crate::{util::face_intersect, Face, NodeIndex};

#[derive(Debug, Clone, Copy, PartialEq)]
/// Represents a surface connecting two nodes
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

    /// Returns the normal which points into the portal
    pub fn normal(&self) -> Vec2 {
        self.portal_ref.normal
    }

    /// Get the portal's face.
    pub fn face(&self) -> &Face {
        self.face
    }

    // Returns true if the line is contained on the surface of the portal
    pub(crate) fn try_clip(&self, start: Vec2, end: Vec2, margin: f32) -> Option<Vec2> {
        let (l, r) = self.apply_margin(margin);
        let p = face_intersect((l, r), start, (end - start).perp());

        // let rel = (p - self.vertices[0]).dot(self.vertices[1] - self.vertices[0]);
        if p.distance > 0.0 && p.distance < 1.0 {
            Some(p.point)
        } else {
            None
        }
    }

    pub(crate) fn clip(&self, start: Vec2, end: Vec2, margin: f32) -> Vec2 {
        let (l, r) = self.apply_margin(margin);
        let p = face_intersect((l, r), start, (end - start).perp());

        // let rel = (p - self.vertices[0]).dot(self.vertices[1] - self.vertices[0]);
        if p.distance < 0.0 {
            l
        } else if p.distance > 1.0 {
            r
        } else {
            p.point
        }
    }

    pub fn apply_margin(&self, margin: f32) -> (Vec2, Vec2) {
        let dir = self.face.dir();
        let l = self.face.vertices[0] + margin * dir;
        let r = self.face.vertices[1] - margin * dir;
        (l, r)
    }
}

impl<'a> Deref for Portal<'a> {
    type Target = PortalRef;

    fn deref(&self) -> &Self::Target {
        &self.portal_ref
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// References a portal
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct PortalRef {
    pub(crate) src: NodeIndex,
    pub(crate) dst: NodeIndex,
    pub(crate) face: usize,
    // Normal may be different than the face due to the normal pointing through
    // the portal
    pub(crate) normal: Vec2,
}

impl PortalRef {
    /// Returns the normal which points into the portal
    pub fn normal(&self) -> Vec2 {
        self.normal
    }

    /// Get the portal ref's src.
    pub fn src(&self) -> NodeIndex {
        self.src
    }

    /// Get the portal ref's dst.
    pub fn dst(&self) -> NodeIndex {
        self.dst
    }

    /// Get a mutable reference to the portal ref's normal.
    pub fn normal_mut(&mut self) -> &mut Vec2 {
        &mut self.normal
    }
}
