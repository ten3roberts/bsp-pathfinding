use std::{array, f32::consts::TAU};

use glam::Vec2;

use crate::{util::Intersect, TOLERANCE};

/// Defines a 2d shape
#[derive(Default, Debug, Clone)]
pub struct Shape {
    vertices: Vec<Vec2>,
}

impl Shape {
    pub fn new(vertices: &[Vec2]) -> Self {
        Self {
            vertices: vertices.to_vec(),
        }
    }

    pub fn regular_polygon(sides: usize, radius: f32, origin: Vec2) -> Self {
        let turn = TAU / sides as f32;
        let vertices = (0..sides)
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
        let vertices = vec![
            Vec2::new(-half_size.x, -half_size.y) + origin,
            Vec2::new(half_size.x, -half_size.y) + origin,
            Vec2::new(half_size.x, half_size.y) + origin,
            Vec2::new(-half_size.x, half_size.y) + origin,
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
pub struct Face {
    pub normal: Vec2,
    pub vertices: [Vec2; 2],
}

impl Face {
    pub fn new(vertices: [Vec2; 2]) -> Self {
        let dir = (vertices[1] - vertices[0]).normalize();
        let normal = Vec2::new(dir.y, -dir.x);
        Self { normal, vertices }
    }

    /// Get the face's vertices.
    pub fn vertices(&self) -> [Vec2; 2] {
        self.vertices
    }

    pub fn into_tuple(&self) -> (Vec2, Vec2) {
        (self.vertices[0], self.vertices[1])
    }

    /// Get the face's normal.
    pub fn normal(&self) -> Vec2 {
        self.normal
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
    pub fn split(&self, p: Intersect) -> [Self; 2] {
        [
            Face::new([p.point, self.vertices[0]]),
            Face::new([self.vertices[1], p.point]),
        ]
    }

    pub fn midpoint(&self) -> Vec2 {
        (self.vertices[0] + self.vertices[1]) / 2.0
    }
}

impl<'a> IntoIterator for &'a Shape {
    type Item = Face;

    type IntoIter = Faces<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.faces()
    }
}

impl<'a> IntoIterator for &'a Face {
    type Item = Vec2;

    type IntoIter = array::IntoIter<Vec2, 2>;

    fn into_iter(self) -> Self::IntoIter {
        self.vertices.into_iter()
    }
}

pub struct Faces<'a> {
    vertices: &'a [Vec2],
    current: usize,
    len: usize,
}

impl<'a> Iterator for Faces<'a> {
    type Item = Face;

    // Generate normals from winding
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.len {
            return None;
        }

        let a = self.vertices[self.current];
        let b = self.vertices[(self.current + 1) % self.len];

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

        assert!(faces
            .map(|val| val.normal)
            .inspect(|val| eprintln!("Normal: {:?}", val))
            .eq(normals));
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
    pub fn min(&self, other: Self) -> Self {
        match (self, other) {
            (Side::Back, _) => Side::Back,
            (_, Side::Back) => Side::Back,
            _ => *self,
        }
    }
}
