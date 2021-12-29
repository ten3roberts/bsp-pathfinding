use macroquad::{
    color::hsl_to_rgb,
    prelude::{draw_circle, draw_line, Color, Vec2, BLUE},
};

use crate::{util::line_intersect, Face, Side};

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

    pub(crate) fn draw(index: NodeIndex, nodes: &Nodes, thickness: f32, depth: usize) {
        let node = &nodes[index];

        let dir = Vec2::new(node.normal.y, -node.normal.x);
        let p = node.origin - dir * 100.0;
        let q = node.origin + dir * 100.0;

        let bright = depth as f32 * 0.1;

        draw_line(
            p.x,
            p.y,
            q.x,
            q.y,
            thickness,
            Color::new(bright, bright, bright, 1.0), // hsl_to_rgb(depth as f32 * 0.02, 1.0, 0.5),
        );

        for vertex in &node.vertices {
            draw_circle(vertex.x, vertex.y, thickness, BLUE);
        }

        if let Some(front) = node.front {
            Self::draw(front, nodes, thickness, depth + 1)
        }
        if let Some(back) = node.back {
            Self::draw(back, nodes, thickness, depth + 1)
        }
    }

    // /// It doesn't suffice to use slice::group or itertools::group_by since
    // /// mutable access is required.
    // fn partition(nodes: &mut Nodes, faces: &mut [(Side, Face)]) -> Partitioned {
    //     let mut side = match faces.get(0) {
    //         Some(val) => val.0,
    //         None => return Partitioned::default(),
    //     };

    //     let mut front = None;

    //     let mut back = None;
    //     let mut coplanar = Vec::new();

    //     let mut start = 0;

    //     for i in 1..faces.len() {
    //         let new_side = faces[i].0;

    //         // Break group
    //         if new_side != side || i != faces.len() - 1 {
    //             let faces = &mut faces[start..i];

    //             match side {
    //                 Side::Front => front = Self::new(nodes, faces),
    //                 Side::Back => back = Self::new(nodes, faces),
    //                 Side::Coplanar => coplanar.extend(faces.iter().flat_map(|val| val.1.vertices)),
    //                 Side::Intersecting => todo!(),
    //             }

    //             // Get new group
    //             side = new_side;
    //             start = i;
    //         }
    //     }

    //     Partitioned {
    //         front,
    //         back,
    //         coplanar,
    //     }
    // }
}

#[derive(Default, Debug)]
struct Partitioned {
    front: Option<NodeIndex>,
    back: Option<NodeIndex>,
    coplanar: Vec<Vec2>,
}
