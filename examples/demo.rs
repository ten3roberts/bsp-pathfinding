use bsp_path_finding::{BSPNode, BSPTree, Edges, Face, Shape};
use macroquad::{color::hsl_to_rgb, prelude::*};

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;

struct Colorscheme {
    background: Color,
    edge: Color,
    shape: Color,
    bsp_plane: fn(usize) -> Color,
}

#[allow(dead_code)]
const DARK_COLORSCHEME: Colorscheme = Colorscheme {
    background: BLACK,
    edge: DARKPURPLE,
    shape: WHITE,
    bsp_plane: |depth| hsl_to_rgb(depth as f32 / 6.0, 1.0, 0.5),
};

#[allow(dead_code)]
const LIGHT_COLORSCHEME: Colorscheme = Colorscheme {
    background: WHITE,
    edge: DARKPURPLE,
    shape: BLACK,
    bsp_plane: |depth| hsl_to_rgb(depth as f32 / 6.0, 1.0, 0.5),
};

#[allow(dead_code)]
const GRAYSCALE: Colorscheme = Colorscheme {
    background: WHITE,
    edge: GRAY,
    shape: BLACK,
    bsp_plane: |depth| hsl_to_rgb(1.0, 0.0, depth as f32 / 6.0),
};

const COLORSCHEME: Colorscheme = GRAYSCALE;

/// Draws a dotted line, performance isn't great due to many draw calls. This is
/// acceptable as it is only for visualization.
fn draw_line_dotted(p: Vec2, q: Vec2, thickness: f32, color: Color) {
    let step = thickness * 2.0;
    let radius = thickness / 2.0;

    let steps = (p.distance(q) / step).floor() as usize;

    let dir = (q - p).normalize();

    (0..=steps).for_each(|val| {
        let t = p + dir * val as f32 * step;

        draw_circle(t.x, t.y, radius, color);
    })
}

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

    let tri1 = Shape::new(&[
        Vec2::new(600.0, 100.0),
        Vec2::new(650.0, 200.0),
        Vec2::new(500.0, 200.0),
    ]);

    let poly1 = Shape::regular_polygon(5, 80.0, Vec2::new(500.0, 400.0));
    let poly2 = Shape::regular_polygon(3, 80.0, Vec2::new(200.0, 100.0));

    let corners = [
        Vec2::new(0.0, screen_height() - 0.0),
        Vec2::new(screen_width() - 0.0, screen_height() - 0.0),
        Vec2::new(screen_width() - 0.0, 0.0),
        Vec2::new(0.0, 0.0),
    ];

    // Encapsulate the scene to allow for calculation of bsp bounds for
    // visualization
    let bounds = [
        Face::new([corners[0], corners[1]]),
        Face::new([corners[1], corners[2]]),
        Face::new([corners[2], corners[3]]),
        Face::new([corners[3], corners[0]]),
    ];

    let world = &[rect1, rect2, tri1, poly1, poly2];

    let tree = BSPTree::with_bounds(world.iter().flat_map(|val| val.faces()), &bounds)
        .expect("Existent faces");

    let edges = tree.generate_edges();

    loop {
        clear_background(COLORSCHEME.background);

        tree.draw();
        edges.draw();
        world.draw();

        next_frame().await
    }
}

const THICKNESS: f32 = 3.0;
const EDGE_THICKNESS: f32 = 4.0;
const NORMAL_LEN: f32 = 32.0;
const ARROW_LEN: f32 = 8.0;

trait Draw {
    fn draw(&self);
}

impl Draw for Shape {
    fn draw(&self) {
        for face in self.faces() {
            let a = face.vertices[1];
            let b = face.vertices[0];

            draw_line(a.x, a.y, b.x, b.y, THICKNESS, COLORSCHEME.shape);
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
        let normal = self.normal();
        let origin = self.origin();

        let color = (COLORSCHEME.bsp_plane)(self.depth());

        let end = origin + normal * NORMAL_LEN;
        let normal_perp = Vec2::new(normal.y, -normal.x);

        draw_line(origin.x, origin.y, end.x, end.y, THICKNESS, color);

        // Draw arrow head
        draw_triangle(
            end + normal * ARROW_LEN,
            end + normal_perp * ARROW_LEN,
            end - normal_perp * ARROW_LEN,
            color,
        );

        if let Some(bounds) = self.bounds() {
            let p = bounds[0];
            let q = bounds[1];

            draw_line(p.x, p.y, q.x, q.y, THICKNESS, color);
        }

        // for p in self.vertices() {
        //     draw_circle(p.x, p.y, 16.0 - self.depth() as f32 * 2.0, color);
        // }
    }
}

impl Draw for Edges {
    fn draw(&self) {
        for edge in self.iter().flatten() {
            let [p, q] = edge.origins();
            draw_line_dotted(p, q, EDGE_THICKNESS, COLORSCHEME.edge);
        }
    }
}
