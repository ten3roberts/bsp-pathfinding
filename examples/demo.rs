use bsp_pathfinding::{
    astar::{Path, SearchInfo},
    heuristics, BSPNode, BSPTree, Face, NavigationContext, Portal, Portals, Shape,
};
use macroquad::{
    color::hsl_to_rgb,
    prelude::{
        clear_background, draw_circle, draw_line, draw_triangle, get_char_pressed,
        is_mouse_button_down, mouse_position, next_frame, screen_height, screen_width, Color, Conf,
        MouseButton, Vec2, BLACK, BLUE, DARKGREEN, DARKPURPLE, GRAY, RED, WHITE,
    },
};
use rand::{rngs::StdRng, SeedableRng};

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

#[allow(dead_code)]
fn spawn_scene_1() -> Vec<Face> {
    let rect1 = Shape::rect(Vec2::new(200.0, 100.0), Vec2::new(200.0, 300.0));
    let rect2 = Shape::rect(Vec2::new(50.0, 200.0), Vec2::new(230.0, 450.0));

    let tri1 = Shape::new(&[
        Vec2::new(600.0, 100.0),
        Vec2::new(650.0, 200.0),
        Vec2::new(550.0, 200.0),
        Vec2::new(600.0, 100.0),
    ]);

    let poly1 = Shape::regular_polygon(5, 50.0, Vec2::new(500.0, 320.0));
    let poly2 = Shape::regular_polygon(3, 50.0, Vec2::new(200.0, 100.0));

    [rect1, rect2, tri1, poly1, poly2]
        .iter()
        .flatten()
        .collect()
}

#[allow(dead_code)]
fn spawn_scene_2() -> Vec<Face> {
    vec![
        Face::new([Vec2::new(800.0, 30.0), Vec2::new(30.0, 30.0)]),
        Face::new([Vec2::new(200.0, 30.0), Vec2::new(200.0, 400.0)]),
        Face::new([Vec2::new(200.0, 400.0), Vec2::new(30.0, 400.0)]),
        Face::new([Vec2::new(300.0, 300.0), Vec2::new(300.0, 30.0)]),
        Face::new([Vec2::new(500.0, 300.0), Vec2::new(300.0, 300.0)]),
        Face::new([Vec2::new(500.0, 30.0), Vec2::new(500.0, 300.0)]),
        Face::new([Vec2::new(550.0, 300.0), Vec2::new(550.0, 30.0)]),
        Face::new([Vec2::new(750.0, 30.0), Vec2::new(550.0, 300.0)]),
        Face::new([Vec2::new(750.0, 500.0), Vec2::new(750.0, 30.0)]),
        Face::new([Vec2::new(30.0, 500.0), Vec2::new(800.0, 500.0)]),
        // Box in the lower mid
        Face::new([Vec2::new(400.0, 500.0), Vec2::new(400.0, 400.0)]),
        Face::new([Vec2::new(400.0, 400.0), Vec2::new(500.0, 400.0)]),
        Face::new([Vec2::new(500.0, 400.0), Vec2::new(500.0, 500.0)]),
    ]
}

#[macroquad::main(window_conf)]
async fn main() {
    let scenes = [spawn_scene_1(), spawn_scene_2()];
    let mut world = &scenes[0];

    let mut start = Vec2::new(screen_width() / 2.0, screen_height() / 2.0);
    let mut end = Vec2::new(screen_width() / 2.0, screen_height() * 0.2);

    let mut seed = 0;
    let mut nav = None;

    let mut depth = 10;

    loop {
        clear_background(COLORSCHEME.background);

        if is_mouse_button_down(MouseButton::Left) {
            let pos: Vec2 = mouse_position().into();

            if pos.distance(end) < pos.distance(start) {
                end = pos
            } else {
                start = pos;
            }
        }

        if let Some(c) = get_char_pressed() {
            match c {
                '1'..='9' => {
                    let num = c as usize - '1' as usize;
                    if num < scenes.len() {
                        world = &scenes[num];
                        nav = None;
                    }
                }
                'r' => {
                    nav = None;
                    seed += 1
                }
                'R' if seed > 0 => {
                    nav = None;
                    seed -= 1
                }
                'l' => {
                    depth += 1;
                }
                'h' if depth > 0 => {
                    depth -= 1;
                }
                _ => {}
            }
        }

        let nav = nav.get_or_insert_with(|| {
            NavigationContext::new_shuffle(world.iter().cloned(), &mut StdRng::seed_from_u64(seed))
        });

        start += nav.locate(start).unwrap().depth;

        draw_circle(start.x, start.y, POINT_RADIUS, COLORSCHEME.start);
        draw_circle(end.x, end.y, POINT_RADIUS, COLORSCHEME.end);

        let path = nav.find_path(
            start,
            end,
            heuristics::euclidiean,
            SearchInfo {
                agent_radius: POINT_RADIUS,
            },
        );

        let tree = nav.tree().unwrap();
        tree.descendants()
            .filter(|(_, val)| val.depth().max(10) <= depth)
            .for_each(|(_, val)| val.draw());

        // world.draw();

        let portals = nav.portals();

        if depth > 0 {
            portals.draw();

            for portal in portals.get(tree.locate(start).index()) {
                draw_arrow(
                    portal.face().midpoint(),
                    portal.face().midpoint() + portal.normal() * 10.0,
                    COLORSCHEME.edge,
                );
            }
        }

        if let Some(path) = path {
            path.draw();
        }

        next_frame().await
    }
}

const THICKNESS: f32 = 3.0;
const POINT_RADIUS: f32 = 10.0;
const VERTEX_RADIUS: f32 = 6.0;
const EDGE_RADIUS: f32 = 4.0;
const PATH_THICKNESS: f32 = 4.0;
const NORMAL_LEN: f32 = 16.0;
const ARROW_LEN: f32 = 4.0;

trait Draw {
    fn draw(&self);
}

impl Draw for Face {
    fn draw(&self) {
        let a = self.vertices[1];
        let b = self.vertices[0];

        draw_line(a.x, a.y, b.x, b.y, THICKNESS, COLORSCHEME.shape);
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

impl Draw for [Face] {
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
        let color = (COLORSCHEME.bsp_plane)(self.depth());

        for face in self.faces() {
            let origin = face.midpoint();
            let normal = face.normal();
            let end = origin + normal * NORMAL_LEN;
            draw_arrow(origin, end, color);
            face.draw();
        }
        self.faces().iter().for_each(|v| v.draw());
    }
}

fn draw_arrow(start: Vec2, end: Vec2, color: Color) {
    draw_line(start.x, start.y, end.x, end.y, THICKNESS, color);

    let dir = (end - start).normalize();
    let dir_perp = dir.perp();

    // Draw arrow head
    draw_triangle(
        end + dir * ARROW_LEN,
        end + dir_perp * ARROW_LEN,
        end - dir_perp * ARROW_LEN,
        color,
    );
}

impl<'a> Draw for Portal<'a> {
    fn draw(&self) {
        let [a, b] = self.face().vertices;

        draw_line_dotted(a, b, EDGE_RADIUS, COLORSCHEME.edge);
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
