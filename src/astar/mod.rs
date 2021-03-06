use core::slice;
use std::{
    collections::{BinaryHeap, HashSet},
    ops::{Deref, DerefMut, RangeBounds},
};

use glam::Vec2;
use slotmap::{secondary::Entry, Key, SecondaryMap};
use smallvec::{Drain, SmallVec};

use crate::{BSPTree, NodeIndex, Portal, PortalRef, Portals, TOLERANCE};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WayPoint {
    point: Vec2,
    node: NodeIndex,
    portal: Option<PortalRef>,
}

impl Deref for WayPoint {
    type Target = Vec2;

    fn deref(&self) -> &Self::Target {
        &self.point
    }
}

impl WayPoint {
    pub fn new(point: Vec2, node: NodeIndex, portal: Option<PortalRef>) -> Self {
        Self {
            point,
            node,
            portal,
        }
    }

    /// Get the way point's point.
    pub fn point(&self) -> Vec2 {
        self.point
    }

    /// Get the way point's portal.
    pub fn portal(&self) -> Option<PortalRef> {
        self.portal
    }
}

#[derive(Debug, Clone, Default)]
pub struct Path {
    points: SmallVec<[WayPoint; 8]>,
}

impl<'a> IntoIterator for &'a Path {
    type Item = &'a WayPoint;

    type IntoIter = slice::Iter<'a, WayPoint>;

    fn into_iter(self) -> Self::IntoIter {
        self.points.iter()
    }
}

impl Path {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_points(points: impl Into<SmallVec<[WayPoint; 8]>>) -> Self {
        Self {
            points: points.into(),
        }
    }

    /// Get a reference to the path's points.
    pub fn points(&self) -> &[WayPoint] {
        self.points.as_ref()
    }

    pub fn push(&mut self, value: WayPoint) {
        self.points.push(value)
    }

    pub fn append(&mut self, other: &mut Self) {
        self.points.append(&mut other.points)
    }

    /// Creates a path using the euclidian path
    pub fn euclidian(start: Vec2, end: Vec2) -> Path {
        Path::from_points(vec![
            WayPoint::new(start, NodeIndex::null(), None),
            WayPoint::new(end, NodeIndex::null(), None),
        ])
    }

    pub fn clear(&mut self) {
        self.points.clear()
    }

    pub fn drain<R: RangeBounds<usize>>(&mut self, range: R) -> Drain<'_, [WayPoint; 8]> {
        self.points.drain(range)
    }
}

impl Deref for Path {
    type Target = [WayPoint];

    fn deref(&self) -> &Self::Target {
        &self.points
    }
}

impl DerefMut for Path {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.points
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Backtrace<'a> {
    // Index to the portal
    node: NodeIndex,
    // The first side of the portal
    point: Vec2,
    portal: Option<Portal<'a>>,
    prev: Option<NodeIndex>,
    start_cost: f32,
    total_cost: f32,
}

impl<'a> Backtrace<'a> {
    fn start(node: NodeIndex, point: Vec2, heuristic: f32) -> Self {
        Self {
            node,
            point,
            portal: None,
            prev: None,
            start_cost: 0.0,
            total_cost: heuristic,
        }
    }

    fn new(portal: Portal<'a>, point: Vec2, prev: &Backtrace, heuristic: f32) -> Self {
        let start_cost = prev.start_cost + point.distance(prev.point);
        Self {
            node: portal.dst(),
            portal: Some(portal),
            point,
            prev: Some(prev.node),
            start_cost,
            total_cost: start_cost + heuristic,
        }
    }
}

// Order by lowest total_cost
impl<'a> PartialOrd for Backtrace<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.total_cost.partial_cmp(&self.total_cost)
    }
}

impl<'a> Eq for Backtrace<'a> {}

impl<'a> Ord for Backtrace<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .total_cost
            .partial_cmp(&self.total_cost)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct SearchInfo {
    pub agent_radius: f32,
}

