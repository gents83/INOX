use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
};

use cpython::{NoArgs, ObjectProtocol, PyDict, PyList, PyResult, Python};
use nrg_scene::{ObjectData, SceneData};
use nrg_serialize::SerializeFile;

#[derive(Default)]
pub struct Exporter {
    working_dir: PathBuf,
    export_dir: PathBuf,
}

impl Exporter {
    pub fn process(
        &mut self,
        py: Python,
        working_dir: &Path,
        file_to_export: &Path,
    ) -> PyResult<Vec<PathBuf>> {
        println!("Exporting scene from Blender...");

        let mut scene_paths = Vec::new();
        if let Some(filename) = file_to_export.file_stem() {
            self.working_dir = working_dir.to_path_buf();
            self.export_dir = self
                .working_dir
                .join("data_raw")
                .join("blender_export")
                .join(filename);

            if let Ok(_) = create_dir_all(self.export_dir.as_path()) {
                // Blender data import
                let export_scene = py
                    .import("bpy")?
                    .get(py, "ops")?
                    .getattr(py, "export_scene")?;

                let scene_path = self.export_dir.join(format!(
                    "{}.{}",
                    filename.to_str().unwrap_or("Scene"),
                    "gltf"
                ));
                let scene_path = scene_path.to_str().unwrap_or_default().to_string();

                let kwargs = PyDict::new(py);
                kwargs.set_item(py, "filepath", scene_path.clone())?;
                kwargs.set_item(py, "check_existing", true)?;
                kwargs.set_item(py, "export_format", "GLTF_SEPARATE")?;
                kwargs.set_item(py, "export_apply", true)?;
                kwargs.set_item(py, "export_materials", "EXPORT")?;
                kwargs.set_item(py, "export_cameras", true)?;
                kwargs.set_item(py, "export_yup", true)?;
                kwargs.set_item(py, "export_lights", true)?;
                export_scene.call_method(py, "gltf", NoArgs, Some(&kwargs))?;

                self.export_custom_data(py, self.export_dir.as_path())?;

                let scene_path = scene_path.replace("data_raw", "data");
                let scene_path = scene_path.replace(".gltf", ".scene_data");
                scene_paths.push(PathBuf::from(scene_path));
            }
        }
        Ok(scene_paths)
    }

    fn export_custom_data(&self, py: Python, export_dir: &Path) -> PyResult<bool> {
        let data = py.import("bpy")?.get(py, "data")?;

        // For every Blender scene
        let scenes = data
            .getattr(py, "scenes")?
            .call_method(py, "values", NoArgs, None)?;
        let scenes = scenes.cast_as::<PyList>(py)?;
        for scene in scenes.iter(py) {
            let scene_name: String = scene.getattr(py, "name")?.extract(py)?;
            let mut scene_data = SceneData::default();
            scene_data.load_from_file(
                export_dir
                    .join(format!("{}.{}", scene_name, SceneData::extension()).as_str())
                    .as_path(),
            );
            let objects = scene
                .getattr(py, "objects")?
                .call_method(py, "values", NoArgs, None)?;
            let objects = objects.cast_as::<PyList>(py)?;
            for object in objects.iter(py) {
                let object_name: String = object.getattr(py, "name")?.extract(py)?;
                let mut object_data = ObjectData::default();
                object_data.load_from_file(
                    export_dir
                        .join(format!("{}.{}", object_name, ObjectData::extension()).as_str())
                        .as_path(),
                );
                //println!("Checking object {:?}", object_name);
            }
        }
        Ok(true)
    }
}
