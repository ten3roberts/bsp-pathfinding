use glam::Vec2;
use itertools::Itertools;
use ordered_float::NotNan;

use crate::{
    astar, BSPNode, BSPTree, Face, NodeIndex, NodePayload, Path, Portals, SearchInfo, TOLERANCE,
};

/// Contains a layered graph and edges necessary for path finding
#[derive(Default)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct LayeredNavigationContext {
    layers: Vec<(NotNan<f32>, BSPTree, Portals)>,
}

impl LayeredNavigationContext {
    pub fn new(faces: impl IntoIterator<Item = (f32, Face)>) -> Self {
        let layers = faces
            .into_iter()
            .map(|(k, v)| (NotNan::new(k).unwrap(), v))
            .sorted_by_key(|(k, _)| *k)
            .group_by(|(k, _)| (k / TOLERANCE).round() as i32)
            .into_iter()
            .flat_map(|(k, faces)| {
                let tree = BSPTree::new(faces.map(|(_, v)| v).collect_vec())?;
                let mut portals = Portals::new();
                portals.generate(&tree);
                Some((NotNan::new(k as f32 * TOLERANCE).unwrap(), tree, portals))
            })
            .collect_vec();

        Self { layers }
    }

    pub fn layers(&self) -> impl Iterator<Item = &(NotNan<f32>, BSPTree, Portals)> {
        self.layers.iter()
    }

    pub fn locate(&self, layer: f32, point: Vec2) -> Option<NodePayload> {
        self.layer(layer).map(|v| v.1.locate(point))
    }

    pub fn layer(&self, layer: f32) -> Option<&(NotNan<f32>, BSPTree, Portals)> {
        let layer = NotNan::new(layer).ok()?;
        let mut slice = &self.layers[..];
        loop {
            let i = slice.len() / 2;
            let mid = &slice[i];

            if slice.len() == 1 {
                return Some(mid);
            }

            match layer.cmp(&mid.0) {
                std::cmp::Ordering::Less => slice = &slice[0..i],
                std::cmp::Ordering::Equal => return Some(mid),
                std::cmp::Ordering::Greater => slice = &slice[i..],
            }
        }
    }

    pub fn node(&self, layer: f32, index: NodeIndex) -> Option<&BSPNode> {
        self.layer(layer).and_then(|v| v.1.node(index))
    }

    pub fn find_path_inc<'a>(
        &self,
        layer: f32,
        start: Vec2,
        end: Vec2,
        heuristic: impl Fn(Vec2, Vec2) -> f32,
        info: SearchInfo,
        path: &'a mut Option<Path>,
    ) -> Option<&'a mut Path> {
        let (_, tree, portals) = self.layer(layer)?;
        astar(tree, portals, start, end, heuristic, info, path)
    }

    pub fn find_path(
        &self,
        layer: f32,
        start: Vec2,
        end: Vec2,
        heuristic: impl Fn(Vec2, Vec2) -> f32,
        info: SearchInfo,
    ) -> Option<Path> {
        let mut path = None;
        self.find_path_inc(layer, start, end, heuristic, info, &mut path);
        path
    }
}
