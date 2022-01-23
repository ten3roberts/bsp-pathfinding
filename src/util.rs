use std::ops::Deref;

use glam::Vec2;

pub(crate) fn face_intersect(a: (Vec2, Vec2), p: Vec2, normal: Vec2) -> Intersect {
    let dir = a.1 - a.0;
    face_intersect_dir(a.0, dir, p, normal)
}

pub(crate) fn face_intersect_dir(a: Vec2, dir: Vec2, p: Vec2, normal: Vec2) -> Intersect {
    let l = (p - a).dot(normal) / (dir.dot(normal));
    Intersect::new(a + dir * l, l)
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub(crate) struct Intersect {
    pub point: Vec2,
    pub distance: f32,
}

impl Intersect {
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
