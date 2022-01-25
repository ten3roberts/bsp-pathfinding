use glam::Vec2;

use crate::{
    util::{face_intersect, face_intersect_dir, Intersect},
    ClippedFace, Face, Side, TOLERANCE,
};

use super::{NodeIndex, Nodes};

#[derive(Debug)]
pub struct BSPNode {
    origin: Vec2,
    normal: Vec2,

    front: Option<NodeIndex>,
    back: Option<NodeIndex>,

    depth: usize,
    // Is true if this node contains normals which face each other. This
    // indicates that this is both a front and a back face
    double_planar: bool,
}

impl BSPNode {
    /// Creates a new BSPNode and inserts it into nodes.
    /// Returns None if there were not faces to create a node from
    pub fn new(nodes: &mut Nodes, faces: &[Face], depth: usize) -> Option<NodeIndex> {
        let (current, faces) = faces.split_first()?;
        // let dir = (current.vertices[1] - current.vertices[0]).normalize();
        let p = current.vertices[0];

        let mut front = Vec::new();
        let mut back = Vec::new();

        let normal = current.normal;

        let mut double_planar = false;

        for face in faces {
            let side = face.side_of(current.vertices[0], current.normal);
            match side {
                Side::Front => front.push(*face),
                Side::Back => back.push(*face),
                Side::Coplanar => {
                    double_planar = double_planar || face.normal.dot(current.normal) < 0.0
                }
                Side::Intersecting => {
                    // Split the line in two and repeat the process
                    dbg!("Intersecting");
                    let intersect = face_intersect(face.into_tuple(), p, normal);

                    let [a, b] = face.split(intersect.point);

                    assert!(a.normal.dot(face.normal) > 0.0);
                    assert!(b.normal.dot(face.normal) > 0.0);

                    front.push(a);
                    back.push(b)
                }
            }
        }

        let front = Self::new(nodes, &mut front, depth + 1);
        let back = Self::new(nodes, &mut back, depth + 1);

        assert!(current.normal.is_normalized());

        let node = Self {
            // Any point will do
            origin: current.midpoint(),
            normal: current.normal,
            double_planar,
            front,
            back,
            depth,
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
        mut portal: ClippedFace,
        root_side: Side,
    ) -> Vec<ClippedFace> {
        let node = &nodes[index];
        let side = portal.side_of(node.origin, node.normal);

        // Allow back faces to override front
        if side == Side::Front && root_side == Side::Back {
            portal.sides = [Side::Front; 2];
        }

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
                let [front, back] = portal.split(node.origin, node.normal, node.double_planar);

                assert!(front.normal.dot(portal.normal) > 0.0);
                assert!(back.normal.dot(portal.normal) > 0.0);

                let mut result = Self::clip(index, nodes, front, root_side);
                result.append(&mut Self::clip(index, nodes, back, root_side));
                result
            }
            _ => {
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
        result: &mut Vec<ClippedFace>,
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

        if node.back().is_some() {
            let portal = ClippedFace::new(face.vertices, [min_side, max_side], index, index);

            result.extend(
                Self::clip(index, nodes, portal, Side::Front)
                    .into_iter()
                    .filter(|val| val.src != val.dst && val.sides == [Side::Front; 2]),
            );
        }

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

    /// Get the bspnode's double planar.
    pub fn double_planar(&self) -> bool {
        self.double_planar
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
