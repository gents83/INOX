use inox_scene::CameraData;
use inox_serialize::SerializeFile;

#[test]
fn camera_data_defaults_match_the_runtime_file_format() {
    let data = CameraData::default();

    assert!(data.aspect_ratio > 0.0);
    assert!(data.near < data.far);
    assert_eq!(CameraData::extension(), "camera");
}
