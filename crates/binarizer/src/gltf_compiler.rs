use std::{
    fs::{self, create_dir_all, File},
    io::{Seek, SeekFrom},
    path::{Path, PathBuf},
};

use crate::{need_to_binarize, ExtensionHandler};
use gltf::{
    accessor::{DataType, Dimensions},
    buffer::{Source, View},
    camera::Projection,
    image::Source as ImageSource,
    khr_lights_punctual::{Kind, Light},
    material::AlphaMode,
    mesh::Mode,
    Accessor, Camera, Gltf, Node, Primitive, Semantic, Texture,
};
use sabi_filesystem::convert_in_local_path;
use sabi_graphics::{
    LightData, LightType, MaterialAlphaMode, MaterialData, MeshCategoryId, MeshData, TextureType,
    VertexData, DEFAULT_MESH_CATEGORY_IDENTIFIER, MAX_TEXTURE_COORDS_SETS,
};
use sabi_math::{Mat4Ops, Matrix4, NewAngle, Parser, Radians, Vector2, Vector3, Vector4};
use sabi_messenger::MessengerRw;
use sabi_profiler::debug_log;
use sabi_resources::{DATA_FOLDER, DATA_RAW_FOLDER};
use sabi_scene::{CameraData, ObjectData, SceneData};
use sabi_serialize::{Serialize, SerializeFile};

const GLTF_EXTENSION: &str = "gltf";

const DEFAULT_PIPELINE: &str = "pipelines/Default.pipeline_data";

#[derive(PartialEq, Eq)]
enum NodeType {
    Object,
    Camera,
    Light,
}

pub struct GltfCompiler {
    global_messenger: MessengerRw,
}

impl GltfCompiler {
    pub fn new(global_messenger: MessengerRw) -> Self {
        Self { global_messenger }
    }

