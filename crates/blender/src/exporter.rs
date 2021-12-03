use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
};

use pyo3::{
    types::{PyDict, PyList},
    PyObject, PyResult, Python, ToPyObject,
};
use sabi_scene::{ObjectData, SceneData};
use sabi_serialize::SerializeFile;

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

            if create_dir_all(self.export_dir.as_path()).is_ok() {
                // Blender data import
                let export_scene = py.import("bpy")?.getattr("ops")?.getattr("export_scene")?;

                let scene_path = self.export_dir.join(format!(
                    "{}.{}",
                    filename.to_str().unwrap_or("Scene"),
                    "gltf"
                ));
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
                export_scene.call_method("gltf", (), Some(kwargs))?;

                self.export_custom_data(py, self.export_dir.as_path())?;

                let scene_path = scene_path.replace("data_raw", "data");
                let scene_path = scene_path.replace(".gltf", ".scene_data");
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
            let scene_name: String = scene.getattr("name")?.extract()?;
            let mut scene_data = SceneData::default();
            scene_data.load_from_file(
                export_dir
                    .join(format!("{}.{}", scene_name, SceneData::extension()).as_str())
                    .as_path(),
            );
            let objects = scene.getattr("objects")?.call_method("values", (), None)?;
            let objects = objects.cast_as::<PyList>()?;
            for object in objects.iter() {
                let object_name: String = object.getattr("name")?.extract()?;
                let mut object_data = ObjectData::default();
                let path = export_dir
                    .join(format!("{}.{}", object_name, ObjectData::extension()).as_str());
                object_data.load_from_file(path.as_path());
                self.process_object_properties(
                    py,
                    &object.to_object(py),
                    &mut object_data,
                    path.as_path(),
                )?;
            }
        }
        Ok(true)
    }

    fn process_object_properties(
        &self,
        py: Python,
        object: &PyObject,
        object_data: &mut ObjectData,
        path: &Path,
    ) -> PyResult<bool> {
        let is_dirty = false;
        if let Ok(properties) = object.getattr(py, "sabi_properties") {
            let logic = properties.getattr(py, "logic")?;
            self.export_logic(py, &logic, object_data)?;
        }
        if is_dirty {
            object_data.save_to_file(path);
        }
        Ok(true)
    }

    fn export_logic(
        &self,
        py: Python,
        logic: &PyObject,
        _object_data: &mut ObjectData,
    ) -> PyResult<bool> {
        if !logic.is_none(py) {
            let data: String = logic.call_method(py, "serialize", (), None)?.extract(py)?;
            println!("NodeTree:\n{}", data);
            /*
            let name: String = logic.getattr(py, "name")?.extract(py)?;
            println!("NodeTree: {}", name);
            let nodes = logic
                .getattr(py, "nodes")?
                .call_method(py, "values", (), None)?;
            let nodes = nodes.cast_as::<PyList>(py)?;
            let mut serialized_nodes = HashMap::new();
            for node in nodes.iter() {
                let node_name: String = node.getattr("name")?.extract()?;
                let node_type: String = node.getattr("bl_idname")?.extract()?;

                println!("Node: {}", node_name);
                println!("Type: {}", node_type);
                let data: String = node.call_method("serialize", (), None)?.extract()?;
                serialized_nodes.insert(node_name, data);
            }
            */
        }
        Ok(true)
    }
}
