use glam::Vec2;

pub fn euclidiean(start: Vec2, end: Vec2) -> f32 {
    (end - start).length()
}

pub fn manhattan(start: Vec2, end: Vec2) -> f32 {
    let x = end.x - start.x;
    let y = end.y - start.y;
    x.abs() + y.abs()
}
