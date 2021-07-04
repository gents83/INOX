use std::{
    fs::{self, create_dir_all, File},
    io::{Seek, SeekFrom},
    path::{Path, PathBuf},
};

use crate::{need_to_binarize, ExtensionHandler, Parser};
use gltf::{
    accessor::{DataType, Dimensions},
    buffer::{Source, View},
    image::Source as ImageSource,
    mesh::Mode,
    Accessor, Gltf, Node, Primitive, Semantic,
};
use nrg_graphics::{MaterialData, MeshData, VertexData};
use nrg_math::{Vector2, Vector3, Vector4};
use nrg_messenger::MessengerRw;
use nrg_resources::{convert_in_local_path, DATA_FOLDER, DATA_RAW_FOLDER};
use nrg_scene::ObjectData;
use nrg_serialize::serialize_to_file;

const GLTF_EXTENSION: &str = "gltf";
const MESH_DATA_EXTENSION: &str = "mesh_data";
const MATERIAL_DATA_EXTENSION: &str = "material_data";
const OBJECT_DATA_EXTENSION: &str = "object_data";

pub struct GltfCompiler {
    global_messenger: MessengerRw,
}

impl GltfCompiler {
    pub fn new(global_messenger: MessengerRw) -> Self {
        Self { global_messenger }
    }

    fn num_from_type(accessor: &Accessor) -> usize {
        match accessor.dimensions() {
            Dimensions::Vec2 => 2,
            Dimensions::Vec3 => 3,
            Dimensions::Vec4 => 4,
            Dimensions::Mat2 => 4,
            Dimensions::Mat3 => 9,
            Dimensions::Mat4 => 16,
            _ => 1,
        }
    }
    fn bytes_from_dimension(accessor: &Accessor) -> usize {
        match accessor.data_type() {
            DataType::F32 | DataType::U32 => 4,
            DataType::U16 | DataType::I16 => 2,
            _ => 1,
        }
    }

    fn read_accessor_from_path<T>(path: &Path, accessor: &Accessor) -> Option<Vec<T>>
    where
        T: Parser,
    {
        let view = if let Some(sparse) = accessor.sparse() {
            Some(sparse.values().view())
        } else {
            accessor.view()
        };
        if let Some(view) = view {
            if let Some(parent_folder) = path.parent() {
                match view.buffer().source() {
                    Source::Uri(local_path) => {
                        let filepath = parent_folder.to_path_buf().join(local_path);
                        if let Ok(mut file) = fs::File::open(filepath) {
                            return Some(Self::read_from_file::<T>(&mut file, &view, &accessor));
                        } else {
                            eprintln!("Unable to open file: {:?}", local_path);
                        }
                    }
                    Source::Bin => {}
                }
            }
        }
        None
    }

    fn read_from_file<T>(file: &mut File, view: &View, accessor: &Accessor) -> Vec<T>
    where
        T: Parser,
    {
        let count = accessor.count();
        let view_offset = view.offset();
        let accessor_offset = accessor.offset();
        let starting_offset = view_offset + accessor_offset;
        let view_stride = view.stride().unwrap_or(0);
        let type_stride = T::size();
        let stride = if view_stride > type_stride {
            view_stride - type_stride
        } else {
            0
        };
        let mut result = Vec::new();
        file.seek(SeekFrom::Start(starting_offset as _)).ok();
        for _i in 0..count {
            let v = T::parse(file);
            result.push(v);
            file.seek(SeekFrom::Current(stride as _)).ok();
        }
        result
    }

    fn extract_indices(path: &Path, primitive: &Primitive) -> Vec<u32> {
        let mut indices = Vec::new();
        debug_assert!(primitive.mode() == Mode::Triangles);
        if let Some(accessor) = primitive.indices() {
            let num = Self::num_from_type(&accessor);
            let num_bytes = Self::bytes_from_dimension(&accessor);
            debug_assert!(num == 1);
            if num_bytes == 1 {
                if let Some(ind) = Self::read_accessor_from_path::<u8>(path, &accessor) {
                    indices = ind.iter().map(|e| *e as u32).collect();
                }
            } else if num_bytes == 2 {
                if let Some(ind) = Self::read_accessor_from_path::<u16>(path, &accessor) {
                    indices = ind.iter().map(|e| *e as u32).collect();
                }
            } else if let Some(ind) = Self::read_accessor_from_path::<u32>(path, &accessor) {
                indices = ind;
            }
        }
        indices
    }

