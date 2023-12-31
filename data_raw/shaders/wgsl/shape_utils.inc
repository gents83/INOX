
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

fn draw_line_3d(image_pixel: vec2<u32>, image_size: vec2<u32>, start: vec3<f32>, end: vec3<f32>, previous_color: vec3<f32>, line_color: vec3<f32>, size: f32) -> vec3<f32>
{    
    let origin = pixel_to_world(image_pixel, image_size, -1.);
    let far = pixel_to_world(image_pixel, image_size, 1.);
    let direction = normalize(far - origin);

    let line_dir = normalize(end-start);
    let v1 = start-origin;
	let d0 = dot(direction, line_dir);
    let d1 = dot(direction, v1);
    let d2 = dot(line_dir, v1);
	var len = (d0*d1-d2)/(1.-d0*d0);
    len = clamp(len, 0., length(end-start));
    let p = start+line_dir*len;
    let value = length(cross(p-origin, direction));
    return mix(line_color, previous_color, 1.-size/value);
}