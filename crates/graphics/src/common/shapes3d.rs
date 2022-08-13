use std::f32::consts::PI;

use inox_math::{Mat4Ops, MatBase, Matrix4, VecBaseFloat, Vector2, Vector3, Vector4};

use crate::{MeshData, MeshletData};

pub fn create_cube(size: Vector3, color: Vector4) -> MeshData {
    create_cube_from_min_max(-size, size, color)
}

pub fn create_cube_from_min_max(min: Vector3, max: Vector3, color: Vector4) -> MeshData {
    let mut mesh_data = MeshData::default();
    mesh_data.add_vertex_pos_color_normal_uv(
        Vector3::new(min.x, min.y, min.z),
        color,
        Vector3::new(-1., -1., -1.),
        Vector2::new(0., 0.),
    );
    mesh_data.add_vertex_pos_color_normal_uv(
        Vector3::new(max.x, min.y, min.z),
        color,
        Vector3::new(1., -1., -1.),
        Vector2::new(1., 0.),
    );
    mesh_data.add_vertex_pos_color_normal_uv(
        Vector3::new(max.x, max.y, min.z),
        color,
        Vector3::new(1., 1., -1.),
        Vector2::new(1., 1.),
    );
    mesh_data.add_vertex_pos_color_normal_uv(
        Vector3::new(min.x, max.y, min.z),
        color,
        Vector3::new(-1., 1., -1.),
        Vector2::new(0., 1.),
    );
    mesh_data.add_vertex_pos_color_normal_uv(
        Vector3::new(min.x, min.y, max.z),
        color,
        Vector3::new(-1., -1., 1.),
        Vector2::new(0., 0.),
    );
    mesh_data.add_vertex_pos_color_normal_uv(
        Vector3::new(max.x, min.y, max.z),
        color,
        Vector3::new(1., -1., 1.),
        Vector2::new(1., 0.),
    );
    mesh_data.add_vertex_pos_color_normal_uv(
        Vector3::new(max.x, max.y, max.z),
        color,
        Vector3::new(1., 1., 1.),
        Vector2::new(1., 1.),
    );
    mesh_data.add_vertex_pos_color_normal_uv(
        Vector3::new(min.x, max.y, max.z),
        color,
        Vector3::new(-1., 1., 1.),
        Vector2::new(0., 1.),
    );
    mesh_data.indices = [
        0, 1, 3, 3, 1, 2, 1, 5, 2, 2, 5, 6, 5, 4, 6, 6, 4, 7, 4, 0, 7, 7, 0, 3, 3, 2, 7, 7, 2, 6,
        4, 5, 0, 0, 5, 1,
    ]
    .to_vec();
    let meshlet = MeshletData {
        vertices_count: mesh_data.vertex_count() as _,
        indices_count: mesh_data.index_count() as _,
        ..Default::default()
    };
    mesh_data.meshlets.push(meshlet);
    mesh_data
}

