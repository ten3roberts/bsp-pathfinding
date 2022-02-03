use std::f32::consts::PI;

use bsp_pathfinding::*;
use glam::{Mat3, Vec2};

#[test]
fn face() {
    let f = Face::new([-Vec2::X, Vec2::X]);
    let trans = Mat3::from_scale_angle_translation(Vec2::splat(2.0), PI, Vec2::new(0.0, 1.0));

    let f = f.transform(trans);
    assert!(f.vertices[0].distance(Vec2::X * 2.0 + Vec2::new(0.0, 1.0)) < 0.01);
    assert!(f.vertices[1].distance(-Vec2::X * 2.0 + Vec2::new(0.0, 1.0)) < 0.01);
    assert!(f.normal().distance(Vec2::Y) < 0.01);
}

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

    // Find a path
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
