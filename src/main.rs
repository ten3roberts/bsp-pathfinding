use macroquad::prelude::*;
use path_finding::{BSPTree, Shape, World};

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;

fn window_conf() -> Conf {
    Conf {
        window_title: "Runtime path finding using Binary Spatial Partitioning".to_owned(),
        fullscreen: false,
        window_width: WIDTH,
        window_height: HEIGHT,
        ..Default::default()
    }
}
#[macroquad::main(window_conf)]
async fn main() {
    let rect1 = Shape::rect(Vec2::new(200.0, 100.0), Vec2::new(200.0, 300.0));
    let rect2 = Shape::rect(Vec2::new(50.0, 200.0), Vec2::new(275.0, 450.0));
    let tri = Shape::new(&[
        Vec2::new(600.0, 100.0),
        Vec2::new(650.0, 200.0),
        Vec2::new(500.0, 200.0),
    ]);

    let world = World::new(&[rect1, rect2, tri]);

    let tree = BSPTree::new(&world).expect("Existent faces");

    loop {
        clear_background(WHITE);

        tree.draw(10.0);

        world.draw(3.0);

        next_frame().await
    }
}
