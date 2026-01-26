use inox_math::{Vector3, Vector4};
use inox_resources::to_slice;
use metis::Graph;

use crate::mesh::MeshVertex;

#[allow(dead_code)]
fn simplify_test() {
    // 4----5----6
    // |    |    |
    // 1----2----7
    // |    |    |
    // 0----3----8
    #[rustfmt::skip]
    let vertices = [
        MeshVertex { pos: Vector4::new(0., 0., 0., 1.), ..Default::default() },
        MeshVertex { pos: Vector4::new(0., 1., 0., 1.), ..Default::default() },
        MeshVertex { pos: Vector4::new(1., 1., 0., 1.), ..Default::default() },
        MeshVertex { pos: Vector4::new(1., 0., 0., 1.), ..Default::default() },
        MeshVertex { pos: Vector4::new(0., 2., 0., 1.), ..Default::default() },
        MeshVertex { pos: Vector4::new(1., 2., 0., 1.), ..Default::default() },
        MeshVertex { pos: Vector4::new(2., 2., 0., 1.), ..Default::default() },
        MeshVertex { pos: Vector4::new(2., 1., 0., 1.), ..Default::default() },
        MeshVertex { pos: Vector4::new(2., 0., 0., 1.), ..Default::default() },
    ];
    #[rustfmt::skip]
    let indices = [
        0, 1, 2,
        2, 3, 0,
        1, 4, 5,
        5, 2, 1,
        2, 5, 6,
        6, 7, 2,
        2, 7, 3,
        3, 7, 8,
    ];

    let vertex_stride = size_of::<MeshVertex>();

    struct TestStruct {
        pos: Vector3,
        tangent: Vector4,
    }
    let test = [MeshVertex {
        pos: Vector4::new(0., 1., 0., 1.),
        ..Default::default()
    }];
    let test_bytes: &[u8] = to_slice(&test);
    println!("test_bytes ({}): {:?}", test_bytes.len(), test_bytes);
    println!("vertex_stride: {vertex_stride}");

    let vertices_bytes = to_slice(&vertices);
    let vertex_stride = size_of::<MeshVertex>();
    let vertex_data_adapter = meshopt::VertexDataAdapter::new(vertices_bytes, vertex_stride, 0);

    println!(
        "vertices_bytes ({}): {:?}",
        vertices_bytes.len(),
        vertices_bytes
    );

    let target_count = 6;
    let target_error = 0.01;
    let simplified_indices = meshopt::simplify(
        &indices,
        vertex_data_adapter.as_ref().unwrap(),
        target_count,
        target_error,
        meshopt::SimplifyOptions::LockBorder
            | meshopt::SimplifyOptions::Sparse
            | meshopt::SimplifyOptions::ErrorAbsolute,
        None,
    );

    println!("Indices ({}): {:?}", indices.len(), indices);
    println!(
        "Simplified indices ({}): {:?}",
        simplified_indices.len(),
        simplified_indices
    );

    debug_assert!(
        !simplified_indices.is_empty() && simplified_indices.len() < indices.len(),
        "No simplification happened with meshoptimizer"
    );
}

#[allow(dead_code)]
fn partition_test() {
    // 0----3----6----9----12
    // | #0 | #1 | #2 | #3 |
    // 1----4----7----10---13
    // | #4 | #5 | #6 | #7 |
    // 2----5----8----11---14
    #[rustfmt::skip]
    let clusters_adiacency = [
        vec![1, 4],
        vec![0, 2, 5],
        vec![1, 3, 6],
        vec![2, 7],
        vec![0, 5],
        vec![1, 4, 6],
        vec![2, 5, 7],
        vec![3, 6],
    ];

    let num_meshlets = clusters_adiacency.len();
    let mut xadj = Vec::new();
    let mut adjncy = Vec::new();
    for a in clusters_adiacency {
        let start = adjncy.len() as i32;
        xadj.push(start);
        for v in a {
            adjncy.push(v);
        }
    }
    xadj.push(adjncy.len() as i32);

    let mut groups = Vec::new();
    let expected = vec![vec![0, 1, 4, 5], vec![2, 3, 6, 7]];
    let num_groups = expected.len();
    if let Ok(graph) = Graph::new(1, num_groups as i32, &xadj, &adjncy) {
        let mut part = vec![0; num_meshlets];
        if let Ok(result) = graph.part_kway(&mut part) {
            for group_index in 0..num_groups as i32 {
                let mut group = Vec::new();
                part.iter().enumerate().for_each(|(i, &v)| {
                    if v == group_index {
                        group.push(i);
                    }
                });
                group.sort();
                groups.push(group);
            }
            groups.sort();
            println!("Result[{result}] = {groups:?}");
        }
    }
    debug_assert!(
        groups == expected,
        "\nExpecting: {expected:?}\nResult: {groups:?}"
    );
}

