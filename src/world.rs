use crate::Shape;

/// Represents the navigable world or scene
pub struct World {
    shapes: Vec<Shape>,
}

impl Default for World {
    fn default() -> Self {
        Self {
            shapes: Default::default(),
        }
    }
}

impl World {
    pub fn new(shapes: &[Shape]) -> Self {
        Self {
            shapes: shapes.to_vec(),
        }
    }

    pub fn add_shape(&mut self, shape: Shape) {
        self.shapes.push(shape)
    }

    pub fn draw(&self, thickness: f32) {
        for shape in &self.shapes {
            shape.draw(thickness);
        }
    }
}
