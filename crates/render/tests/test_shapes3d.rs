use inox_math::{VecBase, Vector3, Vector4};
use inox_render::{create_cylinder, create_torus};

fn assert_valid_indices(mesh: &inox_render::MeshData) {
    let vertex_count = mesh.vertex_count() as u32;
    assert!(mesh.indices.iter().all(|index| *index < vertex_count));
    assert_eq!(mesh.indices.len() % 3, 0);
    for triangle in mesh.indices.as_chunks::<3>().0 {
        assert!(triangle[0] != triangle[1] || triangle[1] != triangle[2]);
    }
}

#[test]
fn cylinder_indices_match_non_square_side_grid() {
    let mesh = create_cylinder(1.0, 1.0, 5, 2.0, 3, Vector4::default_one());
    assert_valid_indices(&mesh);
    assert_eq!(mesh.indices.len(), 5 * 3 * 6 + 5 * 3 * 2);
    assert_eq!(&mesh.indices[0..6], &[0, 1, 6, 6, 1, 7]);
    assert_eq!(&mesh.indices[84..90], &[16, 17, 22, 22, 17, 23]);
}

#[test]
fn torus_indices_match_unequal_main_and_tube_segments() {
    let mesh = create_torus(
        Vector3::default_zero(),
        2.0,
        0.5,
        7,
        5,
        Vector3::new(0.0, 1.0, 0.0),
        Vector4::default_one(),
    );
    assert_valid_indices(&mesh);
    assert_eq!(mesh.indices.len(), 7 * 5 * 6);
    assert_eq!(&mesh.indices[0..6], &[0, 1, 6, 6, 1, 7]);
    assert_eq!(&mesh.indices[7 * 5 * 6 - 6..], &[40, 41, 4, 4, 41, 5]);
}
