use macroquad::prelude::Vec2;

/// Returns the intersection points between two lines
pub fn line_intersect(a: (Vec2, Vec2), b: (Vec2, Vec2)) -> Vec2 {
    let a_dir = (a.1 - a.0).normalize_or_zero();
    let b_dir = (b.1 - b.0).normalize_or_zero();
    let rel = b.0 - a.0;
    let dot = a_dir.perp_dot(b_dir);
    // project rel onto plane defined by a
    let length = rel.perp_dot(a_dir);

    let p = b.0 + b_dir * (length / dot);

    p
}
