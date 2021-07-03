use std::{
    fs::{self, create_dir_all, File},
    io::{Seek, SeekFrom},
    path::{Path, PathBuf},
};

use crate::{need_to_binarize, ExtensionHandler, Parser};
use gltf::{
    accessor::{DataType, Dimensions},
    buffer::{Source, View},
    Accessor, Gltf, Primitive, Semantic,
};
use nrg_graphics::{MeshData, VertexData};
use nrg_math::{Vector2, Vector3, Vector4};
use nrg_messenger::MessengerRw;
use nrg_resources::{DATA_FOLDER, DATA_RAW_FOLDER};
use nrg_serialize::serialize_to_file;

const GLTF_EXTENSION: &str = "gltf";
const MESH_DATA_EXTENSION: &str = "mesh_data";

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

    fn get_count(accessor: &Accessor) -> usize {
        let count = accessor.count();
        let num = Self::num_from_type(accessor);
        count / num
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
                            return Self::read_from_file::<T>(&mut file, &view, &accessor);
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

    fn read_from_file<T>(file: &mut File, view: &View, accessor: &Accessor) -> Option<Vec<T>>
    where
        T: Parser,
    {
        let count = Self::get_count(&accessor);
        let view_offset = view.offset();
        let accessor_offset = accessor.offset();
        let starting_offset = view_offset + accessor_offset;
        let view_stride = view.stride().unwrap_or(0);
        if file.seek(SeekFrom::Start((starting_offset) as _)).is_ok() {
            let mut result = Vec::new();
            for _i in 0..count {
                let v = T::parse(file);
                result.push(v);
                file.seek(SeekFrom::Current(view_stride as _)).ok();
            }
            return Some(result);
        }
        None
    }

    fn extract_indices(path: &Path, primitive: &Primitive) -> Vec<u32> {
        let mut indices = Vec::new();
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

    fn process_path(path: &Path) {
        if let Ok(gltf) = Gltf::open(path) {
            //println!("MeshCount: {:?}", gltf.meshes().len());
            for (_mesh_index, mesh) in gltf.meshes().enumerate() {
                //println!("Mesh[{}]: {:?}", _mesh_index, mesh.name());
                for (_primitive_index, primitive) in mesh.primitives().enumerate() {
                    //println!("Primitive[{}]: ", _primitive_index);
                    let vertices = Self::extract_mesh_data(path, &primitive);
                    let indices = Self::extract_indices(path, &primitive);
                    let mut mesh_data = MeshData::default();
                    mesh_data.append_mesh(vertices.as_slice(), indices.as_slice());
                    Self::create_mesh_file(path, &mesh_data);
                }
            }
        }
    }

    fn create_mesh_file(path: &Path, mesh_data: &MeshData) {
        let extension = path.extension().unwrap().to_str().unwrap();
        let source_ext = format!(".{}", extension);
        let destination_ext = format!(".{}", MESH_DATA_EXTENSION);
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
            from_source_to_compiled.replace(source_ext.as_str(), destination_ext.as_str());

        let new_path = PathBuf::from(from_source_to_compiled);
        if !new_path.exists() {
            let result = create_dir_all(new_path.parent().unwrap());
            debug_assert!(result.is_ok());
        }
        if need_to_binarize(path, new_path.as_path()) {
            println!("Converting mesh {:?}", new_path);
            serialize_to_file(mesh_data, new_path);
        }
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
