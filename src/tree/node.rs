use glam::Vec2;

use crate::{util::line_intersect, Face, Side, TOLERANCE};

use super::{NodeIndex, Nodes};

pub struct BSPNode {
    origin: Vec2,
    normal: Vec2,

    front: Option<NodeIndex>,
    back: Option<NodeIndex>,

    vertices: Vec<Vec2>,
}

impl BSPNode {
    /// Creates a new BSPNode and inserts it into nodes.
    /// Returns None if there were not faces to create a node from
    pub fn new(nodes: &mut Nodes, faces: &[Face]) -> Option<NodeIndex> {
        let (current, faces) = faces.split_first()?;
        let mut vertices: Vec<_> = faces.iter().flat_map(|val| val.vertices).collect();

        let mut front = Vec::new();
        let mut back = Vec::new();

        for face in faces {
            let side = face.side_of(current);
            match side {
                Side::Front => front.push(*face),
                Side::Back => back.push(*face),
                Side::Coplanar => vertices.extend(face.vertices),
                Side::Intersecting => {
                    // Split the line in two and repeat the process

                    // Split face around this point
                    let intersect = line_intersect(
                        (face.vertices[0], face.vertices[1]),
                        (current.vertices[0], current.vertices[1]),
                    );

                    let [a, b] = face.split(intersect);

                    assert!(a.normal.dot(face.normal) > 0.0);
                    assert!(b.normal.dot(face.normal) > 0.0);

                    // Either a is in front, and b behind, or vice versa
                    if let Side::Front = a.side_of(current) {
                        front.push(a);
                        back.push(b)
                    } else {
                        front.push(b);
                        back.push(a)
                    }
                }
            }
        }

        let front = Self::new(nodes, &mut front);
        let back = Self::new(nodes, &mut back);

        let node = Self {
            // Any point will do
            origin: current.vertices[0],
            normal: current.normal,
            front,
            back,
            vertices,
        };

        Some(nodes.insert(node))
    }

    pub fn get_side(&self, point: Vec2) -> Side {
        let dot = (point - self.origin).dot(self.normal());

        if dot.abs() < TOLERANCE {
            Side::Coplanar
        } else if dot <= 0.0 {
            Side::Back
        } else {
            Side::Front
        }
    }

    /// Get the bspnode's front.
    pub fn front(&self) -> Option<NodeIndex> {
        self.front
    }

    /// Get the bspnode's back.
    pub fn back(&self) -> Option<NodeIndex> {
        self.back
    }

    /// Get a reference to the bspnode's vertices.
    pub fn vertices(&self) -> &[Vec2] {
        self.vertices.as_ref()
    }

    /// Get the bspnode's normal.
    pub fn normal(&self) -> Vec2 {
        self.normal
    }

    /// Get the bspnode's origin.
    pub fn origin(&self) -> Vec2 {
        self.origin
    }

    pub fn descendants<'a>(index: NodeIndex, nodes: &'a Nodes) -> DescendantsIter<'a> {
        DescendantsIter {
            nodes,
            stack: vec![index],
        }
    }
}

pub struct DescendantsIter<'a> {
    nodes: &'a Nodes,

    stack: Vec<NodeIndex>,
}

impl<'a> Iterator for DescendantsIter<'a> {
    type Item = (NodeIndex, &'a BSPNode);

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.stack.pop()?;

        let node = &self.nodes[index];
        if let Some(front) = node.front {
            self.stack.push(front)
        }
        if let Some(back) = node.back {
            self.stack.push(back)
        }

        Some((index, node))
    }
}