pub fn create_cylinder(
    base_radius: f32,
    top_radius: f32,
    num_slices: u32,
    height: f32,
    num_stack: u32,
    color: Vector4,
) -> MeshData {
    let mut mesh_data = MeshData::default();

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
            let angle: f32 = j as f32 * angle_step;

            mesh_data.add_vertex_pos_color_normal_uv(
                [radius * angle.cos(), radius * angle.sin(), z].into(),
                color,
                [
                    angle.cos() * nx0 - angle.sin() * ny0,
                    angle.sin() * nx0 + angle.cos() * ny0,
                    nz0,
                ]
                .into(),
                [j as f32 / num_slices as f32, t].into(),
            );
        }
    }

    //then base
    let base_vertex_index = mesh_data.vertex_count() as _;
    mesh_data.add_vertex_pos_color_normal_uv(
        [0., 0., -height * 0.5].into(),
        color,
        [0., 0., -1.].into(),
        [0.5, 0.5].into(),
    );

    for i in 0..num_slices + 1 {
        let angle: f32 = i as f32 * angle_step;

        mesh_data.add_vertex_pos_color_normal_uv(
            [
                base_radius * angle.cos(),
                base_radius * angle.sin(),
                -height * 0.5,
            ]
            .into(),
            color,
            [0., 0., -1.].into(),
            [-angle.cos() * 0.5 + 0.5, -angle.sin() * 0.5 + 0.5].into(), // flip horizontal
        );
    }

    //then top
    let top_vertex_index = mesh_data.vertex_count() as _;

    mesh_data.add_vertex_pos_color_normal_uv(
        [0., 0., height * 0.5].into(),
        color,
        [0., 0., 1.].into(),
        [0.5, 0.5].into(),
    );

    for i in 0..num_slices + 1 {
        let angle: f32 = i as f32 * angle_step;

        mesh_data.add_vertex_pos_color_normal_uv(
            [
                top_radius * angle.cos(),
                top_radius * angle.sin(),
                height * 0.5,
            ]
            .into(),
            color,
            [0., 0., 1.].into(),
            [angle.cos() * 0.5 + 0.5, -angle.sin() * 0.5 + 0.5].into(),
        );
    }

    //fill indices for sides
    for i in 0..num_stack + 1 {
        let mut k1 = i * (num_slices); // bebinning of current stack
        let mut k2 = k1 + num_slices; // beginning of next stack

        for _ in 0..num_slices + 1 {
            mesh_data.indices.push(k1);
            mesh_data.indices.push(k1 + 1);
            mesh_data.indices.push(k2);

            mesh_data.indices.push(k2);
            mesh_data.indices.push(k1 + 1);
            mesh_data.indices.push(k2 + 1);

            k1 += 1;
            k2 += 1;
        }
    }

    // fill indices for base
    let mut k = base_vertex_index + 1;
    for i in 0..num_slices + 1 {
        mesh_data.indices.push(base_vertex_index);
        // last triangle
        if i >= (num_slices - 1) {
            mesh_data.indices.push(base_vertex_index + 1);
        } else {
            mesh_data.indices.push(k + 1);
        }
        mesh_data.indices.push(k);
        k += 1;
    }

    // fill indices for top
    let mut k = top_vertex_index + 1;
    for i in 0..num_slices + 1 {
        mesh_data.indices.push(top_vertex_index);
        mesh_data.indices.push(k);
        // last triangle
        if i >= (num_slices - 1) {
            mesh_data.indices.push(top_vertex_index + 1);
        } else {
            mesh_data.indices.push(k + 1);
        }
        k += 1;
    }

    let meshlet = MeshletData {
        vertices_count: mesh_data.vertex_count() as _,
        indices_count: mesh_data.index_count() as _,
        ..Default::default()
    };
    mesh_data.meshlets.push(meshlet);
    mesh_data
}

pub fn create_sphere(
    position: Vector3,
    radius: f32,
    num_slices: u32,
    num_stack: u32,
    color: Vector4,
) -> MeshData {
    let mut mesh_data = MeshData::default();

    let slice_step = 2. * PI / num_slices as f32;
    let stack_step = PI / num_stack as f32;
    let inv = 1. / radius;

    for i in 0..num_stack + 1 {
        let stack_angle = PI / 2. - i as f32 * stack_step; // from pi/2 to -pi/2
        let xy = radius * stack_angle.cos();
        let z = radius * stack_angle.sin();

        for j in 0..num_slices + 1 {
            let slice_angle = j as f32 * slice_step; // from 0 to 2pi
            let mut pos = position;
            pos += [xy * slice_angle.cos(), xy * slice_angle.sin(), z].into();

            mesh_data.add_vertex_pos_color_normal_uv(
                pos,
                color,
                pos * inv,
                [j as f32 / num_slices as f32, i as f32 / num_stack as f32].into(),
            );
        }
    }

    for i in 0..num_stack {
        let mut k1 = i * (num_slices + 1); // beginning of current stack
        let mut k2 = k1 + num_slices + 1; // beginning of next stack

        for _ in 0..num_slices {
            // 2 triangles per sector excluding 1st and last stacks
            if i != 0 {
                mesh_data.indices.push(k1);
                mesh_data.indices.push(k2);
                mesh_data.indices.push(k1 + 1);
            }
            if i != (num_stack - 1) {
                mesh_data.indices.push(k1 + 1);
                mesh_data.indices.push(k2);
                mesh_data.indices.push(k2 + 1);
            }
            k1 += 1;
            k2 += 1;
        }
    }
    let meshlet = MeshletData {
        vertices_count: mesh_data.vertex_count() as _,
        indices_count: mesh_data.index_count() as _,
        ..Default::default()
    };
    mesh_data.meshlets.push(meshlet);
    mesh_data
}

