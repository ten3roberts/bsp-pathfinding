use glam::Vec2;
use ordered_float::OrderedFloat;
use slotmap::Key;

use crate::{
    util::{face_intersect, face_intersect_dir, Intersect},
    Face, Portal, Side, TOLERANCE,
};

use super::{NodeIndex, Nodes};

#[derive(Debug)]
pub struct BSPNode {
    origin: Vec2,
    normal: Vec2,

    front: Option<NodeIndex>,
    back: Option<NodeIndex>,

    depth: usize,
}

impl BSPNode {
    /// Creates a new BSPNode and inserts it into nodes.
    /// Returns None if there were not faces to create a node from
    pub fn new(nodes: &mut Nodes, faces: &[Face], depth: usize) -> Option<NodeIndex> {
        let (current, faces) = faces.split_first()?;
        let mut vertices = current.vertices;
        let dir = (vertices[1] - vertices[0]).normalize();

        let mut front = Vec::new();
        let mut back = Vec::new();

        let p = current.vertices[0];
        let normal = current.normal;

        for face in faces {
            let side = face.side_of(current.vertices[0], current.normal);
            match side {
                Side::Front => front.push(*face),
                Side::Back => back.push(*face),
                Side::Coplanar => vertices = Self::merge_plane(vertices, dir, face.vertices()),
                Side::Intersecting => {
                    // Split the line in two and repeat the process

                    // Split face around this point
                    let intersect = face_intersect((face.vertices[0], face.vertices[1]), p, normal);

                    let [mut a, mut b] = face.split(intersect);

                    dbg!(a.normal(), b.normal(), face.normal());
                    a.normal *= a.normal.dot(face.normal).signum();
                    b.normal *= b.normal.dot(face.normal).signum();
                    assert!(a.normal.dot(face.normal) > 0.0);
                    assert!(b.normal.dot(face.normal) > 0.0);

                    // Either a is in front, and b behind, or vice versa
                    if let Side::Front = a.side_of(p, normal) {
                        front.push(a);
                        back.push(b)
                    } else {
                        front.push(b);
                        back.push(a)
                    }
                }
            }
        }

        // Free up space before recursing
        drop(faces);

        let front = Self::new(nodes, &mut front, depth + 1);
        let back = Self::new(nodes, &mut back, depth + 1);

        assert!(current.normal.is_normalized());

        let node = Self {
            // Any point will do
            origin: current.midpoint(),
            normal: current.normal,
            front,
            back,
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

    /// Get the bspnode's normal.
    #[inline]
    pub fn normal(&self) -> Vec2 {
        self.normal
    }

    /// Get the bspnode's origin.
    #[inline]
    pub fn origin(&self) -> Vec2 {
        self.origin
    }

    pub fn descendants<'a>(index: NodeIndex, nodes: &'a Nodes) -> DescendantsIter<'a> {
        DescendantsIter {
            nodes,
            stack: vec![index],
        }
    }

    /// Get the bspnode's depth.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Clips a face by the BSP faces and returns several smaller faces
    pub fn clip(
        index: NodeIndex,
        nodes: &Nodes,
        mut portal: Portal,
        root_side: Side,
    ) -> Vec<Portal> {
        let node = &nodes[index];
        let side = portal.side_of(node.origin, node.normal);

        // The face is entirely in front of the node
        match (side, node.front, node.back) {
            (Side::Coplanar, Some(front), Some(back)) => {
                // portal.src = NodeIndex::null();
                Self::clip(front, nodes, portal, Side::Front)
                    .into_iter()
                    .map(|val| Self::clip(back, nodes, val, Side::Back))
                    .flatten()
                    .collect()
            }
            (Side::Coplanar, Some(front), _) => Self::clip(front, nodes, portal, Side::Front),
            (Side::Coplanar, _, Some(back)) => Self::clip(back, nodes, portal, Side::Back),
            (Side::Front, Some(front), _) => Self::clip(front, nodes, portal, root_side),
            (Side::Back, _, Some(back)) => Self::clip(back, nodes, portal, root_side),
            (Side::Intersecting, _, _) => {
                // Split the face at the intersection
                let [a, b] = portal.split(node.origin, node.normal);

                let mut result = Self::clip(index, nodes, a, root_side);
                result.append(&mut Self::clip(index, nodes, b, root_side));
                result
            }
            _ => {
                dbg!(side);
                if root_side == Side::Back {
                    portal.dst = index;
                } else {
                    portal.src = index;
                }
                vec![portal]
            }
        }
    }

    pub fn generate_portals(
        index: NodeIndex,
        nodes: &Nodes,
        mut clipping_planes: Vec<Face>,
        result: &mut Vec<Portal>,
    ) {
        let node = &nodes[index];
        let dir = Vec2::new(node.normal.y, -node.normal.x);
        let mut min = Intersect::new(Vec2::ZERO, f32::MAX);
        let mut min_side = Side::Coplanar;
        let mut max_side = Side::Coplanar;
        let mut max = Intersect::new(Vec2::ZERO, f32::MAX);

        clipping_planes.iter().for_each(|val| {
            let intersect = face_intersect_dir(node.origin, dir, val.vertices[0], val.normal());
            if !intersect.distance.is_finite() {
                return;
            }

            let side = if (val.vertices[0] - node.origin()).dot(val.normal()) > 0.0 {
                Side::Front
            } else {
                Side::Back
            };

            if intersect.distance > 0.0 && intersect.distance < max.distance {
                max = intersect;
                max_side = side;
            }
            if intersect.distance < 0.0 && intersect.distance.abs() < min.distance.abs() {
                min = intersect;
                min_side = side;
            }
        });

        let face = Face::new([min.point, max.point]);

        assert_ne!(min_side, Side::Coplanar);
        assert_ne!(max_side, Side::Coplanar);

        let portal = Portal::new(face.vertices, [min_side, max_side], index, index);

        result.extend(
            Self::clip(index, nodes, portal, Side::Front)
                .into_iter()
                .filter(|val| {
                    dbg!(val);
                    // assert_ne!(val.src, val.dst);
                    assert!(!val.src.is_null());
                    assert!(!val.dst.is_null());

                    // true
                    val.sides() == [Side::Front, Side::Front]
                }),
        );

        // Add the current nodes clip plane before recursing
        // result.push(portal);
        clipping_planes.push(face);

        // Clone the clipping faces since the descendants of the children will
        // also be added to the clipping planes,
        // and we want to keep the clipping planes separated for subtrees.
        if let Some(child) = node.front {
            Self::generate_portals(child, nodes, clipping_planes.clone(), result);
        }

        if let Some(child) = node.back {
            Self::generate_portals(child, nodes, clipping_planes.clone(), result);
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.front.is_none() && self.back.is_none()
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
