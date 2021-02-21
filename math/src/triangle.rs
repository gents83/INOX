use crate::{Float, Vector2};

const EPSILON: f64 = 0.001;
const EPSILON_SQUARE: f64 = EPSILON * EPSILON;

pub fn compute_sign<T>(v1: Vector2<T>, v2: Vector2<T>, x: T, y: T) -> T
where
    T: Float,
{
    (x - v2.x) * (v1.y - v2.y) - (v1.x - v2.x) * (y - v2.y)
}

pub fn is_point_in_triangle_naive<T>(
    v1: Vector2<T>,
    v2: Vector2<T>,
    v3: Vector2<T>,
    x: T,
    y: T,
) -> bool
where
    T: Float,
{
    let d1 = compute_sign(v1, v2, x, y);
    let d2 = compute_sign(v2, v3, x, y);
    let d3 = compute_sign(v3, v1, x, y);

    let has_neg = (d1 < T::zero()) || (d2 < T::zero()) || (d3 < T::zero());
    let has_pos = (d1 > T::zero()) || (d2 > T::zero()) || (d3 > T::zero());

    !(has_neg && has_pos)
}

pub fn is_point_in_triangle_boundingbox<T>(
    v1: Vector2<T>,
    v2: Vector2<T>,
    v3: Vector2<T>,
    x: T,
    y: T,
) -> bool
where
    T: Float,
{
    let min_x = T::min(v1.x, T::min(v2.x, v3.x)) - T::from_f64(EPSILON).unwrap();
    let max_x = T::max(v1.x, T::max(v2.x, v3.x)) + T::from_f64(EPSILON).unwrap();
    let min_y = T::min(v1.y, T::min(v2.y, v3.y)) - T::from_f64(EPSILON).unwrap();
    let max_y = T::max(v1.y, T::max(v2.y, v3.y)) + T::from_f64(EPSILON).unwrap();

    !(x < min_x || max_x < x || y < min_y || max_y < y)
}

pub fn compute_distance_square_point_to_segment<T>(v1: Vector2<T>, v2: Vector2<T>, x: T, y: T) -> T
where
    T: Float,
{
    let p1_p2_square_length = (v2.x - v1.x) * (v2.x - v1.x) + (v2.y - v1.y) * (v2.y - v1.y);
    let dot_product =
        ((x - v1.x) * (v2.x - v1.x) + (y - v1.y) * (v2.y - v1.y)) / p1_p2_square_length;
    if dot_product < T::zero() {
        (x - v1.x) * (x - v1.x) + (y - v1.y) * (y - v1.y)
    } else if dot_product <= T::one() {
        let p_p1_square_length = (v1.x - x) * (v1.x - x) + (v1.y - y) * (v1.y - y);
        p_p1_square_length - dot_product * dot_product * p1_p2_square_length
    } else {
        (x - v2.x) * (x - v2.x) + (y - v2.y) * (y - v2.y)
    }
}

pub fn is_point_in_triangle<T>(v1: Vector2<T>, v2: Vector2<T>, v3: Vector2<T>, x: T, y: T) -> bool
where
    T: Float,
{
    let mut result = false;
    if !is_point_in_triangle_boundingbox(v1, v2, v3, x, y) {
        result = false;
    } else if is_point_in_triangle_naive(v1, v2, v3, x, y)
        || compute_distance_square_point_to_segment(v1, v2, x, y)
            <= T::from_f64(EPSILON_SQUARE).unwrap()
        || compute_distance_square_point_to_segment(v2, v3, x, y)
            <= T::from_f64(EPSILON_SQUARE).unwrap()
        || compute_distance_square_point_to_segment(v3, v1, x, y)
            <= T::from_f64(EPSILON_SQUARE).unwrap()
    {
        result = true;
    }
    result
}
