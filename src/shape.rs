use macroquad::prelude::{draw_line, Vec2, BLACK, BLUE};

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
            let a = self.vertices[face.indices[1]];
            let b = self.vertices[face.indices[0]];

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

pub struct Face {
    normal: Vec2,
    indices: [usize; 2],
}

impl Face {
    /// Get the face's indices.
    pub fn indices(&self) -> [usize; 2] {
        self.indices
    }

    /// Get the face's normal.
    pub fn normal(&self) -> Vec2 {
        self.normal
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

        let a = self.current;
        let b = (self.current + 1) % self.len;

        let dir = (self.vertices[b] - self.vertices[a]).normalize();
        let normal = Vec2::new(dir.y, -dir.x);

        self.current += 1;

        Some(Face {
            normal,
            indices: [a, b],
        })
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