pub fn create_arrow(position: Vector3, direction: Vector3, color: Vector4) -> MeshData {
    let mut shape_mesh_data = MeshData::default();

    let height = direction.length();

    let mut cylinder_mesh_data = create_cylinder(0.25, 0.25, 16, height, 1, color);
    cylinder_mesh_data.aabb_min.y += height * 0.5;
    cylinder_mesh_data.aabb_max.y += height * 0.5;
    shape_mesh_data.append_mesh_data(cylinder_mesh_data, false);

    let mut tip_mesh_data = create_cylinder(0.5, 0., 16, 2.5, 1, color);
    tip_mesh_data.aabb_min.y += height;
    tip_mesh_data.aabb_max.y += height;
    shape_mesh_data.append_mesh_data(tip_mesh_data, false);

    let mut matrix = Matrix4::default_identity();
    matrix.look_towards(direction);
    shape_mesh_data.aabb_min = position + matrix.transform(shape_mesh_data.aabb_min);
    shape_mesh_data.aabb_max = position + matrix.transform(shape_mesh_data.aabb_max);
    shape_mesh_data
}

pub fn create_line(start: Vector3, end: Vector3, color: Vector4) -> MeshData {
    let mut mesh_data = MeshData::default();
    mesh_data.add_vertex_pos_color([start.x, start.y, start.z].into(), color);
    mesh_data.add_vertex_pos_color([start.x, start.y, start.z].into(), color);
    mesh_data.add_vertex_pos_color([end.x, end.y, end.z].into(), color);

    mesh_data.indices = [0, 1, 2].to_vec();

    let meshlet = MeshletData {
        vertices_count: mesh_data.vertex_count() as _,
        indices_count: mesh_data.index_count() as _,
        ..Default::default()
    };
    mesh_data.meshlets.push(meshlet);
    mesh_data
}

pub fn create_circumference(
    position: Vector3,
    radius: f32,
    num_slices: u32,
    color: Vector4,
) -> MeshData {
    let mut mesh_data = MeshData::default();

    let slice_step = 2. * PI / num_slices as f32;

    for i in 0..num_slices + 1 {
        let slice_angle_start = i as f32 * slice_step; // from 0 to 2pi
        let slice_angle_end = ((i + 1) % num_slices) as f32 * slice_step; // from 0 to 2pi
        let mut pos1 = position;
        let mut pos2 = position;
        pos1 += [
            radius * slice_angle_start.cos(),
            radius * slice_angle_start.sin(),
            0.,
        ]
        .into();
        pos2 += [
            radius * slice_angle_end.cos(),
            radius * slice_angle_end.sin(),
            0.,
        ]
        .into();

        let m = create_line(pos1, pos2, color);
        mesh_data.append_mesh_data(m, false);
    }
    mesh_data
}

pub fn create_circle(position: Vector3, radius: f32, num_slices: u32, color: Vector4) -> MeshData {
    let mut mesh_data = MeshData::default();

    let slice_step = 2. * PI / num_slices as f32;
    let inv = 1. / radius;

    mesh_data.add_vertex_pos_color_normal_uv(position, color, position * inv, [0.5, 0.5].into());

    for j in 0..num_slices + 1 {
        let slice_angle = j as f32 * slice_step; // from 0 to 2pi
        let mut pos = position;
        pos += [radius * slice_angle.cos(), radius * slice_angle.sin(), 0.].into();

        mesh_data.add_vertex_pos_color_normal_uv(
            pos,
            color,
            pos * inv,
            [radius * slice_angle.cos(), radius * slice_angle.sin()].into(),
        );
    }

    let vertex_count = mesh_data.vertex_count();
    for i in 1..mesh_data.vertex_count() {
        mesh_data.indices.push(0);
        mesh_data.indices.push(i as _);
        mesh_data.indices.push(((i + 1) % vertex_count) as _);
    }

    let meshlet = MeshletData {
        vertices_count: mesh_data.vertex_count() as _,
        indices_count: mesh_data.index_count() as _,
        ..Default::default()
    };
    mesh_data.meshlets.push(meshlet);
    mesh_data
}

