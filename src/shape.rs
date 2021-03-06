use std::{array, f32::consts::TAU};

use glam::{Mat3, Mat4, Vec2, Vec3Swizzles};
use smallvec::{smallvec, SmallVec};

use crate::TOLERANCE;

/// Defines a collection of faces.
/// This struct is not neccesary to use, but helps in constructing squares and
/// other primitives.
#[derive(Default, Debug, Clone)]
pub struct Shape {
    vertices: SmallVec<[Vec2; 8]>,
}

impl Shape {
    pub fn new(vertices: &[Vec2]) -> Self {
        Self {
            vertices: SmallVec::from_slice(vertices),
        }
    }

    pub fn regular_polygon(sides: usize, radius: f32, origin: Vec2) -> Self {
        let turn = TAU / sides as f32;
        let vertices = (0..=sides)
            .map(|val| {
                let x = (turn * val as f32).cos();
                let y = (turn * val as f32).sin();

                Vec2::new(x, y) * radius + origin
            })
            .collect();

        Self { vertices }
    }

    pub fn rect(size: Vec2, origin: Vec2) -> Self {
        let half_size = size / 2.0;
        let vertices = smallvec![
            Vec2::new(-half_size.x, -half_size.y) + origin,
            Vec2::new(half_size.x, -half_size.y) + origin,
            Vec2::new(half_size.x, half_size.y) + origin,
            Vec2::new(-half_size.x, half_size.y) + origin,
            Vec2::new(-half_size.x, -half_size.y) + origin,
        ];

        Self { vertices }
    }

    pub fn faces(&self) -> Faces {
        Faces {
            vertices: &self.vertices,
            current: 0,
            len: self.vertices.len(),
        }
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq)]
/// A two dimensional face of two vertices.
/// Uses counterclockwise winding order to calculate a normal
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct Face {
    pub(crate) normal: Vec2,
    pub vertices: [Vec2; 2],
}

impl Face {
    pub fn new(vertices: [Vec2; 2]) -> Self {
        let dir = (vertices[1] - vertices[0]).normalize();
        let normal = Vec2::new(dir.y, -dir.x);
        Self { normal, vertices }
    }

    // Return the length of the face
    pub fn length(&self) -> f32 {
        (self.vertices[0] - self.vertices[1]).length()
    }

    pub fn length_squared(&self) -> f32 {
        (self.vertices[0] - self.vertices[1]).length_squared()
    }

    /// Get the face's vertices.
    pub fn vertices(&self) -> [Vec2; 2] {
        self.vertices
    }

    pub fn into_tuple(&self) -> (Vec2, Vec2) {
        (self.vertices[0], self.vertices[1])
    }

    /// Get the face's normal.
    #[inline]
    pub fn normal(&self) -> Vec2 {
        self.normal
    }

    /// Transforms the face
    pub fn transform(&self, transform: Mat3) -> Self {
        let [a, b] = self.vertices;
        Face::new([transform.transform_point2(a), transform.transform_point2(b)])
    }

    /// Transforms the face using 3d space using xz plane
    pub fn transform_3d(&self, transform: Mat4) -> Self {
        let a = transform.transform_point3(self.vertices[0].extend(0.0).xzy());
        let b = transform.transform_point3(self.vertices[1].extend(0.0).xzy());

        Self::new([a.xz(), b.xz()])
    }

    /// Returns the side self is in respect to a point and normal
    pub fn side_of(&self, p: Vec2, normal: Vec2) -> Side {
        let a = (self.vertices[0] - p).dot(normal);
        let b = (self.vertices[1] - p).dot(normal);

        if a.abs() < TOLERANCE && b.abs() < TOLERANCE {
            Side::Coplanar
        } else if a >= -TOLERANCE && b >= -TOLERANCE {
            Side::Front
        } else if a <= TOLERANCE && b <= TOLERANCE {
            Side::Back
        } else {
            Side::Intersecting
        }
    }

    /// Splits the face around `p`
    pub fn split(&self, p: Vec2, normal: Vec2) -> [Self; 2] {
        let a = (self.vertices[0] - p).dot(normal);
        if a >= -TOLERANCE {
            [
                Face::new([self.vertices[0], p]),
                Face::new([p, self.vertices[1]]),
            ]
        } else {
            [
                Face::new([p, self.vertices[1]]),
                Face::new([self.vertices[0], p]),
            ]
        }
    }

    /// Returns true if the face is touching the other face
    pub fn adjacent(&self, other: Face) -> bool {
        let p = other.midpoint();
        let a = (self.vertices[0] - p).dot(other.normal);
        let b = (self.vertices[1] - p).dot(other.normal);

        // a.signum() != b.signum()
        (a < -TOLERANCE && b > TOLERANCE) || (b < -TOLERANCE && a > TOLERANCE)
    }

    pub fn midpoint(&self) -> Vec2 {
        (self.vertices[0] + self.vertices[1]) / 2.0
    }

    /// Returns true if `other` overlaps self
    pub fn overlaps(&self, other: &Self) -> bool {
        let dir = self.dir();

        let p = (self.vertices[0]).dot(dir);
        let q = (self.vertices[1]).dot(dir);
        let a = (other.vertices[0]).dot(dir);
        let b = (other.vertices[1]).dot(dir);

        // a -- b in the direction of self
        let (a, b) = if dir.dot(other.dir()) > 0.0 {
            (a, b)
        } else {
            (b, a)
        };

        let la = q - a;
        let lb = b - p;

        let overlap = la.min(lb);

        overlap > TOLERANCE
    }

    pub fn contains_point(&self, p: Vec2) -> bool {
        let dir = self.dir();

        let d = (p - self.vertices[0]).dot(dir);

        d > -TOLERANCE && d < self.length() + TOLERANCE
    }

    pub fn dir(&self) -> Vec2 {
        (self.vertices[1] - self.vertices[0]).normalize()
    }
}

impl<'a> IntoIterator for &'a Shape {
    type Item = Face;

    type IntoIter = Faces<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.faces()
    }
}

impl<'a> IntoIterator for Face {
    type Item = Vec2;

    type IntoIter = array::IntoIter<Vec2, 2>;

    fn into_iter(self) -> Self::IntoIter {
        self.vertices.into_iter()
    }
}

impl<'a> IntoIterator for &'a Face {
    type Item = Vec2;

    type IntoIter = array::IntoIter<Vec2, 2>;

    fn into_iter(self) -> Self::IntoIter {
        self.vertices.into_iter()
    }
}

#[doc(hidden)]
pub struct Faces<'a> {
    vertices: &'a [Vec2],
    current: usize,
    len: usize,
}

impl<'a> Iterator for Faces<'a> {
    type Item = Face;

    // Generate normals from winding
    fn next(&mut self) -> Option<Self::Item> {
        if self.current + 1 == self.len {
            return None;
        }

        let a = self.vertices[self.current];
        let b = self.vertices[(self.current + 1)];

        self.current += 1;
        Some(Face::new([a, b]))
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::Shape;

    #[test]
    fn shape_rect() {
        let rect = Shape::rect(Vec2::new(2.0, 1.0), Vec2::new(1.0, 0.0));

        let faces = rect.faces();

        let normals = [-Vec2::Y, Vec2::X, Vec2::Y, -Vec2::X];

        assert!(faces.map(|val| val.normal).eq(normals));
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Side {
    Front,
    Back,
    Coplanar,
    Intersecting,
}

impl Side {
    pub fn min_side(&self, other: Self) -> Self {
        match (self, other) {
            (Side::Back, _) | (_, Side::Back) => Side::Back,
            _ => *self,
        }
    }
}
