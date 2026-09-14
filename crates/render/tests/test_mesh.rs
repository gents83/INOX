use inox_render::MeshFlags;
use inox_serialize::{deserialize, serialize};

#[test]
fn serializes_and_restores_mesh_flags() {
    let flags = MeshFlags::Visible | MeshFlags::Tranparent | MeshFlags::Wireframe;
    let bytes = serialize(&flags);
    let restored = deserialize::<MeshFlags>(&bytes).expect("mesh flags should deserialize");

    assert_eq!(restored, flags);
    assert!(restored.contains(MeshFlags::Visible));
    assert!(restored.contains(MeshFlags::Tranparent));
    assert!(restored.contains(MeshFlags::Wireframe));
    assert!(!restored.contains(MeshFlags::Opaque));
}
