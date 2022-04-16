use std::f32::consts::PI;

use inox_math::{Mat4Ops, MatBase, Matrix4, VecBase, Vector3, Vector4};

use crate::{PbrVertexData, MAX_TEXTURE_COORDS_SETS};

pub fn create_cube(size: Vector3) -> ([PbrVertexData; 8], [u32; 36]) {
    create_cube_from_min_max(-size, size)
}

pub fn create_cube_from_min_max(min: Vector3, max: Vector3) -> ([PbrVertexData; 8], [u32; 36]) {
    let mut vertices = [PbrVertexData::default(); 8];
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
    vertices[0].tex_coord = [[0., 0.].into(); MAX_TEXTURE_COORDS_SETS];
    vertices[1].tex_coord = [[1., 0.].into(); MAX_TEXTURE_COORDS_SETS];
    vertices[2].tex_coord = [[1., 1.].into(); MAX_TEXTURE_COORDS_SETS];
    vertices[3].tex_coord = [[0., 1.].into(); MAX_TEXTURE_COORDS_SETS];
    vertices[4].tex_coord = [[0., 0.].into(); MAX_TEXTURE_COORDS_SETS];
    vertices[5].tex_coord = [[1., 0.].into(); MAX_TEXTURE_COORDS_SETS];
    vertices[6].tex_coord = [[1., 1.].into(); MAX_TEXTURE_COORDS_SETS];
    vertices[7].tex_coord = [[0., 1.].into(); MAX_TEXTURE_COORDS_SETS];
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
) -> (Vec<PbrVertexData>, Vec<u32>) {
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
            let mut vertex = PbrVertexData::default();
            let angle: f32 = j as f32 * angle_step;

            vertex.pos = [radius * angle.cos(), radius * angle.sin(), z].into();
            vertex.normal = [
                angle.cos() * nx0 - angle.sin() * ny0,
                angle.sin() * nx0 + angle.cos() * ny0,
                nz0,
            ]
            .into();
            vertex.tex_coord = [[j as f32 / num_slices as f32, t].into(); MAX_TEXTURE_COORDS_SETS];
            vertices.push(vertex);
        }
    }

    //then base
    let base_vertex_index = vertices.len() as _;
    let mut center_base_vertex = PbrVertexData::default();

    center_base_vertex.pos.z = -height * 0.5;
    center_base_vertex.normal.z = -1.;
    center_base_vertex.tex_coord = [[0.5, 0.5].into(); MAX_TEXTURE_COORDS_SETS];
    vertices.push(center_base_vertex);

    for i in 0..num_slices + 1 {
        let mut vertex = PbrVertexData::default();
        let angle: f32 = i as f32 * angle_step;

        vertex.pos = [
            base_radius * angle.cos(),
            base_radius * angle.sin(),
            center_base_vertex.pos.z,
        ]
        .into();
        vertex.normal = center_base_vertex.normal;
        vertex.tex_coord =
            [[-angle.cos() * 0.5 + 0.5, -angle.sin() * 0.5 + 0.5].into(); MAX_TEXTURE_COORDS_SETS]; // flip horizontal
        vertices.push(vertex);
    }

    //then top
    let top_vertex_index = vertices.len() as _;
    let mut center_top_vertex = PbrVertexData::default();

    center_top_vertex.pos.z = height * 0.5;
    center_top_vertex.normal.z = 1.;
    center_top_vertex.tex_coord = [[0.5, 0.5].into(); MAX_TEXTURE_COORDS_SETS];
    vertices.push(center_top_vertex);

    for i in 0..num_slices + 1 {
        let mut vertex = PbrVertexData::default();
        let angle: f32 = i as f32 * angle_step;

        vertex.pos = [
            top_radius * angle.cos(),
            top_radius * angle.sin(),
            center_top_vertex.pos.z,
        ]
        .into();
        vertex.normal = center_top_vertex.normal;
        vertex.tex_coord =
            [[angle.cos() * 0.5 + 0.5, -angle.sin() * 0.5 + 0.5].into(); MAX_TEXTURE_COORDS_SETS];
        vertices.push(vertex);
    }

    //fill indices for sides
    for i in 0..num_stack + 1 {
        let mut k1 = i * (num_slices); // bebinning of current stack
        let mut k2 = k1 + num_slices; // beginning of next stack

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

pub fn create_sphere(
    position: Vector3,
    radius: f32,
    num_slices: u32,
    num_stack: u32,
    color: Vector4,
) -> (Vec<PbrVertexData>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let slice_step = 2. * PI / num_slices as f32;
    let stack_step = PI / num_stack as f32;
    let inv = 1. / radius;

    for i in 0..num_stack + 1 {
        let stack_angle = PI / 2. - i as f32 * stack_step; // from pi/2 to -pi/2
        let xy = radius * stack_angle.cos();
        let z = radius * stack_angle.sin();

        for j in 0..num_slices + 1 {
            let mut vertex = PbrVertexData::default();
            let slice_angle = j as f32 * slice_step; // from 0 to 2pi
            vertex.pos = position;
            vertex.pos += [xy * slice_angle.cos(), xy * slice_angle.sin(), z].into();
            vertex.normal = vertex.pos * inv;
            vertex.tex_coord = [[j as f32 / num_slices as f32, i as f32 / num_stack as f32].into();
                MAX_TEXTURE_COORDS_SETS];
            vertex.color = color;
            vertices.push(vertex);
        }
    }

    for i in 0..num_stack {
        let mut k1 = i * (num_slices + 1); // beginning of current stack
        let mut k2 = k1 + num_slices + 1; // beginning of next stack

        for _ in 0..num_slices {
            // 2 triangles per sector excluding 1st and last stacks
            if i != 0 {
                indices.push(k1);
                indices.push(k2);
                indices.push(k1 + 1);
            }
            if i != (num_stack - 1) {
                indices.push(k1 + 1);
                indices.push(k2);
                indices.push(k2 + 1);
            }
            k1 += 1;
            k2 += 1;
        }
    }

    (vertices, indices)
}

pub fn create_arrow(position: Vector3, direction: Vector3) -> (Vec<PbrVertexData>, Vec<u32>) {
    let mut shape_vertices = Vec::new();
    let mut shape_indices = Vec::new();

    let height = direction.length();

    let (mut vertices, mut indices) = create_cylinder(0.25, 0.25, 16, height, 1);
    vertices.iter_mut().for_each(|v| {
        v.pos.z += height * 0.5;
    });
    indices
        .iter_mut()
        .for_each(|i| *i += shape_vertices.len() as u32);
    shape_vertices.append(&mut vertices);
    shape_indices.append(&mut indices);

    let (mut vertices, mut indices) = create_cylinder(0.5, 0., 16, 2.5, 1);
    vertices.iter_mut().for_each(|v| {
        v.pos.z += height;
    });
    indices
        .iter_mut()
        .for_each(|i| *i += shape_vertices.len() as u32);
    shape_vertices.append(&mut vertices);
    shape_indices.append(&mut indices);

    let mut matrix = Matrix4::default_identity();
    matrix.look_towards(direction);
    shape_vertices.iter_mut().for_each(|v| {
        v.pos = position + matrix.transform(v.pos);
    });

    (shape_vertices, shape_indices)
}

pub fn create_line(start: Vector3, end: Vector3, color: Vector4) -> ([PbrVertexData; 3], [u32; 3]) {
    let direction = (end - start).normalized();
    let mut vertices = [PbrVertexData::default(); 3];
    vertices[0].pos = [start.x, start.y, start.z].into();
    vertices[1].pos = [start.x, start.y, start.z].into();
    vertices[2].pos = [end.x, end.y, end.z].into();

    vertices[0].normal = -direction;
    vertices[1].normal = -direction;
    vertices[2].normal = direction;

    vertices[2].tex_coord = [[1., 1.].into(); MAX_TEXTURE_COORDS_SETS];

    vertices[0].color = color;
    vertices[1].color = color;
    vertices[2].color = color;

    let indices = [0, 1, 2];

    (vertices, indices)
}

pub fn create_hammer(position: Vector3, direction: Vector3) -> (Vec<PbrVertexData>, Vec<u32>) {
    let mut shape_vertices = Vec::new();
    let mut shape_indices = Vec::new();

    let height = direction.length();

    let (mut vertices, mut indices) = create_cylinder(0.25, 0.25, 16, height, 1);
    vertices.iter_mut().for_each(|v| {
        v.pos.z += height * 0.5;
    });
    indices
        .iter_mut()
        .for_each(|i| *i += shape_vertices.len() as u32);
    shape_vertices.append(&mut vertices);
    shape_indices.append(&mut indices);

    let (mut vertices, mut indices) =
        create_cube_from_min_max(Vector3::new(-0.5, -0.5, -0.5), Vector3::new(0.5, 0.5, 0.5));

    vertices.iter_mut().for_each(|v| {
        v.pos.z += height;
    });
    indices
        .iter_mut()
        .for_each(|i| *i += shape_vertices.len() as u32);
    shape_vertices.append(&mut vertices.to_vec());
    shape_indices.append(&mut indices.to_vec());

    let mut matrix = Matrix4::default_identity();
    matrix.look_towards(direction);
    shape_vertices.iter_mut().for_each(|v| {
        v.pos = position + matrix.transform(v.pos);
    });

    (shape_vertices, shape_indices)
}

pub fn create_torus(
    position: Vector3,
    main_radius: f32,
    tube_radius: f32,
    num_main_slices: u32,
    num_tube_slices: u32,
    direction: Vector3,
) -> (Vec<PbrVertexData>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let main_step = 2. * PI / num_main_slices as f32;
    let tube_step = 2. * PI / num_tube_slices as f32;

    for i in 0..num_main_slices + 1 {
        let main_angle = i as f32 * main_step;
        for j in 0..num_tube_slices + 1 {
            let mut vertex = PbrVertexData::default();
            let tube_angle = j as f32 * tube_step;
            vertex.pos = [
                (main_radius + tube_radius * tube_angle.cos()) * main_angle.cos(),
                (main_radius + tube_radius * tube_angle.cos()) * main_angle.sin(),
                tube_radius * tube_angle.sin(),
            ]
            .into();
            vertex.normal = [
                main_angle.cos() * tube_angle.cos(),
                main_angle.sin() * tube_angle.cos(),
                tube_angle.sin(),
            ]
            .into();
            vertex.tex_coord = [[
                j as f32 / num_tube_slices as f32,
                i as f32 * (2. / num_main_slices as f32),
            ]
            .into(); MAX_TEXTURE_COORDS_SETS];
            vertices.push(vertex);
        }
    }
    for i in 0..num_main_slices + 1 {
        let mut k1 = i * (num_main_slices); // bebinning of current stack
        let mut k2 = k1 + num_main_slices; // beginning of next stack

        for _ in 0..num_tube_slices + 1 {
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

    let mut matrix = Matrix4::default_identity();
    matrix.look_towards(direction);
    vertices.iter_mut().for_each(|v| {
        v.pos = position + matrix.transform(v.pos);
    });

    (vertices, indices)
}
