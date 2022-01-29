use bsp_pathfinding::{astar::SearchInfo, heuristics, NavigationContext, Shape};
use glam::Vec2;

#[test]
fn simple() {
    // Define a simple scene
    let square = Shape::rect(Vec2::new(50.0, 50.0), Vec2::new(0.0, 0.0));
    let left = Shape::rect(Vec2::new(10.0, 200.0), Vec2::new(-200.0, 10.0));
    let right = Shape::rect(Vec2::new(10.0, 200.0), Vec2::new(200.0, 10.0));
    let bottom = Shape::rect(Vec2::new(200.0, 10.0), Vec2::new(10.0, -200.0));
    let top = Shape::rect(Vec2::new(200.0, 10.0), Vec2::new(10.0, 200.0));

    // Create navigational context from the scene
    let nav = NavigationContext::new([square, left, right, top, bottom].iter().flatten());

    // Find a pat
    let start = Vec2::new(-100.0, 0.0);
    let end = Vec2::new(100.0, 30.0);

    let path = nav
        .find_path(start, end, heuristics::euclidiean, SearchInfo::default())
        .expect("Failed to find a path");

    dbg!(&path);

    assert!(path.iter().map(|val| val.point()).eq([
        start,
        Vec2::new(-25.0, 25.0),
        Vec2::new(25.0, 27.0), // Slight shortcut
        end,
    ]));
}
