use std::{
    fs::{create_dir_all, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use pyo3::{
    prelude::PyAnyMethods,
    types::{PyDict, PyDictMethods, PyList},
    IntoPyObjectExt, Py, PyAny, PyResult, Python,
};

use inox_nodes::LogicData;

use inox_serialize::SerializeFile;

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
                export_scene.call_method("gltf", (), Some(&kwargs))?;

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
        let scenes = scenes.cast::<PyList>()?;
        if let Ok(scene) = scenes.try_iter() {
            let objects = scene.getattr("objects")?.call_method("values", (), None)?;
            let objects = objects.cast::<PyList>()?;
            if let Ok(object) = objects.try_iter() {
                self.process_object_properties(py, &object.into_py_any(py)?, export_dir)?;
            }
        }
        Ok(true)
    }

    fn process_object_properties(
        &self,
        py: Python,
        object: &Py<PyAny>,
        path: &Path,
    ) -> PyResult<bool> {
        if let Ok(properties) = object.getattr(py, "inox_properties") {
            if let Ok(logic) = properties.getattr(py, "logic") {
                self.export_logic(py, &logic, path)?;
            }
        }
        Ok(true)
    }

    fn export_logic(&self, py: Python, logic: &Py<PyAny>, path: &Path) -> PyResult<bool> {
        let export_dir = path.join(LogicData::extension());
        if !logic.is_none(py) && create_dir_all(export_dir.as_path()).is_ok() {
            let mut data: String = logic.call_method(py, "serialize", (), None)?.extract(py)?;
            data = data.replace(": ", ":");
            data = data.replace(", ", ",");
            let name: String = logic.getattr(py, "name")?.extract(py)?;
            let path = export_dir.join(format!("{}.{}", name, LogicData::extension()).as_str());

            let file = File::create(path.as_path()).unwrap();
            let mut writer = BufWriter::new(file);
            if writer.write_all(data.as_bytes()).is_ok() {
                println!("NodeTree {name} exported in {path:?}");
            } else {
                eprintln!("Failed to deserialize logic {name}");
            }
        }
        Ok(true)
    }
}
