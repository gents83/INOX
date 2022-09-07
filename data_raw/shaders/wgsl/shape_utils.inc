
fn draw_line(uv: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>, width: f32) -> f32 {
    let d = p2 - p1;
    let t = clamp(dot(d,uv-p1) / dot(d,d), 0., 1.);
    let proj = p1 + d * t;
    return 1. - smoothstep(0., width, length(uv - proj));
}

fn draw_circle(uv: vec2<f32>, center: vec2<f32>, radius: f32, width: f32) -> f32 {
    let p = uv - center;
    let d = sqrt(dot(p,p));
    return 1. - smoothstep(0., width, abs(radius-d));
}