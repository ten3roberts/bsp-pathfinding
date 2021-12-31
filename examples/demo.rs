use bsp_path_finding::{BSPNode, BSPTree, Shape};
use macroquad::prelude::*;

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

    let world = &[rect1, rect2, tri];

    let tree = BSPTree::new(world.iter().flat_map(|val| val.faces())).expect("Existent faces");

    loop {
        clear_background(WHITE);

        tree.draw();

        world.draw();

        next_frame().await
    }
}

trait Draw {
    fn draw(&self);
}

impl Draw for Shape {
    fn draw(&self) {
        const THICKNESS: f32 = 5.0;
        for face in self.faces() {
            let a = face.vertices[1];
            let b = face.vertices[0];

            draw_line(a.x, a.y, b.x, b.y, THICKNESS, BLACK);
        }
    }
}

impl Draw for [Shape] {
    fn draw(&self) {
        self.iter().for_each(|val| val.draw())
    }
}

impl Draw for BSPTree {
    fn draw(&self) {
        self.descendants().for_each(|(_, val)| val.draw())
    }
}

impl Draw for BSPNode {
    fn draw(&self) {
        const THICKNESS: f32 = 2.0;
        const NORMAL_LEN: f32 = 32.0;
        let normal = self.normal();
        let dir = Vec2::new(normal.y, -normal.x);
        let origin = self.origin();
        let p = origin - dir * 100.0;
        let q = origin + dir * 100.0;

        draw_line(p.x, p.y, q.x, q.y, THICKNESS, PURPLE);

        let end = origin + normal * NORMAL_LEN;
        draw_line(origin.x, origin.y, end.x, end.y, THICKNESS, BLUE);

        for vertex in self.vertices() {
            draw_circle(vertex.x, vertex.y, THICKNESS, BLUE);
        }
    }
}
