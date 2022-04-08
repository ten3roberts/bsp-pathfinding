use glam::Vec2;

use crate::{util::face_intersect, Side, TOLERANCE};

pub type ClipFace<F> = (Side, F);

pub trait Face {
    type Vertex;
    type Clip: IntoIterator<Item = ClipFace<Self>>;

    fn with_radius(radius: f32, normal: Self::Vertex) -> Self;

    /// Returns the side of self in respect to other
    fn side_of(&self, other: &Self) -> Side;

    /// Return a point on the face
    /// Is idempotent
    fn point(&self) -> Self::Vertex;

    fn point_side(&self, p: &Self::Vertex) -> Side;

    /// Clip self by other.
    /// If faces are coplanar, it should return self  other
    fn clip_against(&self, other: &Self) -> Option<Self::Clip>;

    fn clip_point(&self, p: Self::Vertex) -> Result<Self::Vertex, Self::Vertex>;
    fn project(&self, )

    /// Returns the point index of `other` which touches `self`.
    fn adjacent(&self, other: &Self) -> Option<usize>;
    fn normal(&self) -> Self::Vertex;
}

pub struct Face2D {
    points: [Vec2; 2],
}

impl Face2D {
    pub fn new(points: [Vec2; 2]) -> Self {
        Self { points }
    }

    fn len(&self) -> f32 {
        self.points[1].distance(self.points[0])
    }
}

impl Face for Face2D {
    type Vertex = Vec2;

    type Clip = [ClipFace<Self>; 2];

    fn with_radius(radius: f32, normal: Vec2) -> Self {
        let perp = normal.perp();
        Self::new([perp * radius, -perp * -radius])
    }

    fn side_of(&self, other: &Self) -> Side {
        let normal = other.normal();
        let p = other.points[0];
        let a = (self.points[0] - p).dot(normal);
        let b = (self.points[1] - p).dot(normal);

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

    fn point(&self) -> Self::Vertex {
        self.points[0]
    }

    fn point_side(&self, p: &Self::Vertex) -> Side {
        let dot = (*p - self.point()).dot(self.normal());

        if dot.abs() < TOLERANCE {
            Side::Coplanar
        } else if dot <= 0.0 {
            Side::Back
        } else {
            Side::Front
        }
    }

    fn clip_against(&self, other: &Self) -> Option<Self::Clip> {
        let other_n = other.normal();

        let a = (self.points[0] - other.points[0]).dot(other_n);
        let b = (self.points[1] - other.points[0]).dot(other_n);

        if a < -TOLERANCE && b > TOLERANCE {
            let p = face_intersect(self.points, other.points[0], other_n).point;
            Some([
                (Side::Front, Self::new([p, self.points[1]])),
                (Side::Back, Self::new([self.points[0], p])),
            ])
        } else if a > TOLERANCE && b < -TOLERANCE {
            let p = face_intersect(self.points, other.points[0], other_n).point;
            Some([
                (Side::Front, Self::new([p, self.points[1]])),
                (Side::Back, Self::new([self.points[0], p])),
            ])
        } else {
            None
        }
    }

    fn clip_point(&self, p: Self::Vertex) -> Result<Vec2, Vec2> {
        let norm = self.normal();
        let len = self.len();
        let dist = (p - self.points[0]).dot(norm);
        if dist < -TOLERANCE {
            Err(self.points[0])
        } else if dist > len + TOLERANCE {
            Err(self.points[1])
        } else {
            Ok(p)
        }
    }

    fn adjacent(&self, other: &Self) -> Option<usize> {
        let norm = self.normal();
        let len = self.len();

        other.points.iter().position(|p| {
            let dist = (*p - self.points[0]).dot(norm);
            dist > -TOLERANCE && dist < len + TOLERANCE
        })
    }

    fn normal(&self) -> Self::Vertex {
        (self.points[0] - self.points[1]).perp()
    }
}
