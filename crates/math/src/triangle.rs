use crate::vector::Vector2;

const EPSILON: f32 = 0.001;
const EPSILON_SQUARE: f32 = EPSILON * EPSILON;

pub fn compute_intersection(cp1: Vector2, cp2: Vector2, s: Vector2, e: Vector2) -> Vector2 {
    let dc = Vector2::new(cp1.x - cp2.x, cp1.y - cp2.y);
    let dp = Vector2::new(s.x - e.x, s.y - e.y);
    let n1 = cp1.x * cp2.y - cp1.y * cp2.x;
    let n2 = s.x * e.y - s.y * e.x;
    let n3 = 1. / (dc.x * dp.y - dc.y * dp.x);
    Vector2::new((n1 * dp.x - n2 * dc.x) * n3, (n1 * dp.y - n2 * dc.y) * n3)
}

pub fn is_inside(p: Vector2, cp1: Vector2, cp2: Vector2) -> bool {
    (cp2.x - cp1.x) * (p.y - cp1.y) > (cp2.y - cp1.y) * (p.x - cp1.x)
}

pub fn compute_sign(v1: Vector2, v2: Vector2, x: f32, y: f32) -> f32 {
    (x - v2.x) * (v1.y - v2.y) - (v1.x - v2.x) * (y - v2.y)
}

pub fn is_point_in_triangle_naive(v1: Vector2, v2: Vector2, v3: Vector2, x: f32, y: f32) -> bool {
    let d1 = compute_sign(v1, v2, x, y);
    let d2 = compute_sign(v2, v3, x, y);
    let d3 = compute_sign(v3, v1, x, y);

    let has_neg = (d1 < 0.) || (d2 < 0.) || (d3 < 0.);
    let has_pos = (d1 > 0.) || (d2 > 0.) || (d3 > 0.);

    !(has_neg && has_pos)
}

pub fn is_point_in_triangle_boundingbox(
    v1: Vector2,
    v2: Vector2,
    v3: Vector2,
    x: f32,
    y: f32,
) -> bool {
    let min_x = v1.x.min(v2.x.min(v3.x)) - EPSILON;
    let max_x = v1.x.max(v2.x.max(v3.x)) + EPSILON;
    let min_y = v1.y.min(v2.y.min(v3.y)) - EPSILON;
    let max_y = v1.y.max(v2.y.max(v3.y)) + EPSILON;

    !(x < min_x || max_x < x || y < min_y || max_y < y)
}

pub fn compute_distance_square_point_to_segment(v1: Vector2, v2: Vector2, x: f32, y: f32) -> f32 {
    let p1_p2_square_length = (v2.x - v1.x) * (v2.x - v1.x) + (v2.y - v1.y) * (v2.y - v1.y);
    let dot_product =
        ((x - v1.x) * (v2.x - v1.x) + (y - v1.y) * (v2.y - v1.y)) / p1_p2_square_length;
    if dot_product < 0. {
        (x - v1.x) * (x - v1.x) + (y - v1.y) * (y - v1.y)
    } else if dot_product <= 1. {
        let p_p1_square_length = (v1.x - x) * (v1.x - x) + (v1.y - y) * (v1.y - y);
        p_p1_square_length - dot_product * dot_product * p1_p2_square_length
    } else {
        (x - v2.x) * (x - v2.x) + (y - v2.y) * (y - v2.y)
    }
}

pub fn is_point_in_triangle(v1: Vector2, v2: Vector2, v3: Vector2, x: f32, y: f32) -> bool {
    let mut result = false;
    if !is_point_in_triangle_boundingbox(v1, v2, v3, x, y) {
        result = false;
    } else if is_point_in_triangle_naive(v1, v2, v3, x, y)
        || compute_distance_square_point_to_segment(v1, v2, x, y) <= EPSILON_SQUARE
        || compute_distance_square_point_to_segment(v2, v3, x, y) <= EPSILON_SQUARE
        || compute_distance_square_point_to_segment(v3, v1, x, y) <= EPSILON_SQUARE
    {
        result = true;
    }
    result
}
