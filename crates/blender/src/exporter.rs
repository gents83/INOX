use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
};

use cpython::{
    serde::from_py_object, NoArgs, ObjectProtocol, PyDict, PyList, PyObject, PyResult, PyTuple,
    Python,
};

use nrg_filesystem::convert_in_local_path;
use nrg_graphics::{MeshData, VertexData};
use nrg_math::{Matrix4, Quaternion, Vector2, Vector3};

use nrg_scene::{ObjectData, SceneData};
use nrg_serialize::serialize_to_file;

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
                let data = py.import("bpy")?.get(py, "data")?;

                // For every Blender scene
                let scenes = data
                    .getattr(py, "scenes")?
                    .call_method(py, "values", NoArgs, None)?;
                let scenes = scenes.cast_as::<PyList>(py)?;
                for scene in scenes.iter(py) {
                    // Load scene
                    scene_paths.push(self.export_scene(py, scene)?);
                }
            }
        }
        Ok(scene_paths)
    }

    fn export_scene(&self, py: Python, scene: PyObject) -> PyResult<PathBuf> {
        let scene_name: String = scene.getattr(py, "name")?.extract(py)?;
        let mut scene_data = SceneData::default();
        let objects = scene
            .getattr(py, "objects")?
            .call_method(py, "values", NoArgs, None)?;
        let objects = objects.cast_as::<PyList>(py)?;
        for object in objects.iter(py) {
            if object.getattr(py, "parent")? != py.None() {
                continue;
            }
            let o = self.export_object(py, &object)?;
            scene_data.objects.push(o);
        }

        Ok(self.create_file(scene_data, scene_name.as_str(), "scene_data"))
    }

    fn export_object(&self, py: Python, object: &PyObject) -> PyResult<PathBuf> {
        let object_name: String = object.getattr(py, "name")?.extract(py)?;
        let mut object_data = ObjectData::default();

        // The type of Blender object
        let object_type: String = object.getattr(py, "type")?.extract(py)?;
        // If the object is a mesh
        if object_type == "MESH" {
            let mesh = self.export_mesh(py, &object)?;
            object_data.components.push(mesh);
            // If the object is a camera
        } else if object_type == "CAMERA" {
            // TODO: Handle camera objects
        }

        if let Ok(transform) = self.export_transform(py, object) {
            object_data.transform = transform;
        }

        if let Ok(blend_children) = object.getattr(py, "children") {
            let children = blend_children.cast_as::<PyTuple>(py)?;
            for child in children.iter(py) {
                let o = self.export_object(py, child)?;
                object_data.children.push(o);
            }
        }
        Ok(self.create_file(object_data, object_name.as_str(), "object_data"))
    }

    fn export_materials(&self, py: Python, object: &PyObject) -> PyResult<Vec<PathBuf>> {
        let materials_paths = Vec::new();
        let materials = object
            .getattr(py, "material_slots")?
            .call_method(py, "values", NoArgs, None)?;
        let materials = materials.cast_as::<PyList>(py)?;

        let mut material_names = Vec::new();
        for (index, mat) in materials.iter(py).enumerate() {
            if let Ok(name) = mat.getattr(py, "name")?.extract(py) {
                material_names.push(name);
            } else {
                material_names.push(format!("material_{}", index));
            }
        }

        println!("Materials = {:?}", material_names);

        Ok(materials_paths)
    }

    fn export_mesh(&self, py: Python, mesh: &PyObject) -> PyResult<PathBuf> {
        self.export_materials(py, mesh)?;
        let mesh_name: String = mesh.getattr(py, "name")?.extract(py)?;

        let context = py.import("bpy")?.get(py, "context")?;
        let depsgraph = context.call_method(py, "evaluated_depsgraph_get", NoArgs, None)?;
        let mesh = mesh.call_method(py, "evaluated_get", (depsgraph,), None)?;
        let depsgraph = context.call_method(py, "evaluated_depsgraph_get", NoArgs, None)?;
        let kwargs = PyDict::new(py);
        kwargs.set_item(py, "preserve_all_data_layers", true)?;
        kwargs.set_item(py, "depsgraph", depsgraph)?;
        let mesh = mesh.call_method(py, "to_mesh", NoArgs, Some(&kwargs))?;

        let mut mesh_data = MeshData::default();

        let uv_layers = mesh.getattr(py, "uv_layers")?;
        let is_uv_layer_active = if let Ok(value) = uv_layers.getattr(py, "active")?.extract(py) {
            value
        } else {
            false
        };
        let uv_layers = uv_layers.call_method(py, "values", NoArgs, None)?;
        let uv_layers = uv_layers.cast_as::<PyList>(py)?;

        //Tessellate and triangulate
        mesh.call_method(py, "validate", NoArgs, None)?;
        mesh.call_method(py, "calc_loop_triangles", NoArgs, None)?;
        mesh.call_method(py, "calc_normals_split", NoArgs, None)?;
        if is_uv_layer_active && uv_layers.len(py) > 0 {
            mesh.call_method(py, "calc_tangents", NoArgs, None)?;
        }

        let vertices = mesh
            .getattr(py, "vertices")?
            .call_method(py, "values", NoArgs, None)?;
        let vertices = vertices.cast_as::<PyList>(py)?;

        let triangles = mesh
            .getattr(py, "loop_triangles")?
            .call_method(py, "values", NoArgs, None)?;
        let triangles = triangles.cast_as::<PyList>(py)?;

        let loops = mesh
            .getattr(py, "loops")?
            .call_method(py, "values", NoArgs, None)?;
        let loops = loops.cast_as::<PyList>(py)?;

        for v in vertices.iter(py) {
            let coords = v.getattr(py, "co")?;
            let normal = v.getattr(py, "normal")?;
            let vertex = VertexData {
                pos: Vector3::new(
                    coords.getattr(py, "x")?.extract(py)?,
                    coords.getattr(py, "z")?.extract(py)?,
                    -coords.getattr(py, "y")?.extract(py)?,
                ),
                normal: Vector3::new(
                    normal.getattr(py, "x")?.extract(py)?,
                    normal.getattr(py, "z")?.extract(py)?,
                    -normal.getattr(py, "y")?.extract(py)?,
                ),
                ..Default::default()
            };
            mesh_data.vertices.push(vertex);
        }

        for triangle in triangles.iter(py) {
            if uv_layers.len(py) > 0 {
                let tri_loops: [usize; 3] = from_py_object(py, triangle.getattr(py, "loops")?)?;
                for loop_index in tri_loops {
                    let mesh_loop = loops.get_item(py, loop_index);
                    let vertex_index: usize = mesh_loop.getattr(py, "vertex_index")?.extract(py)?;
                    let tangent: [f32; 3] = from_py_object(py, mesh_loop.getattr(py, "tangent")?)?;
                    mesh_data.vertices[vertex_index].tangent =
                        Vector3::new(tangent[0], tangent[2], -tangent[1]);

                    for i in 0..uv_layers.len(py) {
                        let uv_loops = uv_layers
                            .get_item(py, i)
                            .getattr(py, "data")?
                            .call_method(py, "values", NoArgs, None)?;
                        let uv_loops = uv_loops.cast_as::<PyList>(py)?;

                        let uv: [f32; 2] = from_py_object(
                            py,
                            uv_loops.get_item(py, loop_index).getattr(py, "uv")?,
                        )?;
                        mesh_data.vertices[vertex_index].tex_coord[i] = Vector2::new(uv[0], uv[1]);
                    }
                }
            }

            let verts: [u32; 3] = from_py_object(py, triangle.getattr(py, "vertices")?)?;
            mesh_data.indices.append(&mut verts.to_vec());
        }

        Ok(self.create_file(mesh_data, mesh_name.as_str(), "mesh_data"))
    }

    fn export_transform(&self, py: Python, object: &PyObject) -> PyResult<Matrix4> {
        let transform = if object.getattr(py, "parent")? != py.None() {
            object.getattr(py, "matrix_world")?
        } else {
            object.getattr(py, "matrix_local")?
        };

        let result = transform.call_method(py, "decompose", NoArgs, None)?;
        let tuple = result.cast_as::<PyTuple>(py)?;
        let translation: [f32; 3] = from_py_object(py, tuple.get_item(py, 0))?;
        let rotation: [f32; 4] = from_py_object(py, tuple.get_item(py, 1))?;
        let scale: [f32; 3] = from_py_object(py, tuple.get_item(py, 2))?;
        // Change coordinates so that Y is up
        let t = Vector3::new(translation[0], translation[2], -translation[1]);
        let r = Quaternion::new(rotation[1], rotation[3], -rotation[2], rotation[0]);
        let s = Vector3::new(scale[0], scale[2], scale[1]);

        let matrix = Matrix4::from_translation(t)
            * Matrix4::from(r)
            * Matrix4::from_nonuniform_scale(s.x, s.y, s.z);
        Ok(matrix)
    }

    fn create_file<T>(&self, data: T, new_name: &str, new_extension: &str) -> PathBuf
    where
        T: nrg_serialize::Serialize,
    {
        let destination_ext = format!("{}.{}", new_name, new_extension);
        let new_path = self.export_dir.join(destination_ext);
        if !new_path.exists() {
            let result = create_dir_all(new_path.parent().unwrap());
            debug_assert!(result.is_ok());
        }
        println!("Serializing {:?}", new_path);
        serialize_to_file(&data, new_path.as_path());
        convert_in_local_path(
            new_path.as_path(),
            self.working_dir.join("data_raw").as_path(),
        )
    }
}