#[allow(dead_code)]
fn weighted_partition_test() {
    // X----X----X----X----X-----X----X
    // |         |                    |
    // X   #0    X----X   #1          X
    // |              |               |
    // X----X----X----X----X-----X----X
    // |                   |          |
    // X        #3    X----X    #2    X
    // |              |    |          |
    // X----X----X----X    X-----X----X
    // |              |               |
    // X      #4      X      #5       X
    // |              |               |
    // X----X----X----X----X-----X----X
    #[rustfmt::skip]
    let clusters_adiacency = [
        vec![1, 3],
        vec![0, 2, 3],
        vec![1, 3, 5],
        vec![0, 1, 2, 4, 5],
        vec![3, 5],
        vec![2, 3, 4],
    ];
    #[rustfmt::skip]
    let shared_edges = [
        vec![0, 3, 0, 3, 0, 0],
        vec![3, 0, 2, 1, 0, 0],
        vec![0, 2, 0, 1, 0, 3],
        vec![3, 1, 1, 0, 3, 2],
        vec![0, 0, 0, 3, 0, 2],
        vec![0, 0, 3, 2, 2, 0],
    ];

    let num_meshlets = clusters_adiacency.len();
    let mut xadj = Vec::new();
    let mut adjncy = Vec::new();
    let mut adjwgt = Vec::new();
    clusters_adiacency
        .iter()
        .enumerate()
        .for_each(|(cluster_index, a)| {
            let start = adjncy.len() as i32;
            xadj.push(start);
            for v in a {
                adjncy.push(*v);
                adjwgt.push(shared_edges[cluster_index][*v as usize]);
            }
        });
    xadj.push(adjncy.len() as i32);

    let num_groups = 3;

    let mut groups = Vec::new();
    let expected_without_weights = vec![vec![0, 1], vec![2, 5], vec![3, 4]];
    let expected_with_weights = vec![vec![0, 1], vec![2, 5], vec![3, 4]];

    if let Ok(graph) = Graph::new(1, num_groups, &xadj, &adjncy) {
        let mut part = vec![0; num_meshlets];
        if let Ok(result) = graph.part_kway(&mut part) {
            for group_index in 0..num_groups {
                let mut group = Vec::new();
                part.iter().enumerate().for_each(|(i, &v)| {
                    if v == group_index {
                        group.push(i);
                    }
                });
                group.sort();
                groups.push(group);
            }
            groups.sort();
            println!("Result[{result}] = {groups:?}");
        }
    }
    assert_eq!(
        groups, expected_without_weights,
        "\nExpecting: {expected_without_weights:?}\nResult: {expected_without_weights:?}"
    );
    groups.clear();

    if let Ok(graph) = Graph::new(1, num_groups, &xadj, &adjncy) {
        let mut part = vec![0; num_meshlets];
        if let Ok(result) = graph.set_adjwgt(&adjwgt).part_kway(&mut part) {
            for group_index in 0..num_groups {
                let mut group = Vec::new();
                part.iter().enumerate().for_each(|(i, &v)| {
                    if v == group_index {
                        group.push(i);
                    }
                });
                group.sort();
                groups.push(group);
            }
            groups.sort();
            println!("Result[{result}] = {groups:?}");
        }
    }
    debug_assert!(
        groups == expected_with_weights,
        "\nExpecting: {expected_with_weights:?}\nResult: {expected_with_weights:?}"
    );
}

#[test]
fn tests() {
    simplify_test();
    partition_test();
    weighted_partition_test();
}
