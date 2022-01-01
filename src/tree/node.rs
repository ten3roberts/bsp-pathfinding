use glam::Vec2;
use ordered_float::OrderedFloat;

use crate::{
    util::{line_intersect, line_intersect_dir},
    Face, Side, TOLERANCE,
};

use super::{
    edges::{Edge, NodeEdges},
    NodeIndex, Nodes,
};

pub struct BSPNode {
    origin: Vec2,
    normal: Vec2,

    front: Option<NodeIndex>,
    back: Option<NodeIndex>,

    vertices: [Vec2; 2],

    /// Represents how far the line extends, for visualization purposes
    bounds: Option<[Vec2; 2]>,

    depth: usize,
}

impl BSPNode {
    /// Creates a new BSPNode and inserts it into nodes.
    /// Returns None if there were not faces to create a node from
    pub fn new(
        nodes: &mut Nodes,
        faces: &[Face],
        mut bounds: Option<Vec<Face>>,
        depth: usize,
    ) -> Option<NodeIndex> {
        let (current, faces) = faces.split_first()?;
        let mut vertices = current.vertices;
        let dir = (vertices[1] - vertices[0]).normalize();

        let mut front = Vec::new();
        let mut back = Vec::new();

        for face in faces {
            let side = face.side_of(current);
            match side {
                Side::Front => front.push(*face),
                Side::Back => back.push(*face),
                Side::Coplanar => vertices = Self::merge_plane(vertices, dir, face.vertices()),
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

        // Calculate how long this partition reaches. This is only used for
        // visualization purposes, and does nothing if `None` is passed
        let [p, q] = vertices;
        let min = bounds
            .iter()
            .flatten()
            .map(|bound| line_intersect_dir(bound.into_tuple(), p, dir))
            .filter(|val| val.is_finite() && *val > TOLERANCE)
            .min_by_key(|val| OrderedFloat(*val));

        let max = bounds
            .iter()
            .flatten()
            .map(|bound| line_intersect_dir(bound.into_tuple(), q, -dir))
            .filter(|val| val.is_finite() && *val > TOLERANCE)
            .min_by_key(|val| OrderedFloat(val.abs()));

        let node_bounds = if let (Some(min), Some(max)) = (min, max) {
            let a = p + dir * min;
            let b = q - dir * max;
            Some([a, b])
        } else {
            None
        };

        if let Some(bounds) = bounds.as_mut() {
            bounds.push(*current)
        }

        // Free up space before recursing
        drop(faces);

        let front = Self::new(nodes, &mut front, bounds.clone(), depth + 1);
        let back = Self::new(nodes, &mut back, bounds.clone(), depth + 1);

        assert!(current.normal.is_normalized());

        let node = Self {
            // Any point will do
            origin: current.midpoint(),
            normal: current.normal,
            front,
            back,
            vertices,
            bounds: node_bounds,
            depth,
        };

        Some(nodes.insert(node))
    }

    // Merges the plane by only retaining the extreme vertices in the resulting plane
    pub fn merge_plane(current: [Vec2; 2], dir: Vec2, other: [Vec2; 2]) -> [Vec2; 2] {
        let mut plane = [current[0], current[1], other[0], other[1]];
        plane.sort_unstable_by_key(|val| OrderedFloat(val.dot(dir)));

        let plane = [plane[0], plane[3]];
        plane
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

    /// Get the bspnode's bounds.
    pub fn bounds(&self) -> Option<[Vec2; 2]> {
        self.bounds
    }

    /// Get the bspnode's depth.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Returns the transitive node of this node.
    /// This is the child node that is the closest node in front of the current
    /// node.
    pub(crate) fn transitive_node(index: NodeIndex, point: Vec2, nodes: &Nodes) -> NodeIndex {
        let node = &nodes[index];

        // Start with the front child
        let mut index = match node.front {
            Some(val) => val,
            None => return index,
        };

        loop {
            let node = &nodes[index];

            let rel = node.origin - point;

            let child = if rel.dot(node.normal) > TOLERANCE {
                node.back
            } else {
                node.front
            };

            index = match child {
                Some(val) => val,
                None => return index,
            };
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