pub fn astar<'a, F: Fn(Vec2, Vec2) -> f32>(
    tree: &BSPTree,
    portals: &Portals,
    start: Vec2,
    end: Vec2,
    heuristic: F,
    info: SearchInfo,
    path: &'a mut Option<Path>,
) -> Option<&'a mut Path> {
    let mut open = BinaryHeap::new();
    let start_node = tree.locate(start);
    let end_node = tree.locate(end);

    // // No path if start or end are covered
    // if start_node.covered() || end_node.covered() {
    //     return None;
    // }

    // Find matching start node
    // if let Some(p) = path {
    //     let inc_start = p.iter().position(|p| p.node == start_node.index());

    //     // New end is in the same node as old end
    //     if let (Some(start_idx), Some(last)) = (inc_start, p.last_mut()) {
    //         assert_eq!(last.portal, None);
    //         if last.node == end_node.index() {
    //             last.point = end;
    //             p.drain(0..start_idx);
    //             p[0].point = start;

    //             return path.as_mut();
    //         }
    //     }
    // }

    let start_node = start_node.index();
    let end_node = end_node.index();

    // Information of how a node was reached
    let mut backtraces: SecondaryMap<_, Backtrace> = SecondaryMap::new();
    let start = Backtrace::start(start_node, start, (heuristic)(start, end));

    // Push the fist node
    open.push(start);
    backtraces.insert(start_node, start);

    let mut closed = HashSet::new();

    // Expand the node with the lowest total cost
    while let Some(current) = open.pop() {
        if closed.contains(&current.node) {
            continue;
        }

        // End found
        // Generate backtrace and terminate
        if current.node == end_node {
            let path = path.get_or_insert_with(|| Default::default());

            backtrace(end, current.node, backtraces, path);
            shorten(tree, portals, path, info.agent_radius);
            resolve_clip(portals, path, info.agent_radius);

            return Some(path);
        }

        let end_rel = end - current.point;

        // Add all edges to the open list and update backtraces
        let portals = portals.get(current.node).filter_map(|portal| {
            let face = portal.apply_margin(info.agent_radius);
            if portal.dst() == current.node
                || face.length() < 2.0 * info.agent_radius
                || closed.contains(&portal.dst())
            {
                return None;
            }

            assert_eq!(portal.src(), current.node);

            // Distance to each of the nodes
            let (p1, p2) = face.into_tuple();
            let p1_dist = (heuristic)(p1, end);
            let p2_dist = (heuristic)(p2, end);

            let p = if portal.normal().dot(end_rel) > 0.0 {
                portal.clip(current.point, end, info.agent_radius)
            } else if p1_dist < p2_dist {
                p1
            } else {
                p2
            };

            let backtrace = Backtrace::new(portal, p, &current, (heuristic)(p, end));

            // Update backtrace
            // If the cost to this node is lower than previosuly found,
            // overwrite with the new backtrace.
            match backtraces.entry(backtrace.node).unwrap() {
                Entry::Occupied(mut val) => {
                    if val.get().total_cost > backtrace.total_cost {
                        val.insert(backtrace);
                    } else {
                        return None;
                    }
                }
                Entry::Vacant(entry) => {
                    entry.insert(backtrace);
                }
            }

            Some(backtrace)
        });

        // Add the edges
        open.extend(portals);

        // The current node is now done and won't be revisited
        assert!(closed.insert(current.node))
    }

    None
}

fn backtrace(
    end: Vec2,
    mut current: NodeIndex,
    backtraces: SecondaryMap<NodeIndex, Backtrace>,
    path: &mut Path,
) {
    path.clear();
    path.push(WayPoint::new(end, current, None));
    let mut prev = end;
    loop {
        // Backtrace backwards
        let node = backtraces[current];

        if path.len() < 2 || prev.distance_squared(node.point) > TOLERANCE {
            path.push(WayPoint::new(
                node.point,
                node.node,
                node.portal.as_ref().map(Portal::portal_ref),
            ));
        }

        prev = node.point;

        // Continue up the backtrace
        if let Some(prev) = node.prev {
            current = prev;
        } else {
            break;
        }
    }

    path.reverse();
}

fn resolve_clip(portals: &Portals, path: &mut [WayPoint], margin: f32) {
    if path.len() < 3 {
        return;
    }

    let a = path[0];
    let c = path[2];
    let b = &mut path[1];

    if let Some(portal) = b.portal {
        let portal = portals.from_ref(portal);
        let [p, q] = portal.face.vertices;
        if (b.point().distance(p) < margin + TOLERANCE && portal.adjacent[0])
            || (b.point().distance(q) < margin + TOLERANCE && portal.adjacent[1])
        {
            let normal = portal.normal();
            let a_inc = (a.point - b.point)
                .normalize_or_zero()
                .perp_dot(normal)
                .abs();

            let c_inc = (c.point - b.point)
                .normalize_or_zero()
                .perp_dot(normal)
                .abs();

            b.point += normal * margin * (c_inc - a_inc)
        }
    }

    resolve_clip(portals, &mut path[1..], margin)
}

fn shorten(tree: &BSPTree, portals: &Portals, path: &mut [WayPoint], agent_radius: f32) -> bool {
    if path.len() < 3 {
        return true;
    }

    let a = &path[0];
    let b = &path[1];
    let c = &path[2];

    if let Some(portal) = b.portal {
        let portal = portals.from_ref(portal);
        // c was directly visible from a
        if let Some(p) = portal.try_clip(a.point, c.point, agent_radius) {
            let prev = b.point;

            path[1].point = p;

            // Try to shorten the next strip.
            // If successful, retry shortening for this strip
            if shorten(tree, portals, &mut path[1..], agent_radius)
                && prev.distance_squared(p) > TOLERANCE
            {
                shorten(tree, portals, path, agent_radius);
            }

            return true;
        }
    }

    shorten(tree, portals, &mut path[1..], agent_radius)
}