pub fn create_hammer(position: Vector3, direction: Vector3, color: Vector4) -> MeshData {
    let mut shape_mesh_data = MeshData::default();

    let height = direction.length();

    let mut cylinder_mesh_data = create_cylinder(0.25, 0.25, 16, height, 1, color);
    cylinder_mesh_data.aabb_min.y += height * 0.5;
    cylinder_mesh_data.aabb_max.y += height * 0.5;
    shape_mesh_data.append_mesh_data(cylinder_mesh_data, false);

    let mut cube_mesh_data = create_cube_from_min_max(
        Vector3::new(-0.5, -0.5, -0.5),
        Vector3::new(0.5, 0.5, 0.5),
        color,
    );
    cube_mesh_data.aabb_min.y += height;
    cube_mesh_data.aabb_max.y += height;
    shape_mesh_data.append_mesh_data(cube_mesh_data, false);

    let mut matrix = Matrix4::default_identity();
    matrix.look_towards(direction);
    shape_mesh_data.aabb_min = position + matrix.transform(shape_mesh_data.aabb_min);
    shape_mesh_data.aabb_max = position + matrix.transform(shape_mesh_data.aabb_max);
    shape_mesh_data
}

pub fn create_torus(
    position: Vector3,
    main_radius: f32,
    tube_radius: f32,
    num_main_slices: u32,
    num_tube_slices: u32,
    direction: Vector3,
    color: Vector4,
) -> MeshData {
    let mut mesh_data = MeshData::default();

    let main_step = 2. * PI / num_main_slices as f32;
    let tube_step = 2. * PI / num_tube_slices as f32;

    for i in 0..num_main_slices + 1 {
        let main_angle = i as f32 * main_step;
        for j in 0..num_tube_slices + 1 {
            let tube_angle = j as f32 * tube_step;
            mesh_data.add_vertex_pos_color_normal_uv(
                [
                    (main_radius + tube_radius * tube_angle.cos()) * main_angle.cos(),
                    (main_radius + tube_radius * tube_angle.cos()) * main_angle.sin(),
                    tube_radius * tube_angle.sin(),
                ]
                .into(),
                color,
                [
                    main_angle.cos() * tube_angle.cos(),
                    main_angle.sin() * tube_angle.cos(),
                    tube_angle.sin(),
                ]
                .into(),
                [
                    j as f32 / num_tube_slices as f32,
                    i as f32 * (2. / num_main_slices as f32),
                ]
                .into(),
            );
        }
    }
    for i in 0..num_main_slices + 1 {
        let mut k1 = i * (num_main_slices); // bebinning of current stack
        let mut k2 = k1 + num_main_slices; // beginning of next stack

        for _ in 0..num_tube_slices + 1 {
            mesh_data.indices.push(k1);
            mesh_data.indices.push(k1 + 1);
            mesh_data.indices.push(k2);

            mesh_data.indices.push(k2);
            mesh_data.indices.push(k1 + 1);
            mesh_data.indices.push(k2 + 1);

            k1 += 1;
            k2 += 1;
        }
    }

    let meshlet = MeshletData {
        vertices_count: mesh_data.vertex_count() as _,
        indices_count: mesh_data.index_count() as _,
        ..Default::default()
    };
    mesh_data.meshlets.push(meshlet);

    let mut matrix = Matrix4::default_identity();
    matrix.look_towards(direction);
    mesh_data.aabb_min = position + matrix.transform(mesh_data.aabb_min);
    mesh_data.aabb_max = position + matrix.transform(mesh_data.aabb_max);
    mesh_data
}
