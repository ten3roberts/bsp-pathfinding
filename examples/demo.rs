use bsp_path_finding::{
    astar::{astar, Path},
    BSPNode, BSPTree, Face, Portal, Portals, Shape, Side,
};
use macroquad::{color::hsl_to_rgb, prelude::*};

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;

struct Colorscheme {
    background: Color,
    edge: Color,
    shape: Color,
    start: Color,
    path: Color,
    end: Color,
    bsp_plane: fn(usize) -> Color,
}

#[allow(dead_code)]
const DARK_COLORSCHEME: Colorscheme = Colorscheme {
    background: BLACK,
    edge: DARKPURPLE,
    shape: WHITE,
    start: DARKGREEN,
    end: RED,
    path: BLUE,
    bsp_plane: |depth| hsl_to_rgb(depth as f32 / 8.0, 1.0, 0.5),
};

#[allow(dead_code)]
const LIGHT_COLORSCHEME: Colorscheme = Colorscheme {
    background: WHITE,
    edge: DARKPURPLE,
    shape: BLACK,
    start: DARKGREEN,
    end: RED,
    path: BLUE,
    bsp_plane: |depth| hsl_to_rgb(depth as f32 / 8.0, 1.0, 0.5),
};

#[allow(dead_code)]
const GRAYSCALE: Colorscheme = Colorscheme {
    background: WHITE,
    edge: GRAY,
    shape: BLACK,
    end: GRAY,
    path: BLACK,
    bsp_plane: |depth| hsl_to_rgb(1.0, 0.0, (depth as f32 / 8.0).min(0.9)),
    start: BLACK,
};

const COLORSCHEME: Colorscheme = LIGHT_COLORSCHEME;

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
    let rect2 = Shape::rect(Vec2::new(50.0, 200.0), Vec2::new(230.0, 450.0));

    let tri1 = Shape::new(&[
        Vec2::new(600.0, 100.0),
        Vec2::new(650.0, 200.0),
        Vec2::new(500.0, 200.0),
    ]);

    let poly1 = Shape::regular_polygon(5, 80.0, Vec2::new(500.0, 300.0));
    let poly2 = Shape::regular_polygon(3, 50.0, Vec2::new(200.0, 100.0));

    let mut start = Vec2::new(screen_width() / 2.0, screen_height() / 2.0);
    let mut end = Vec2::new(screen_width() / 2.0, screen_height() * 0.2);

    let world = &[rect1, rect2, tri1, poly1, poly2];

    let tree = BSPTree::new(world.iter().flat_map(|val| val.faces())).expect("Existent faces");

    let portals = Portals::from_tree(&tree);
    loop {
        clear_background(COLORSCHEME.background);

        if is_mouse_button_down(MouseButton::Left) {
            let pos = mouse_position().into();

            start = pos;
        }

        if is_mouse_button_down(MouseButton::Right) {
            let pos = mouse_position().into();

            end = pos;
        }

        draw_circle(start.x, start.y, POINT_RADIUS, COLORSCHEME.start);
        draw_circle(end.x, end.y, POINT_RADIUS, COLORSCHEME.end);

        // Find the path
        let path = astar(&tree, &portals, start, end, |cur, end| cur.distance(end));

        if let Some(path) = path {
            path.draw();
        }

        let node = tree.locate(start);
        if !node.covered() {
            node.node().draw();
            portals.get(node.index()).draw()
        }

        tree.draw();
        world.draw();
        // portals.draw();

        for portal in portals.get(tree.locate(start).index) {
            let dst = tree.node(portal.dst).unwrap();
            let src = tree.node(portal.src).unwrap();

            draw_line_dotted(
                portal.vertices[0],
                dst.origin(),
                EDGE_THICKNESS,
                COLORSCHEME.path,
            );
            draw_line_dotted(
                portal.vertices[1],
                src.origin(),
                EDGE_THICKNESS,
                COLORSCHEME.path,
            );
        }

        next_frame().await
    }
}

const THICKNESS: f32 = 3.0;
const POINT_RADIUS: f32 = 10.0;
const VERTEX_RADIUS: f32 = 6.0;
const EDGE_THICKNESS: f32 = 4.0;
const PATH_THICKNESS: f32 = 6.0;
const NORMAL_LEN: f32 = 32.0;
const ARROW_LEN: f32 = 8.0;

trait Draw {
    fn draw(&self);
}

impl Draw for Face {
    fn draw(&self) {
        let a = self.vertices[1];
        let b = self.vertices[0];

        draw_line(a.x, a.y, b.x, b.y, THICKNESS, COLORSCHEME.shape);
        // draw_circle(a.x, a.y, VERTEX_RADIUS, COLORSCHEME.shape);
        // draw_circle(b.x, b.y, VERTEX_RADIUS, COLORSCHEME.shape);
    }
}

impl Draw for Shape {
    fn draw(&self) {
        for face in self.faces() {
            face.draw()
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
    }
}

impl Draw for Portal {
    fn draw(&self) {
        let a = self.vertices[1];
        let b = self.vertices[0];

        draw_line_dotted(a, b, VERTEX_RADIUS, COLORSCHEME.edge);

        if self.sides()[0] == Side::Front {
            draw_circle(
                self.vertices[0].x,
                self.vertices[0].y,
                VERTEX_RADIUS,
                COLORSCHEME.edge,
            );
        }
        if self.sides()[1] == Side::Front {
            draw_circle(
                self.vertices[1].x,
                self.vertices[1].y,
                VERTEX_RADIUS,
                COLORSCHEME.edge,
            );
        }
    }
}

impl Draw for &[Portal] {
    fn draw(&self) {
        for portal in *self {
            portal.draw();
        }
    }
}
impl Draw for Portals {
    fn draw(&self) {
        for portal in self.iter().flatten() {
            portal.draw()
        }
    }
}

impl Draw for Path {
    fn draw(&self) {
        self.windows(2).for_each(|val| {
            let a = val[0];
            let b = val[1];

            draw_circle(a.x, a.y, VERTEX_RADIUS, COLORSCHEME.path);
            draw_circle(b.x, b.y, VERTEX_RADIUS, COLORSCHEME.path);
            draw_line(a.x, a.y, b.x, b.y, PATH_THICKNESS, COLORSCHEME.path);
        })
    }
}