    fn num_from_type(&mut self, accessor: &Accessor) -> usize {
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
    fn bytes_from_dimension(&mut self, accessor: &Accessor) -> usize {
        match accessor.data_type() {
            DataType::F32 | DataType::U32 => 4,
            DataType::U16 | DataType::I16 => 2,
            _ => 1,
        }
    }

    fn read_accessor_from_path<T>(&mut self, path: &Path, accessor: &Accessor) -> Option<Vec<T>>
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
                            return Some(self.read_from_file::<T>(&mut file, &view, accessor));
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

    fn read_from_file<T>(&mut self, file: &mut File, view: &View, accessor: &Accessor) -> Vec<T>
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

    fn extract_indices(&mut self, path: &Path, primitive: &Primitive) -> Vec<u32> {
        let mut indices = Vec::new();
        debug_assert!(primitive.mode() == Mode::Triangles);
        if let Some(accessor) = primitive.indices() {
            let num = self.num_from_type(&accessor);
            let num_bytes = self.bytes_from_dimension(&accessor);
            debug_assert!(num == 1);
            if num_bytes == 1 {
                if let Some(ind) = self.read_accessor_from_path::<u8>(path, &accessor) {
                    indices = ind.iter().map(|e| *e as u32).collect();
                }
            } else if num_bytes == 2 {
                if let Some(ind) = self.read_accessor_from_path::<u16>(path, &accessor) {
                    indices = ind.iter().map(|e| *e as u32).collect();
                }
            } else if let Some(ind) = self.read_accessor_from_path::<u32>(path, &accessor) {
                indices = ind;
            }
        }
        indices
    }

    fn extract_mesh_data(&mut self, path: &Path, primitive: &Primitive) -> Vec<VertexData> {
        let mut vertices = Vec::new();
        for (_attribute_index, (semantic, accessor)) in primitive.attributes().enumerate() {
            //debug_log("Attribute[{}]: {:?}", _attribute_index, semantic);
            match semantic {
                Semantic::Positions => {
                    let num = self.num_from_type(&accessor);
                    let num_bytes = self.bytes_from_dimension(&accessor);
                    debug_assert!(num == 3 && num_bytes == 4);
                    if let Some(pos) = self.read_accessor_from_path::<Vector3>(path, &accessor) {
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
                    let num = self.num_from_type(&accessor);
                    let num_bytes = self.bytes_from_dimension(&accessor);
                    debug_assert!(num == 3 && num_bytes == 4);
                    if let Some(norm) = self.read_accessor_from_path::<Vector3>(path, &accessor) {
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
                Semantic::Tangents => {
                    let num = self.num_from_type(&accessor);
                    let num_bytes = self.bytes_from_dimension(&accessor);
                    debug_assert!(num == 4 && num_bytes == 4);
                    if let Some(tang) = self.read_accessor_from_path::<Vector4>(path, &accessor) {
                        if vertices.len() < tang.len() {
                            debug_assert!(vertices.is_empty());
                            for t in tang.iter() {
                                let v = VertexData {
                                    tangent: [t.x, t.y, t.z].into(),
                                    ..Default::default()
                                };
                                vertices.push(v);
                            }
                        } else {
                            debug_assert!(vertices.len() == tang.len());
                            for (i, t) in tang.iter().enumerate() {
                                vertices[i].tangent = [t.x, t.y, t.z].into();
                            }
                        }
                    }
                }
                Semantic::Colors(_color_index) => {
                    let num = self.num_from_type(&accessor);
                    let num_bytes = self.bytes_from_dimension(&accessor);
                    debug_assert!(num == 4 && num_bytes == 4);
                    if let Some(col) = self.read_accessor_from_path::<Vector4>(path, &accessor) {
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
                Semantic::TexCoords(texture_index) => {
                    if texture_index >= MAX_TEXTURE_COORDS_SETS as _ {
                        eprintln!(
                            "ERROR: Texture coordinate set {} is out of range (max {})",
                            texture_index, MAX_TEXTURE_COORDS_SETS
                        );
                        continue;
                    }
                    let num = self.num_from_type(&accessor);
                    let num_bytes = self.bytes_from_dimension(&accessor);
                    debug_assert!(num == 2 && num_bytes == 4);
                    if let Some(tex) = self.read_accessor_from_path::<Vector2>(path, &accessor) {
                        if !vertices.is_empty() {
                            for (i, v) in vertices.iter_mut().enumerate() {
                                v.tex_coord[texture_index as usize] = tex[i];
                            }
                        } else {
                            debug_assert!(vertices.is_empty());
                            for t in tex.iter() {
                                let mut v = VertexData::default();
                                v.tex_coord[texture_index as usize] = *t;
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

    fn process_mesh_data(
        &mut self,
        path: &Path,
        mesh_name: &str,
        primitive: &Primitive,
        material_path: &Path,
    ) -> PathBuf {
        let vertices = self.extract_mesh_data(path, primitive);
        let indices = self.extract_indices(path, primitive);
        let mut mesh_data = MeshData::default();
        mesh_data.append_mesh(vertices.as_slice(), indices.as_slice());
        mesh_data.material = material_path.to_path_buf();
        mesh_data.mesh_category_identifier = MeshCategoryId::new(DEFAULT_MESH_CATEGORY_IDENTIFIER);

        Self::create_file(path, &mesh_data, mesh_name)
    }
    fn process_texture(&mut self, path: &Path, texture: Texture) -> PathBuf {
        if let ImageSource::Uri {
            uri,
            mime_type: _, /* fields */
        } = texture.source().source()
        {
            if let Some(parent_folder) = path.parent() {
                let parent_path = parent_folder.to_str().unwrap().to_string();
                let filepath = PathBuf::from(parent_path).join(uri);
                let path = convert_in_local_path(
                    filepath.as_path(),
                    PathBuf::from(DATA_RAW_FOLDER).as_path(),
                );
                return path;
            }
        }
        PathBuf::new()
    }
    fn process_material_data(&mut self, path: &Path, primitive: &Primitive) -> PathBuf {
        let mut material_data = MaterialData::default();

        let material = primitive.material().pbr_metallic_roughness();
        material_data.base_color = material.base_color_factor().into();
        material_data.roughness_factor = material.roughness_factor();
        material_data.metallic_factor = material.metallic_factor();
        material_data.pipeline = PathBuf::from(DEFAULT_PIPELINE);
        if let Some(info) = material.base_color_texture() {
            material_data.textures[TextureType::BaseColor as usize] =
                self.process_texture(path, info.texture());
            material_data.texcoords_set[TextureType::BaseColor as usize] = info.tex_coord() as _;
        }
        if let Some(info) = material.metallic_roughness_texture() {
            material_data.textures[TextureType::MetallicRoughness as usize] =
                self.process_texture(path, info.texture());
            material_data.texcoords_set[TextureType::MetallicRoughness as usize] =
                info.tex_coord() as _;
        }

        let material = primitive.material();
        if let Some(texture) = material.normal_texture() {
            material_data.textures[TextureType::Normal as usize] =
                self.process_texture(path, texture.texture());
            material_data.texcoords_set[TextureType::Normal as usize] = texture.tex_coord() as _;
        }
        if let Some(texture) = material.emissive_texture() {
            material_data.textures[TextureType::Emissive as usize] =
                self.process_texture(path, texture.texture());
            material_data.texcoords_set[TextureType::Emissive as usize] = texture.tex_coord() as _;
        }
        if let Some(texture) = material.occlusion_texture() {
            material_data.textures[TextureType::Occlusion as usize] =
                self.process_texture(path, texture.texture());
            material_data.texcoords_set[TextureType::Occlusion as usize] = texture.tex_coord() as _;
        }
        material_data.alpha_mode = match material.alpha_mode() {
            AlphaMode::Opaque => MaterialAlphaMode::Opaque,
            AlphaMode::Mask => {
                material_data.alpha_cutoff = 0.5;
                MaterialAlphaMode::Mask
            }
            AlphaMode::Blend => MaterialAlphaMode::Blend,
        };
        material_data.alpha_cutoff = primitive.material().alpha_cutoff().unwrap_or(1.);
        material_data.emissive_color = [
            primitive.material().emissive_factor()[0],
            primitive.material().emissive_factor()[1],
            primitive.material().emissive_factor()[2],
            1.,
        ]
        .into();
        if let Some(material) = material.pbr_specular_glossiness() {
            if let Some(texture) = material.specular_glossiness_texture() {
                material_data.textures[TextureType::SpecularGlossiness as usize] =
                    self.process_texture(path, texture.texture());
                material_data.texcoords_set[TextureType::SpecularGlossiness as usize] =
                    texture.tex_coord() as _;
            }
            if let Some(texture) = material.diffuse_texture() {
                material_data.textures[TextureType::Diffuse as usize] =
                    self.process_texture(path, texture.texture());
                material_data.texcoords_set[TextureType::Diffuse as usize] =
                    texture.tex_coord() as _;
            }
            material_data.diffuse_color = material.diffuse_factor().into();
            material_data.specular_color = [
                material.specular_factor()[0],
                material.specular_factor()[1],
                material.specular_factor()[2],
                1.,
            ]
            .into();
        }

        let name = format!(
            "Material_{}",
            primitive.material().index().unwrap_or_default()
        );
        Self::create_file(
            path,
            &material_data,
            primitive.material().name().unwrap_or_else(|| name.as_str()),
        )
    }

    fn process_node(
        &mut self,
        path: &Path,
        node: &Node,
        node_name: &str,
    ) -> Option<(NodeType, PathBuf)> {
        Some(self.process_object(path, node, node_name))
    }

    fn process_object(&mut self, path: &Path, node: &Node, node_name: &str) -> (NodeType, PathBuf) {
        let mut object_data = ObjectData::default();
        let object_transform: Matrix4 = Matrix4::from(node.transform().matrix());
        object_data.transform = object_transform;

        if let Some(mesh) = node.mesh() {
            for (_primitive_index, primitive) in mesh.primitives().enumerate() {
                //debug_log("Primitive[{}]: ", _primitive_index);
                let name = format!("Mesh_{}", mesh.index());
                let material_path = self.process_material_data(path, &primitive);
                let material_path = convert_in_local_path(
                    material_path.as_path(),
                    PathBuf::from(DATA_FOLDER).as_path(),
                );
                let mesh_path = self.process_mesh_data(
                    path,
                    mesh.name().unwrap_or_else(|| name.as_str()),
                    &primitive,
                    material_path.as_path(),
                );
                let mesh_path = convert_in_local_path(
                    mesh_path.as_path(),
                    PathBuf::from(DATA_FOLDER).as_path(),
                );
                object_data.components.push(mesh_path);
            }
        }
        if let Some(camera) = node.camera() {
            let position = object_data.transform.translation();
            let mut matrix =
                Matrix4::from_nonuniform_scale(1., 1., -1.) * object_data.transform.inverse();
            matrix.set_translation(position);
            object_data.transform = matrix;
            let (_, camera_path) = self.process_camera(path, &camera);
            object_data.components.push(convert_in_local_path(
                camera_path.as_path(),
                PathBuf::from(DATA_FOLDER).as_path(),
            ));
        }
        if let Some(light) = node.light() {
            let (_, light_path) = self.process_light(path, &light);
            object_data.components.push(convert_in_local_path(
                light_path.as_path(),
                PathBuf::from(DATA_FOLDER).as_path(),
            ));
        }

        for (_child_index, child) in node.children().enumerate() {
            let name = format!("Node_{}", child.index());
            if let Some(camera) = child.camera() {
                object_data.transform =
                    object_data.transform * Matrix4::from(child.transform().matrix());
                let position = object_data.transform.translation();
                let mut matrix =
                    Matrix4::from_nonuniform_scale(1., 1., -1.) * object_data.transform.inverse();
                matrix.set_translation(position);
                object_data.transform = matrix;
                let (_, camera_path) = self.process_camera(path, &camera);
                object_data.components.push(convert_in_local_path(
                    camera_path.as_path(),
                    PathBuf::from(DATA_FOLDER).as_path(),
                ));
            } else if let Some((node_type, node_path)) =
                self.process_node(path, &child, child.name().unwrap_or_else(|| name.as_str()))
            {
                if node_type == NodeType::Object {
                    let node_path = convert_in_local_path(
                        node_path.as_path(),
                        PathBuf::from(DATA_FOLDER).as_path(),
                    );
                    object_data.children.push(node_path);
                }
            }
        }

        (
            NodeType::Object,
            Self::create_file(path, &object_data, node_name),
        )
    }

    fn process_light(&mut self, path: &Path, light: &Light) -> (NodeType, PathBuf) {
        let mut light_data = LightData {
            color: [light.color()[0], light.color()[1], light.color()[2], 1.],
            intensity: light.intensity(),
            range: light.range().unwrap_or(1.),
            ..Default::default()
        };
        match light.kind() {
            Kind::Directional => {
                light_data.light_type = LightType::Directional as _;
            }
            Kind::Point => {
                light_data.light_type = LightType::Point as _;
            }
            Kind::Spot {
                inner_cone_angle,
                outer_cone_angle,
            } => {
                light_data.light_type = LightType::Spot as _;
                light_data.inner_cone_angle = inner_cone_angle;
                light_data.outer_cone_angle = outer_cone_angle;
            }
        }

        let name = format!("Light_{}", light.index());
        (NodeType::Light, Self::create_file(path, &light_data, &name))
    }

    fn process_camera(&mut self, path: &Path, camera: &Camera) -> (NodeType, PathBuf) {
        let mut camera_data = CameraData::default();
        match camera.projection() {
            Projection::Perspective(p) => {
                camera_data.aspect_ratio = p.aspect_ratio().unwrap_or(1920. / 1080.);
                camera_data.near = p.znear();
                camera_data.far = p.zfar().unwrap_or(camera_data.near + 1000.);
                camera_data.fov = Radians::new(p.yfov()).into();
            }
            Projection::Orthographic(o) => {
                camera_data.near = o.znear();
                camera_data.far = o.zfar();
            }
        }
        let name = format!("Camera_{}", camera.index());

        (
            NodeType::Camera,
            Self::create_file(path, &camera_data, &name),
        )
    }

    fn process_path(&mut self, path: &Path) {
        if let Ok(gltf) = Gltf::open(path) {
            for scene in gltf.scenes() {
                let mut scene_data = SceneData::default();
                let scene_name = path
                    .parent()
                    .unwrap()
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap();

                for node in scene.nodes() {
                    let name = format!("Node_{}", node.index());
                    if let Some((node_type, node_path)) =
                        self.process_node(path, &node, node.name().unwrap_or_else(|| name.as_str()))
                    {
                        match node_type {
                            NodeType::Camera => {
                                scene_data.cameras.push(node_path);
                            }
                            NodeType::Object => {
                                scene_data.objects.push(node_path);
                            }
                            NodeType::Light => {
                                scene_data.lights.push(node_path);
                            }
                        }
                    }
                }

                Self::create_file(path, &scene_data, scene_name);
            }
        }
    }

    fn create_file<T>(path: &Path, data: &T, new_name: &str) -> PathBuf
    where
        T: Serialize + SerializeFile,
    {
        let filename = path.file_name().unwrap().to_str().unwrap();
        let destination_ext = format!("{}.{}", new_name, T::extension());
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
            debug_log(format!("Serializing {:?}", new_path).as_str());
            data.save_to_file(new_path.as_path());
        }
        new_path
    }
}

impl ExtensionHandler for GltfCompiler {
    fn on_changed(&mut self, path: &Path) {
        if let Some(ext) = path.extension() {
            let extension = ext.to_str().unwrap().to_string();
            if extension.as_str() == GLTF_EXTENSION {
                self.process_path(path);
            }
        }
    }
}
