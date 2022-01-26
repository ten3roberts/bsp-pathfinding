use std::{
    collections::{BinaryHeap, HashSet},
    ops::{Deref, DerefMut},
};

use glam::Vec2;
use slotmap::{secondary::Entry, SecondaryMap};

use crate::{BSPTree, NodeIndex, Portal, PortalRef, Portals, TOLERANCE};

#[derive(Debug, Clone)]
pub struct Path {
    points: Vec<WayPoint>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WayPoint {
    point: Vec2,
    portal: Option<PortalRef>,
}

impl Deref for WayPoint {
    type Target = Vec2;

    fn deref(&self) -> &Self::Target {
        &self.point
    }
}

impl WayPoint {
    pub fn new(point: Vec2, portal: Option<PortalRef>) -> Self {
        Self { point, portal }
    }
}

impl Path {
    pub fn new(points: Vec<WayPoint>) -> Self {
        Self { points }
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
}

impl Deref for Path {
    type Target = [WayPoint];

    fn deref(&self) -> &Self::Target {
        self.points.deref()
    }
}

impl DerefMut for Path {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.points.deref_mut()
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

pub fn astar<F: Fn(Vec2, Vec2) -> f32>(
    tree: &BSPTree,
    portals: &Portals,
    start: Vec2,
    end: Vec2,
    heuristic: F,
    info: SearchInfo,
) -> Option<Path> {
    let mut open = BinaryHeap::new();
    let start_node = tree.locate(start);
    let end_node = tree.locate(end);

    // No path if start or end a covered
    if start_node.covered() || end_node.covered() {
        return None;
    }

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
            // shorten_path(current.node, &mut backtraces);
            let mut path = backtrace(end, current.node, backtraces, info.agent_radius);
            // shorten(tree, portals, &mut path, info.agent_radius);
            return Some(path);
        }

        let end_rel = end - current.point;

        // Add all edges to the open list and update backtraces
        let portals = portals.get(current.node).filter_map(|portal| {
            if portal.length() < 2.0 * info.agent_radius
                || portal.dst() == current.node
                || closed.contains(&portal.dst())
            {
                return None;
            }
            assert_eq!(portal.src(), current.node);

            let mid = portal.midpoint();

            // Distance to each of the nodes
            let (p1, p2) = portal.apply_margin(info.agent_radius);
            let p1_dist = (heuristic)(p1, end);
            let p2_dist = (heuristic)(p2, end);

            let p = if end_rel.dot(mid - current.point) > 0.0 {
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
    agent_radius: f32,
) -> Path {
    let mut path = Path::new(vec![WayPoint::new(end, None)]);
    loop {
        let node = backtraces[current];

        let p = if let Some(portal) = node.portal {
            node.point - portal.face.normal * agent_radius
        } else {
            node.point
        };

        path.push(WayPoint::new(
            p,
            node.portal.as_ref().map(Portal::portal_ref),
        ));

        // Continue up the backtrace
        if let Some(prev) = node.prev {
            current = prev;
        } else {
            break;
        }
    }

    path.reverse();
    path
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

            if prev.distance_squared(p) < TOLERANCE {
                return false;
            }

            // Try to shorten the next strip.
            // If successful, retry shortening for this strip
            if shorten(tree, portals, &mut path[1..], agent_radius) {
                shorten(tree, portals, path, agent_radius);
            }

            return true;
        }
    }

    shorten(tree, portals, &mut path[1..], agent_radius)
}
