use std::f32::consts::PI;

use nrg_math::Vector3;

use crate::VertexData;

pub fn create_cube(size: Vector3) -> ([VertexData; 8], [u32; 36]) {
    create_cube_from_min_max(-size, size)
}

pub fn create_cube_from_min_max(min: Vector3, max: Vector3) -> ([VertexData; 8], [u32; 36]) {
    let mut vertices = [VertexData::default(); 8];
    vertices[0].pos = [min.x, min.y, min.z].into();
    vertices[1].pos = [max.x, min.y, min.z].into();
    vertices[2].pos = [max.x, max.y, min.z].into();
    vertices[3].pos = [min.x, max.y, min.z].into();
    vertices[4].pos = [min.x, min.y, max.z].into();
    vertices[5].pos = [max.x, min.y, max.z].into();
    vertices[6].pos = [max.x, max.y, max.z].into();
    vertices[7].pos = [min.x, max.y, max.z].into();
    vertices[0].normal = [-1., -1., -1.].into();
    vertices[1].normal = [1., -1., -1.].into();
    vertices[2].normal = [1., 1., -1.].into();
    vertices[3].normal = [-1., 1., -1.].into();
    vertices[4].normal = [-1., -1., 1.].into();
    vertices[5].normal = [1., -1., 1.].into();
    vertices[6].normal = [1., 1., 1.].into();
    vertices[7].normal = [-1., 1., 1.].into();
    vertices[0].tex_coord = [0., 0.].into();
    vertices[1].tex_coord = [1., 0.].into();
    vertices[2].tex_coord = [1., 1.].into();
    vertices[3].tex_coord = [0., 1.].into();
    vertices[4].tex_coord = [0., 0.].into();
    vertices[5].tex_coord = [1., 0.].into();
    vertices[6].tex_coord = [1., 1.].into();
    vertices[7].tex_coord = [0., 1.].into();
    let indices = [
        0, 1, 3, 3, 1, 2, 1, 5, 2, 2, 5, 6, 5, 4, 6, 6, 4, 7, 4, 0, 7, 7, 0, 3, 3, 2, 7, 7, 2, 6,
        4, 5, 0, 0, 5, 1,
    ];
    (vertices, indices)
}

pub fn create_cylinder(
    base_radius: f32,
    top_radius: f32,
    num_slices: u32,
    height: f32,
    num_stack: u32,
) -> (Vec<VertexData>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let angle_step = 2. * PI / num_slices as f32;

    let angle_z = f32::atan2(base_radius - top_radius, height);
    let nx0 = angle_z.cos();
    let ny0 = 0.;
    let nz0 = angle_z.sin();

    //start with sides
    for i in 0..num_stack + 1 {
        let z = -(height * 0.5) + i as f32 / num_stack as f32 * height;
        let radius = base_radius + i as f32 / num_stack as f32 * (top_radius - base_radius);
        let t = 1. - i as f32 / num_stack as f32;

        for j in 0..num_slices + 1 {
            let mut vertex = VertexData::default();
            let angle: f32 = j as f32 * angle_step;

            vertex.pos = [radius * angle.cos(), radius * angle.sin(), z].into();
            vertex.normal = [
                angle.cos() * nx0 - angle.sin() * ny0,
                angle.sin() * nx0 + angle.cos() * ny0,
                nz0,
            ]
            .into();
            vertex.tex_coord = [j as f32 / num_slices as f32, t].into();
            vertices.push(vertex);
        }
    }

    //then base
    let base_vertex_index = vertices.len() as _;
    let mut center_base_vertex = VertexData::default();

    center_base_vertex.pos.z = -height * 0.5;
    center_base_vertex.normal.z = -1.;
    center_base_vertex.tex_coord = [0.5, 0.5].into();
    vertices.push(center_base_vertex);

    for i in 0..num_slices + 1 {
        let mut vertex = VertexData::default();
        let angle: f32 = i as f32 * angle_step;

        vertex.pos = [
            base_radius * angle.cos(),
            base_radius * angle.sin(),
            center_base_vertex.pos.z,
        ]
        .into();
        vertex.normal = center_base_vertex.normal;
        vertex.tex_coord = [-angle.cos() * 0.5 + 0.5, -angle.sin() * 0.5 + 0.5].into(); // flip horizontal
        vertices.push(vertex);
    }

    //then top
    let top_vertex_index = vertices.len() as _;
    let mut center_top_vertex = VertexData::default();

    center_top_vertex.pos.z = height * 0.5;
    center_top_vertex.normal.z = 1.;
    center_top_vertex.tex_coord = [0.5, 0.5].into();
    vertices.push(center_top_vertex);

    for i in 0..num_slices + 1 {
        let mut vertex = VertexData::default();
        let angle: f32 = i as f32 * angle_step;

        vertex.pos = [
            top_radius * angle.cos(),
            top_radius * angle.sin(),
            center_top_vertex.pos.z,
        ]
        .into();
        vertex.normal = center_top_vertex.normal;
        vertex.tex_coord = [angle.cos() * 0.5 + 0.5, -angle.sin() * 0.5 + 0.5].into();
        vertices.push(vertex);
    }

    //fill indices for sides
    for i in 0..num_stack + 1 {
        let mut k1 = i * (num_slices + 1); // bebinning of current stack
        let mut k2 = k1 + num_slices + 1; // beginning of next stack

        for _ in 0..num_slices + 1 {
            indices.push(k1);
            indices.push(k1 + 1);
            indices.push(k2);

            indices.push(k2);
            indices.push(k1 + 1);
            indices.push(k2 + 1);

            k1 += 1;
            k2 += 1;
        }
    }

    // fill indices for base
    let mut k = base_vertex_index + 1;
    for i in 0..num_slices + 1 {
        indices.push(base_vertex_index);
        // last triangle
        if i >= (num_slices - 1) {
            indices.push(base_vertex_index + 1);
        } else {
            indices.push(k + 1);
        }
        indices.push(k);
        k += 1;
    }

    // fill indices for top
    let mut k = top_vertex_index + 1;
    for i in 0..num_slices + 1 {
        indices.push(top_vertex_index);
        indices.push(k);
        // last triangle
        if i >= (num_slices - 1) {
            indices.push(top_vertex_index + 1);
        } else {
            indices.push(k + 1);
        }
        k += 1;
    }

    (vertices, indices)
}