    fn extract_mesh_data(path: &Path, primitive: &Primitive) -> Vec<VertexData> {
        let mut vertices = Vec::new();
        for (_attribute_index, (semantic, accessor)) in primitive.attributes().enumerate() {
            //println!("Attribute[{}]: {:?}", _attribute_index, semantic);
            match semantic {
                Semantic::Positions => {
                    let num = Self::num_from_type(&accessor);
                    let num_bytes = Self::bytes_from_dimension(&accessor);
                    debug_assert!(num == 3 && num_bytes == 4);
                    if let Some(pos) = Self::read_accessor_from_path::<Vector3>(path, &accessor) {
                        if vertices.len() < pos.len() {
                            debug_assert!(vertices.is_empty());
                            for p in pos.iter() {
                                let v = VertexData {
                                    pos: *p,
                                    ..Default::default()
                                };
                                vertices.push(v);
                            }
                        } else {
                            debug_assert!(vertices.len() == pos.len());
                            for (i, p) in pos.iter().enumerate() {
                                vertices[i].pos = *p;
                            }
                        }
                    }
                }
                Semantic::Normals => {
                    let num = Self::num_from_type(&accessor);
                    let num_bytes = Self::bytes_from_dimension(&accessor);
                    debug_assert!(num == 3 && num_bytes == 4);
                    if let Some(norm) = Self::read_accessor_from_path::<Vector3>(path, &accessor) {
                        if vertices.len() < norm.len() {
                            debug_assert!(vertices.is_empty());
                            for n in norm.iter() {
                                let v = VertexData {
                                    normal: *n,
                                    ..Default::default()
                                };
                                vertices.push(v);
                            }
                        } else {
                            debug_assert!(vertices.len() == norm.len());
                            for (i, n) in norm.iter().enumerate() {
                                vertices[i].normal = *n;
                            }
                        }
                    }
                }
                Semantic::Colors(_color_index) => {
                    let num = Self::num_from_type(&accessor);
                    let num_bytes = Self::bytes_from_dimension(&accessor);
                    debug_assert!(num == 4 && num_bytes == 4);
                    if let Some(col) = Self::read_accessor_from_path::<Vector4>(path, &accessor) {
                        if vertices.len() < col.len() {
                            debug_assert!(vertices.is_empty());
                            for c in col.iter() {
                                let v = VertexData {
                                    color: *c,
                                    ..Default::default()
                                };
                                vertices.push(v);
                            }
                        } else {
                            debug_assert!(vertices.len() == col.len());
                            for (i, c) in col.iter().enumerate() {
                                vertices[i].color = *c;
                            }
                        }
                    }
                }
                Semantic::TexCoords(_texture_index) => {
                    let num = Self::num_from_type(&accessor);
                    let num_bytes = Self::bytes_from_dimension(&accessor);
                    debug_assert!(num == 2 && num_bytes == 4);
                    if let Some(tex) = Self::read_accessor_from_path::<Vector2>(path, &accessor) {
                        if !vertices.is_empty() {
                            for (i, v) in vertices.iter_mut().enumerate() {
                                v.tex_coord = tex[i];
                            }
                        } else {
                            debug_assert!(vertices.is_empty());
                            for t in tex.iter() {
                                let v = VertexData {
                                    tex_coord: *t,
                                    ..Default::default()
                                };
                                vertices.push(v);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        vertices
    }

    fn process_mesh_data(path: &Path, mesh_name: &str, primitive: &Primitive) -> PathBuf {
        let vertices = Self::extract_mesh_data(path, &primitive);
        let indices = Self::extract_indices(path, &primitive);
        let mut mesh_data = MeshData::default();
        mesh_data.append_mesh(vertices.as_slice(), indices.as_slice());

        Self::create_file(path, &mesh_data, mesh_name, MESH_DATA_EXTENSION)
    }
    fn process_material_data(path: &Path, primitive: &Primitive, mesh_path: PathBuf) -> PathBuf {
        let mut material_data = MaterialData::default();

        let mesh_path =
            convert_in_local_path(mesh_path.as_path(), PathBuf::from(DATA_FOLDER).as_path());
        material_data.meshes.push(mesh_path);

        let material = primitive.material().pbr_metallic_roughness();
        material_data.diffuse_color = material.base_color_factor().into();
        if let Some(texture) = material.base_color_texture() {
            match texture.texture().source().source() {
                ImageSource::Uri {
                    uri,
                    mime_type: _, /* fields */
                } => {
                    if let Some(parent_folder) = path.parent() {
                        let parent_path = parent_folder.to_str().unwrap().to_string();
                        let filepath = PathBuf::from(parent_path).join(uri);
                        let path = convert_in_local_path(
                            filepath.as_path(),
                            PathBuf::from(DATA_RAW_FOLDER).as_path(),
                        );
                        material_data.textures.push(path);
                    }
                }
                ImageSource::View {
                    view: _,
                    mime_type: _,
                } => {}
            }
        }
        let name = format!("Material_{}", primitive.material().index().unwrap());
        Self::create_file(
            path,
            &material_data,
            primitive.material().name().unwrap_or_else(|| name.as_str()),
            MATERIAL_DATA_EXTENSION,
        )
    }

    fn process_node(path: &Path, node: &Node, node_name: &str) -> PathBuf {
        let mut object_data = ObjectData {
            transform: node.transform().matrix().into(),
            ..Default::default()
        };

        if let Some(mesh) = node.mesh() {
            for (_primitive_index, primitive) in mesh.primitives().enumerate() {
                //println!("Primitive[{}]: ", _primitive_index);
                let name = format!("Mesh_{}", mesh.index());
                let mesh_path = Self::process_mesh_data(
                    path,
                    mesh.name().unwrap_or_else(|| name.as_str()),
                    &primitive,
                );
                let material_path = Self::process_material_data(path, &primitive, mesh_path);
                let material_path = convert_in_local_path(
                    material_path.as_path(),
                    PathBuf::from(DATA_FOLDER).as_path(),
                );
                object_data.material = material_path;
            }
        }

        for (_child_index, child) in node.children().enumerate() {
            let name = format!("Node_{}", child.index());
            let object_path =
                Self::process_node(path, &child, child.name().unwrap_or_else(|| name.as_str()));
            let object_path =
                convert_in_local_path(object_path.as_path(), PathBuf::from(DATA_FOLDER).as_path());
            object_data.children.push(object_path);
        }

        Self::create_file(path, &object_data, node_name, OBJECT_DATA_EXTENSION)
    }

    fn process_path(path: &Path) {
        if let Ok(gltf) = Gltf::open(path) {
            for scene in gltf.scenes() {
                for node in scene.nodes() {
                    if node.index() == 0 {
                        Self::process_node(
                            path,
                            &node,
                            path.parent()
                                .unwrap()
                                .file_stem()
                                .unwrap()
                                .to_str()
                                .unwrap(),
                        );
                    } else {
                        let name = format!("Node_{}", node.index());
                        Self::process_node(
                            path,
                            &node,
                            node.name().unwrap_or_else(|| name.as_str()),
                        );
                    }
                }
            }
        }
    }

    fn create_file<T>(path: &Path, mesh_data: &T, new_name: &str, new_extension: &str) -> PathBuf
    where
        T: nrg_serialize::Serialize,
    {
        let filename = path.file_name().unwrap().to_str().unwrap();
        let destination_ext = format!("{}.{}", new_name, new_extension);
        let mut from_source_to_compiled = path.to_str().unwrap().to_string();
        from_source_to_compiled = from_source_to_compiled.replace(
            PathBuf::from(DATA_RAW_FOLDER)
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap(),
            PathBuf::from(DATA_FOLDER)
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap(),
        );
        from_source_to_compiled =
            from_source_to_compiled.replace(filename, destination_ext.as_str());

        let new_path = PathBuf::from(from_source_to_compiled);
        if !new_path.exists() {
            let result = create_dir_all(new_path.parent().unwrap());
            debug_assert!(result.is_ok());
        }
        if need_to_binarize(path, new_path.as_path()) {
            println!("Serializing {:?}", new_path);
            serialize_to_file(mesh_data, new_path.clone());
        }
        new_path
    }
}

impl ExtensionHandler for GltfCompiler {
    fn on_changed(&mut self, path: &Path) {
        if let Some(ext) = path.extension() {
            let extension = ext.to_str().unwrap().to_string();
            if extension.as_str() == GLTF_EXTENSION {
                Self::process_path(path);
            }
        }
    }
}
