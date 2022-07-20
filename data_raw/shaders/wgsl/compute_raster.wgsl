#import "utils.wgsl"
#import "common.wgsl"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> indices: Indices;
@group(0) @binding(2)
var<storage, read> vertices: Vertices;
@group(0) @binding(3)
var<storage, read> positions: PositionsAndColors;
@group(0) @binding(4)
var<storage, read> meshes: Meshes;
@group(0) @binding(5)
var<storage, read> meshlets: Meshlets;

@group(1) @binding(0)
var render_target: texture_storage_2d_array<r32sint, read_write>;


fn project(v: vec3<f32>, mvp: mat4x4<f32>, m: mat4x4<f32>) -> vec2<f32> {
    let world_position = m * vec4<f32>(v, 1.0);
    var clip_position = mvp * world_position;
    clip_position.x = ((1. + (clip_position.x / clip_position.w)) / 2.) * constant_data.screen_width;
    clip_position.y = (1. - (1. + (clip_position.y / clip_position.w)) / 2.) * constant_data.screen_height;
    return clip_position.xy;
}

fn is_off_screen(x: i32, y: i32) -> bool {
    if (x < 0 || x > i32(constant_data.screen_width) || y < 0 ||
        y > i32(constant_data.screen_height)) {
        return true;
    }
    return false;
}

fn is_point_off_screen(p: vec2<f32>) -> bool {
    return is_off_screen(i32(p.x), i32(p.y));
}

fn compute_min_max(v1: vec2<f32>, v2: vec2<f32>, v3: vec2<f32>) -> vec4<f32> {
    var min_max = vec4<f32>(0.);
    min_max.x = min(min(v1.x, v2.x), v3.x);
    min_max.y = min(min(v1.y, v2.y), v3.y);
    min_max.z = max(max(v1.x, v2.x), v3.x);
    min_max.w = max(max(v1.y, v2.y), v3.y); 
    return min_max;
}

// From: https://github.com/ssloy/tinyrenderer/wiki/Lesson-2:-Triangle-rasterization-and-back-face-culling
fn barycentric(v1: vec2<f32>, v2: vec2<f32>, v3: vec2<f32>, p: vec2<f32>) -> vec3<f32> {
    let u = cross(
        vec3<f32>(v3.x - v1.x, v2.x - v1.x, v1.x - p.x), 
        vec3<f32>(v3.y - v1.y, v2.y - v1.y, v1.y - p.y)
    );
    if (abs(u.z) < 1.0) {
        return vec3<f32>(-1.0, 1.0, 1.0);
    }
    return vec3<f32>(1.0 - (u.x+u.y)/u.z, u.y/u.z, u.x/u.z); 
}

fn write_pixel(x: i32, y: i32, v: u32) {
    textureStore(render_target, vec2<i32>(x, y), 0, vec4<i32>(i32(v), i32(v), i32(v), i32(v)));
}

fn draw_triangle(v1: vec2<f32>, v2: vec2<f32>, v3: vec2<f32>, index: u32) {
    let min_max = compute_min_max(v1, v2, v3);
    let start_x = i32(min_max.x);
    let start_y = i32(min_max.y);
    let end_x = i32(min_max.z);
    let end_y = i32(min_max.w); 
    for (var x: i32 = start_x; x <= end_x; x = x + 1) {
        for (var y : i32 = start_y; y <= end_y; y = y + 1) {
            let bc = barycentric(v1, v2, v3, vec2<f32>(f32(x), f32(y)));
            if (bc.x < 0.0 || bc.y < 0.0 || bc.z < 0.0) {
                continue;
            }
            write_pixel(x, y, index);
        }
    }
}

fn draw_line(v1: vec3<f32>, v2: vec3<f32>, index: u32) {
    let a = vec2<f32>(v1.x, v1.y);
    let b = vec2<f32>(v2.x, v2.y);

    let dist = i32(distance(a, b));
    for (var i = 0; i < dist; i = i + 1) {
        let x = i32(v1.x + f32(v2.x - v1.x) * (f32(i) / f32(dist)));
        let y = i32(v1.y + f32(v2.y - v1.y) * (f32(i) / f32(dist)));
        write_pixel(x, y, index);
    }
}

fn draw_point(v: vec2<f32>, index: u32) {
    write_pixel(i32(v.x), i32(v.y), index);
}

@compute
@workgroup_size(16, 1, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(local_invocation_index) local_invocation_index: u32, 
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let meshlet_id = global_invocation_id.x;
    let num_meshlets = arrayLength(&meshlets.data);
    if (meshlet_id >= num_meshlets) {
        return;
    }

    let num_meshes = arrayLength(&meshes.data);
    var mesh_id = 0u;
    for(var mi = 0u; mi < num_meshes; mi++) {
        if meshlet_id >= meshes.data[mi].meshlet_offset &&
            meshlet_id < meshes.data[mi].meshlet_offset + meshes.data[mi].meshlet_count {
                mesh_id = mi;
                break;
        }
    }

    let meshlet = &meshlets.data[meshlet_id];
    let mesh = &meshes.data[mesh_id];  
    let mvp = constant_data.proj * constant_data.view;
    let start = u32((*mesh).indices_offset + (*meshlet).indices_offset);
    let end = u32(((*mesh).indices_offset + (*meshlet).indices_offset + (*meshlet).indices_count));  
    let offset = (*mesh).vertex_offset + (*meshlet).vertex_offset;
    for(var i = start; i < end; i = i + 3u) {
        let index = offset + i;
        let i1 = indices.data[index + 0u];
        let i2 = indices.data[index + 1u];
        let i3 = indices.data[index + 2u];

        let v1 = &vertices.data[i1];
        let v2 = &vertices.data[i2];
        let v3 = &vertices.data[i3];
        
        let p1 = project(positions.data[(*v1).position_and_color_offset].xyz, mvp, (*mesh).transform);
        let p2 = project(positions.data[(*v2).position_and_color_offset].xyz, mvp, (*mesh).transform);
        let p3 = project(positions.data[(*v3).position_and_color_offset].xyz, mvp, (*mesh).transform);
        
        if (is_point_off_screen(p1) || is_point_off_screen(p2) || is_point_off_screen(p3)) {
            continue;
        }        
        draw_triangle(p1, p2, p3, meshlet_id);
    }   
}