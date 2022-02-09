use glam::Vec2;
use rpds::Vector;
use smallvec::{smallvec, SmallVec};

use crate::{
    util::{face_intersect, face_intersect_dir, Intersect},
    ClippedFace, Face, Side, TOLERANCE,
};

use super::{NodeIndex, Nodes};

#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
/// Represents  a single node in the binary tree.
/// The node constitutes of a splitting plane and children behind and in front
/// of the plane.
///
/// A node can be double planar, which means that the partitioning plane
/// contains two faces with opposite facing normals.
pub struct BSPNode {
    origin: Vec2,
    normal: Vec2,

    front: Option<NodeIndex>,
    back: Option<NodeIndex>,

    faces: SmallVec<[Face; 2]>,

    depth: usize,
}

impl BSPNode {
    /// Creates a new BSPNode and inserts it into nodes.
    /// Returns None if there were not faces to create a node from
    pub fn from_faces(nodes: &mut Nodes, faces: &[Face], depth: usize) -> Option<NodeIndex> {
        let (current, faces) = faces.split_first()?;
        // let dir = (current.vertices[1] - current.vertices[0]).normalize();
        let p = current.vertices[0];

        let mut front = Vec::new();
        let mut back = Vec::new();

        let mut coplanar = smallvec![*current];

        let normal = current.normal;

        for face in faces {
            let side = face.side_of(current.vertices[0], current.normal);
            match side {
                Side::Front => front.push(*face),
                Side::Back => back.push(*face),
                Side::Coplanar => {
                    coplanar.push(*face);
                }
                Side::Intersecting => {
                    // Split the line in two and repeat the process
                    let intersect = face_intersect(face.into_tuple(), p, normal);

                    let [a, b] = face.split(intersect.point, normal);

                    assert_eq!(a.side_of(p, normal), Side::Front);
                    assert_eq!(b.side_of(p, normal), Side::Back);

                    assert!(a.normal.dot(face.normal) > 0.0);
                    assert!(b.normal.dot(face.normal) > 0.0);

                    front.push(a);
                    back.push(b)
                }
            }
        }

        let front = Self::from_faces(nodes, &front, depth + 1);
        let back = Self::from_faces(nodes, &back, depth + 1);

        assert!(current.normal.is_normalized());

        let node = Self {
            // Any point will do
            origin: current.midpoint(),
            faces: coplanar,
            normal: current.normal,
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

    pub fn descendants(index: NodeIndex, nodes: &Nodes) -> Descendants {
        Descendants {
            nodes,
            stack: vec![index],
        }
    }

    /// Get the bspnode's depth.
    pub fn depth(&self) -> usize {
        self.depth
    }

    fn get_adjacent_side(&self, p: Vec2, other: Vec2) -> Option<Side> {
        self.faces
            .iter()
            .filter_map(|f| {
                if f.contains_point(p) {
                    Some(if (other - f.vertices[0]).dot(f.normal()) > 0.0 {
                        Side::Front
                    } else {
                        Side::Back
                    })
                } else {
                    None
                }
            })
            .reduce(|acc, val| acc.min_side(val))
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
        let a = (portal.vertices[0] - node.origin).dot(node.normal);
        let b = (portal.vertices[1] - node.origin).dot(node.normal);

        // a is touching the plane
        if a.abs() < TOLERANCE {
            if let Some(ad) = node.get_adjacent_side(portal.vertices[0], portal.vertices[1]) {
                portal.adjacent[0] = true;
                portal.sides[0] = ad;
            }
        }
        // b is touching the plane
        if b.abs() < TOLERANCE {
            if let Some(ad) = node.get_adjacent_side(portal.vertices[1], portal.vertices[0]) {
                portal.adjacent[1] = true;
                portal.sides[1] = ad;
            }
        }

        Self::clip_new(index, nodes, portal, side, root_side)
    }

    fn clip_new(
        index: NodeIndex,
        nodes: &Nodes,
        mut portal: ClippedFace,
        side: Side,
        root_side: Side,
    ) -> Vec<ClippedFace> {
        let node = &nodes[index];
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
                let [front, back] = portal.split(node.origin, node.normal);

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
        clipping_planes: &Vector<Face>,
        result: &mut impl Extend<ClippedFace>,
    ) {
        let node = &nodes[index];
        let dir = Vec2::new(node.normal.y, -node.normal.x);
        let mut min = Intersect::new(Vec2::ZERO, f32::MAX);
        let mut adjacent = [false, false];
        let mut max = Intersect::new(Vec2::ZERO, f32::MAX);

        clipping_planes.iter().for_each(|val| {
            let intersect = face_intersect_dir(node.origin, dir, val.vertices[0], val.normal());
            if !intersect.distance.is_finite() {
                return;
            }

            let ad = val.contains_point(intersect.point);

            if intersect.distance > 0.0 && intersect.distance < max.distance {
                max = intersect;
                adjacent[0] = ad;
            }
            if intersect.distance < 0.0 && intersect.distance.abs() < min.distance.abs() {
                min = intersect;
                adjacent[1] = ad;
            }
        });

        let portal = ClippedFace::new(
            [max.point, min.point],
            [Side::Front, Side::Front],
            adjacent,
            index,
            index,
        );

        result.extend(
            Self::clip(index, nodes, portal, Side::Front)
                .into_iter()
                .filter(|val| {
                    val.src != val.dst
                        && val.sides == [Side::Front; 2]
                        && !node.faces.iter().any(|face| face.overlaps(val))
                }),
        );

        // Add the current nodes clip plane before recursing
        // result.push(portal);
        let clipping_planes = node
            .faces
            .iter()
            .fold(clipping_planes.clone(), |acc, val| acc.push_back(*val));

        // Clone the clipping faces since the descendants of the children will
        // also be added to the clipping planes,
        // and we want to keep the clipping planes separated for subtrees.
        if let Some(child) = node.front {
            Self::generate_portals(child, nodes, &clipping_planes, result);
        }

        if let Some(child) = node.back {
            Self::generate_portals(child, nodes, &clipping_planes, result);
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.front.is_none() && self.back.is_none()
    }

    /// Get a reference to the bspnode's faces.
    pub fn faces(&self) -> &[Face] {
        &self.faces
    }
}

pub struct Descendants<'a> {
    nodes: &'a Nodes,

    stack: Vec<NodeIndex>,
}

impl<'a> Iterator for Descendants<'a> {
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
