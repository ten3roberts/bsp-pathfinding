use macroquad::prelude::Vec2;

/// Defines a 2d shape
pub struct Shape {
    vertices: Vec<Vec2>,
}

impl Shape {
    pub fn rect(size: Vec2, origin: Vec2) -> Self {
        let half_size = size * 0.5;
        let vertices = vec![
            Vec2::new(-half_size.x, -half_size.y) + origin,
            Vec2::new(half_size.x, -half_size.y) + origin,
            Vec2::new(half_size.x, half_size.y) + origin,
            Vec2::new(-half_size.x, half_size.y) + origin,
        ];

        Self { vertices }
    }

    fn faces(&self) -> Faces {
        Faces {
            vertices: &self.vertices,
            current: 0,
            len: self.vertices.len(),
        }
    }
}

struct Face {
    normal: Vec2,
    indices: [usize; 2],
}

struct Faces<'a> {
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
        let square = Shape::rect(Vec2::new(2.0, 1.0), Vec2::new(1.0, 0.0));

        let faces = square.faces();

        let normals = [-Vec2::Y, Vec2::X, Vec2::Y, -Vec2::X];

        assert!(faces
            .map(|val| val.normal)
            .inspect(|val| eprintln!("Normal: {:?}", val))
            .eq(normals));
    }
}
