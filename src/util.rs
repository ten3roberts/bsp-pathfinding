use std::ops::Deref;

use glam::Vec2;

/// Returns the intersection points between two lines
pub fn line_intersect(a: (Vec2, Vec2), b: (Vec2, Vec2)) -> Vec2 {
    let a_dir = (a.1 - a.0).normalize_or_zero();
    let b_dir = (b.1 - b.0).normalize_or_zero();
    let rel = b.0 - a.0;
    let dot = a_dir.perp_dot(b_dir);
    // project rel onto plane defined by a
    let length = rel.perp_dot(a_dir);

    let p = b.0 + b_dir * (length / dot);

    p
}

pub fn face_intersect(a: (Vec2, Vec2), p: Vec2, normal: Vec2) -> Intersect {
    let dir = a.1 - a.0;
    face_intersect_dir(a.0, dir, p, normal)
}

pub fn face_intersect_dir(a: Vec2, dir: Vec2, p: Vec2, normal: Vec2) -> Intersect {
    let l = (p - a).dot(normal) / (dir.dot(normal));
    Intersect::new(a + dir * l, l)
}

/// Returns the intersection point between two lines as a measure of the length
/// along b
pub fn line_intersect_dir(a: (Vec2, Vec2), b: Vec2, b_dir: Vec2) -> f32 {
    let a_dir = (a.1 - a.0).normalize_or_zero();
    let rel = b - a.0;
    let dot = a_dir.perp_dot(b_dir);
    // project rel onto plane defined by a
    let length = rel.perp_dot(a_dir);

    length / dot
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Intersect {
    pub point: Vec2,
    pub distance: f32,
}

impl Intersect {
    /// Returns the point closest with the furthest absolute distance to the
    /// line origin
    pub fn min_abs(&self, other: Self) -> Self {
        if self.distance.abs() < other.distance.abs() {
            *self
        } else {
            other
        }
    }

    pub fn new(point: Vec2, distance: f32) -> Self {
        Self { point, distance }
    }
}

impl Deref for Intersect {
    type Target = Vec2;

    fn deref(&self) -> &Self::Target {
        &self.point
    }
}
