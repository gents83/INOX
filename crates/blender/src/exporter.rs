use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
};

use pyo3::{
    types::{PyDict, PyList},
    PyObject, PyResult, Python, ToPyObject,
};

use sabi_nodes::{LogicData, NodeTree};

use sabi_serialize::{deserialize, SerializeFile};

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
            let scene_name = filename.to_str().unwrap_or("Scene");
            self.working_dir = working_dir.to_path_buf();
            self.export_dir = self
                .working_dir
                .join("data_raw")
                .join("blender_export")
                .join(scene_name);

            if create_dir_all(self.export_dir.as_path()).is_ok() {
                // Blender data import
                let export_scene = py.import("bpy")?.getattr("ops")?.getattr("export_scene")?;
                let scene_path = self.export_dir.join(format!("{}.{}", scene_name, "gltf"));
                let scene_path = scene_path.to_str().unwrap_or_default().to_string();

                let kwargs = PyDict::new(py);
                kwargs.set_item("filepath", scene_path.clone())?;
                kwargs.set_item("check_existing", true)?;
                kwargs.set_item("export_format", "GLTF_SEPARATE")?;
                kwargs.set_item("export_apply", true)?;
                kwargs.set_item("export_materials", "EXPORT")?;
                kwargs.set_item("export_cameras", true)?;
                kwargs.set_item("export_yup", true)?;
                kwargs.set_item("export_lights", true)?;
                kwargs.set_item("export_extras", true)?;
                kwargs.set_item("export_texture_dir", "./textures/")?;
                export_scene.call_method("gltf", (), Some(kwargs))?;

                self.export_custom_data(py, self.export_dir.as_path())?;

                let scene_path = scene_path.replace("data_raw", "data");
                let scene_path = scene_path.replace(".gltf", ".scene");
                scene_paths.push(PathBuf::from(scene_path));
            }
        }
        Ok(scene_paths)
    }

    fn export_custom_data(&self, py: Python, export_dir: &Path) -> PyResult<bool> {
        let data = py.import("bpy")?.getattr("data")?;

        // For every Blender scene
        let scenes = data.getattr("scenes")?.call_method("values", (), None)?;
        let scenes = scenes.cast_as::<PyList>()?;
        for scene in scenes.iter() {
            let objects = scene.getattr("objects")?.call_method("values", (), None)?;
            let objects = objects.cast_as::<PyList>()?;
            for object in objects.iter() {
                self.process_object_properties(py, &object.to_object(py), export_dir)?;
            }
        }
        Ok(true)
    }

    fn process_object_properties(
        &self,
        py: Python,
        object: &PyObject,
        path: &Path,
    ) -> PyResult<bool> {
        if let Ok(properties) = object.getattr(py, "sabi_properties") {
            if let Ok(logic) = properties.getattr(py, "logic") {
                self.export_logic(py, &logic, path)?;
            }
        }
        Ok(true)
    }

    fn export_logic(&self, py: Python, logic: &PyObject, path: &Path) -> PyResult<bool> {
        let export_dir = path.join(LogicData::extension());
        if !logic.is_none(py) && create_dir_all(export_dir.as_path()).is_ok() {
            let data: String = logic.call_method(py, "serialize", (), None)?.extract(py)?;
            let name: String = logic.getattr(py, "name")?.extract(py)?;

            if let Ok(node_tree) = deserialize::<NodeTree>(&data) {
                let path = export_dir.join(format!("{}.{}", name, LogicData::extension()).as_str());
                println!("NodeTree deserialized in {:?}", path);
                node_tree.save_to_file(path.as_path());
            }
        }
        Ok(true)
    }
}
