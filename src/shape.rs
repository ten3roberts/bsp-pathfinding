use macroquad::prelude::{draw_line, Vec2, BLACK, BLUE};

use crate::TOLERANCE;

/// Defines a 2d shape
#[derive(Default, Debug, Clone)]
pub struct Shape {
    vertices: Vec<Vec2>,
}

const NORMAL_LENGTH: f32 = 30.0;

impl Shape {
    pub fn new(vertices: &[Vec2]) -> Self {
        Self {
            vertices: vertices.to_vec(),
        }
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

    pub fn draw(&self, thickness: f32) {
        for face in self.faces() {
            let a = face.vertices[1];
            let b = face.vertices[0];

            let normal = face.normal();

            draw_line(a.x, a.y, b.x, b.y, thickness, BLACK);
            let mid = (a + b) / 2.0;
            draw_line(
                mid.x,
                mid.y,
                mid.x + normal.x * NORMAL_LENGTH,
                mid.y + normal.y * NORMAL_LENGTH,
                thickness,
                BLUE,
            )
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

    /// Get the face's normal.
    pub fn normal(&self) -> Vec2 {
        self.normal
    }

    /// Returns the side self is in respect to `face`
    pub fn side_of(&self, face: &Face) -> Side {
        let p = face.vertices[0];
        let a = (p - self.vertices[0]).dot(face.normal);
        let b = (p - self.vertices[1]).dot(face.normal);

        if a.abs() < TOLERANCE && b.abs() < TOLERANCE {
            Side::Coplanar
        } else if a >= 0.0 && b >= 0.0 {
            Side::Front
        } else if a <= 0.0 && b <= 0.0 {
            Side::Back
        } else {
            Side::Intersecting
        }
    }

    /// Splits the face around `p`
    pub fn split(&self, p: Vec2) -> [Self; 2] {
        [
            Face::new([p, self.vertices[0]]),
            Face::new([self.vertices[1], p]),
        ]
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
    use macroquad::prelude::Vec2;

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
