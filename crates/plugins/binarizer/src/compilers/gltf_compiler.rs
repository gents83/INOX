use std::{
    fs::{self, create_dir_all, File},
    io::{Seek, SeekFrom},
    path::{Path, PathBuf},
};

use crate::{
    mesh::{
        build_meshlets_adjacency, compute_clusters, compute_meshlets, create_mesh_data,
        group_meshlets, optimize_mesh, MeshVertex,
    },
    need_to_binarize, to_local_path, ExtensionHandler,
};
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

use inox_bvh::{create_linearized_bvh, BVHTree, AABB};
use inox_graphics::{
    LightData, LightType, MaterialData, MaterialFlags, TextureType, MAX_LOD_LEVELS,
    MAX_TEXTURE_COORDS_SETS,
};
use inox_log::debug_log;
use inox_math::{
    Mat4Ops, Matrix4, NewAngle, Parser, Radians, VecBase, Vector2, Vector3, Vector4, Vector4h,
};

use inox_nodes::LogicData;
use inox_resources::SharedDataRc;
use inox_scene::{CameraData, ObjectData, SceneData};
use inox_serialize::{
    deserialize, inox_serializable::SerializableRegistryRc, Deserialize, Serialize, SerializeFile,
};
use mikktspace::{generate_tangents, Geometry};

const GLTF_EXTENSION: &str = "gltf";

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "inox_serialize")]
struct ExtraData {
    name: String,
    #[serde(rename = "type")]
    typename: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "inox_serialize")]
struct ExtraProperties {
    logic: ExtraData,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "inox_serialize")]
struct Extras {
    inox_properties: ExtraProperties,
}

#[derive(PartialEq, Eq)]
enum NodeType {
    Object,
    Camera,
    Light,
}
struct GltfGeometry {
    vertices: Vec<MeshVertex>,
    indices: Vec<u32>,
}

impl Geometry for GltfGeometry {
    fn num_faces(&self) -> usize {
        self.indices.len() / 3
    }

    fn num_vertices_of_face(&self, _face: usize) -> usize {
        3
    }

    fn position(&self, face: usize, vert: usize) -> [f32; 3] {
        self.vertices[self.indices[face * 3 + vert] as usize]
            .pos
            .into()
    }

    fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
        self.vertices[self.indices[face * 3 + vert] as usize]
            .normal
            .unwrap_or_else(Vector3::default_zero)
            .into()
    }

    fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
        self.vertices[self.indices[face * 3 + vert] as usize]
            .uv_0
            .unwrap_or_else(Vector2::default_zero)
            .into()
    }
}

#[derive(Default)]
pub struct GltfCompiler {
    shared_data: SharedDataRc,
    data_raw_folder: PathBuf,
    data_folder: PathBuf,
    node_index: usize,
    material_index: usize,
}

impl GltfCompiler {
    pub fn new(shared_data: SharedDataRc, data_raw_folder: &Path, data_folder: &Path) -> Self {
        Self {
            shared_data,
            data_raw_folder: data_raw_folder.to_path_buf(),
            data_folder: data_folder.to_path_buf(),
            node_index: 0,
            material_index: 0,
        }
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
                            eprintln!("Unable to open file: {local_path}");
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
        debug_assert!(primitive.mode() == Mode::Triangles);
        let mut indices = Vec::new();
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

    fn extract_vertices(&mut self, path: &Path, primitive: &Primitive) -> Vec<MeshVertex> {
        let mut vertices = Vec::new();

        primitive.attributes().enumerate().for_each(|(_attribute_index, (semantic, accessor))| {
            //debug_log!("Attribute[{}]: {:?}", _attribute_index, semantic);
            match semantic {
                Semantic::Positions => {
                    let num = self.num_from_type(&accessor);
                    let num_bytes = self.bytes_from_dimension(&accessor);
                    debug_assert!(num == 3 && num_bytes == 4);
                    if let Some(pos) = self.read_accessor_from_path::<Vector3>(path, &accessor) {
                        if vertices.is_empty() {
                            vertices.resize(pos.len(), MeshVertex::default());
                        }
                        pos.iter().enumerate().for_each(|(i, v)| {
                            vertices[i].pos = *v;
                        });
                    }
                }
                Semantic::Normals => {
                    let num = self.num_from_type(&accessor);
                    let num_bytes = self.bytes_from_dimension(&accessor);
                    debug_assert!(num == 3 && num_bytes == 4);
                    if let Some(norm) = self.read_accessor_from_path::<Vector3>(path, &accessor) {
                        if vertices.is_empty() {
                            vertices.resize(norm.len(), MeshVertex::default());
                        }
                        norm.iter().enumerate().for_each(|(i, v)| {
                            vertices[i].normal = Some(*v);
                        });
                    }
                }
                Semantic::Tangents => {
                    let num = self.num_from_type(&accessor);
                    let num_bytes = self.bytes_from_dimension(&accessor);
                    debug_assert!(num == 4);
                    if num_bytes == 2 {
                        debug_assert!(num_bytes == 2);
                        if let Some(tan) = self.read_accessor_from_path::<Vector4h>(path, &accessor)
                        {
                            if vertices.is_empty() {
                                vertices.resize(tan.len(), MeshVertex::default());
                            }
                            tan.iter().enumerate().for_each(|(i, v)| {
                                vertices[i].tangent =
                                    Some([v.x as f32, v.y as f32, v.z as f32, v.z as f32].into());
                            });
                        }
                    } else {
                        debug_assert!(num_bytes == 4);
                        if let Some(tan) = self.read_accessor_from_path::<Vector4>(path, &accessor)
                        {
                            tan.iter().enumerate().for_each(|(i, v)| {
                                vertices[i].tangent = Some(*v);
                            });
                        }
                    }
                }
                Semantic::Colors(_color_index) => {
                    let num = self.num_from_type(&accessor);
                    let num_bytes = self.bytes_from_dimension(&accessor);
                    debug_assert!(num == 4);
                    if num_bytes == 2 {
                        debug_assert!(num_bytes == 2);
                        if let Some(col) = self.read_accessor_from_path::<Vector4h>(path, &accessor)
                        {
                            if vertices.is_empty() {
                                vertices.resize(col.len(), MeshVertex::default());
                            }
                            col.iter().enumerate().for_each(|(i, v)| {
                                vertices[i].color =
                                    Some([v.x as f32, v.y as f32, v.z as f32, v.z as f32].into());
                            });
                        }
                    } else {
                        debug_assert!(num_bytes == 4);
                        if let Some(col) = self.read_accessor_from_path::<Vector4>(path, &accessor)
                        {
                            col.iter().enumerate().for_each(|(i, v)| {
                                vertices[i].color = Some(*v);
                            });
                        }
                    }
                }
                Semantic::TexCoords(texture_index) => {
                    let num = self.num_from_type(&accessor);
                    let num_bytes = self.bytes_from_dimension(&accessor);
                    let max = if let Some(max) = accessor.max() {
                        max.as_array().unwrap().iter().map(|v| v.as_f64().unwrap() as f32).collect()
                    } else {
                        vec![0.; 2]
                    };
                    let min = if let Some(min) = accessor.min() {
                        min.as_array().unwrap().iter().map(|v| v.as_f64().unwrap() as f32).collect()
                    } else {
                        vec![0.; 2]
                    };
                    let min: Vector2 = Vector2::new(min[0], min[1]);
                    let max: Vector2 = Vector2::new(max[0], max[1]);
                    debug_assert!(num == 2 && num_bytes == 4);
                    if let Some(tex) = self.read_accessor_from_path::<Vector2>(path, &accessor) {
                        if vertices.is_empty() {
                            vertices.resize(tex.len(), MeshVertex::default());
                        }
                        match texture_index {
                            0 => {
                                tex.iter().enumerate().for_each(|(i, v)| {
                                    vertices[i].uv_0 = Some(v - (max - min));
                                });
                            }
                            1 => {
                                tex.iter().enumerate().for_each(|(i, v)| {
                                    vertices[i].uv_1 = Some(v - (max - min));
                                });
                            }
                            2 => {
                                tex.iter().enumerate().for_each(|(i, v)| {
                                    vertices[i].uv_2 = Some(v - (max - min));
                                });
                            }
                            3 => {
                                tex.iter().enumerate().for_each(|(i, v)| {
                                    vertices[i].uv_3 = Some(v - (max - min));
                                });
                            }
                            _ => {
                                eprintln!(
                                "ERROR: Texture coordinate set {texture_index} is out of range (max {MAX_TEXTURE_COORDS_SETS})",
                            );
                            }
                        }
                    }
                }
                _ => {}
            }
        });
        vertices
    }

    fn process_mesh_data(
        &mut self,
        path: &Path,
        mesh_name: &str,
        primitive: &Primitive,
        material_path: &Path,
    ) -> PathBuf {
        let vertices = self.extract_vertices(path, primitive);
        let indices = self.extract_indices(path, primitive);
        let mut geometry = GltfGeometry { vertices, indices };
        generate_tangents(&mut geometry);

        let (mesh_vertices, geometry_indices) =
            optimize_mesh(&geometry.vertices, &geometry.indices);

        let mut meshlet_indices_offset = 0;
        let mut meshlets_per_lod = Vec::new();
        let (meshlets, mut mesh_indices) = compute_meshlets(&mesh_vertices, &geometry_indices);
        meshlets_per_lod.push(meshlets);
        meshlet_indices_offset += mesh_indices.len();

        let mut is_meshlet_tree_created = false;
        let mut level = 0;
        while !is_meshlet_tree_created {
            let previous_lod_meshlets = meshlets_per_lod.last_mut().unwrap();
            let meshlets_adjacency = build_meshlets_adjacency(previous_lod_meshlets, &mesh_indices);
            let groups = group_meshlets(&meshlets_adjacency);
            level += 1;
            let (mut cluster_indices, cluster_meshlets) = compute_clusters(
                &groups,
                previous_lod_meshlets,
                meshlet_indices_offset,
                &mesh_vertices,
                &mesh_indices,
            );

            meshlet_indices_offset += cluster_indices.len();
            mesh_indices.append(&mut cluster_indices);
            meshlets_per_lod.push(cluster_meshlets);

            is_meshlet_tree_created = groups.len() == 1 || level >= (MAX_LOD_LEVELS - 1);
        }

        let mut mesh_data = create_mesh_data(&mesh_vertices, &mesh_indices);
        mesh_data.meshlets = meshlets_per_lod;
        mesh_data.meshlets.reverse();
        mesh_data.meshlets_bvh.clear();
        mesh_data.material = material_path.to_path_buf();

        mesh_data
            .meshlets
            .iter()
            .enumerate()
            .for_each(|(lod_level, meshlets)| {
                println!("LOD {} has {} meshlets", lod_level, meshlets.len());

                let mut meshlets_aabbs = Vec::new();
                meshlets_aabbs.resize_with(meshlets.len(), AABB::empty);
                meshlets.iter().enumerate().for_each(|(i, m)| {
                    println!(
                        "LOD {} Meshlet {} has as children {:?}",
                        lod_level, i, m.child_meshlets
                    );
                    meshlets_aabbs[i] = AABB::create(m.aabb_min, m.aabb_max, i as _);
                });
                let bvh = BVHTree::new(&meshlets_aabbs);
                mesh_data.meshlets_bvh.push(create_linearized_bvh(&bvh));
            });

        self.create_file(
            path,
            &mesh_data,
            mesh_name,
            "mesh",
            self.shared_data.serializable_registry(),
        )
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
                let path = to_local_path(
                    filepath.as_path(),
                    self.data_raw_folder.as_path(),
                    self.data_folder.as_path(),
                );
                return path;
            }
        }
        PathBuf::new()
    }
    fn process_material_data(&mut self, path: &Path, primitive: &Primitive) -> PathBuf {
        let mut material_data = MaterialData::default();

        let material = primitive.material().pbr_metallic_roughness();
        material_data.flags = MaterialFlags::MetallicRoughness;
        //println!("Flags = MetallicRoughness");
        material_data.base_color = material.base_color_factor().into();
        material_data.roughness_factor = material.roughness_factor();
        material_data.metallic_factor = material.metallic_factor();
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
        if material.unlit() {
            material_data.flags |= MaterialFlags::Unlit;
            //println!("Flags |= Unlit");
        }
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
            material_data.occlusion_strength = texture.strength();
        };
        material_data.alpha_cutoff = 0.5;
        match material.alpha_mode() {
            AlphaMode::Opaque => {
                material_data.flags |= MaterialFlags::AlphaModeOpaque;
                //println!("Flags |= AlphaModeOpaque");
            }
            AlphaMode::Mask => {
                material_data.alpha_cutoff = 0.5;
                material_data.flags |= MaterialFlags::AlphaModeMask;
                //println!("Flags |= AlphaModeMask");
            }
            AlphaMode::Blend => {
                material_data.flags |= MaterialFlags::AlphaModeBlend;
                //println!("Flags |= AlphaModeBlend");
            }
        };
        material_data.alpha_cutoff = primitive.material().alpha_cutoff().unwrap_or(1.);
        material_data.emissive_color = [
            primitive.material().emissive_factor()[0],
            primitive.material().emissive_factor()[1],
            primitive.material().emissive_factor()[2],
        ]
        .into();
        if let Some(pbr) = material.pbr_specular_glossiness() {
            material_data.flags |= MaterialFlags::SpecularGlossiness;
            //println!("Flags |= SpecularGlossiness");
            if let Some(texture) = pbr.specular_glossiness_texture() {
                material_data.textures[TextureType::SpecularGlossiness as usize] =
                    self.process_texture(path, texture.texture());
                material_data.texcoords_set[TextureType::SpecularGlossiness as usize] =
                    texture.tex_coord() as _;
            }
            if let Some(texture) = pbr.diffuse_texture() {
                material_data.textures[TextureType::Diffuse as usize] =
                    self.process_texture(path, texture.texture());
                material_data.texcoords_set[TextureType::Diffuse as usize] =
                    texture.tex_coord() as _;
            }
            material_data.diffuse_factor = pbr.diffuse_factor().into();
            material_data.specular_glossiness_factor = [
                pbr.specular_factor()[0],
                pbr.specular_factor()[1],
                pbr.specular_factor()[2],
                pbr.glossiness_factor(),
            ]
            .into();
        }

        material_data.ior = if let Some(ior) = material.ior() {
            material_data.flags |= MaterialFlags::Ior;
            //println!("Flags |= Ior");
            ior
        } else {
            1.5
        };
        if let Some(specular) = material.specular() {
            material_data.flags |= MaterialFlags::Specular;
            //println!("Flags |= Specular");
            material_data.specular_factors = [
                specular.specular_color_factor()[0],
                specular.specular_color_factor()[1],
                specular.specular_color_factor()[2],
                specular.specular_factor(),
            ]
            .into();
            if let Some(texture) = specular.specular_texture() {
                material_data.textures[TextureType::Specular as usize] =
                    self.process_texture(path, texture.texture());
                material_data.texcoords_set[TextureType::Specular as usize] =
                    texture.tex_coord() as _;
            }
            if let Some(texture) = specular.specular_color_texture() {
                material_data.textures[TextureType::SpecularColor as usize] =
                    self.process_texture(path, texture.texture());
                material_data.texcoords_set[TextureType::SpecularColor as usize] =
                    texture.tex_coord() as _;
            }
        }
        if let Some(transmission) = material.transmission() {
            material_data.flags |= MaterialFlags::Transmission;
            //println!("Flags |= Transmission");
            material_data.transmission_factor = transmission.transmission_factor();
            if let Some(texture) = transmission.transmission_texture() {
                material_data.textures[TextureType::Transmission as usize] =
                    self.process_texture(path, texture.texture());
                material_data.texcoords_set[TextureType::Transmission as usize] =
                    texture.tex_coord() as _;
            }
        }
        if let Some(volume) = material.volume() {
            material_data.flags |= MaterialFlags::Volume;
            //println!("Flags |= Volume");
            material_data.attenuation_color_and_distance = [
                volume.attenuation_color()[0],
                volume.attenuation_color()[1],
                volume.attenuation_color()[2],
                volume.attenuation_distance(),
            ]
            .into();
            if let Some(texture) = volume.thickness_texture() {
                material_data.textures[TextureType::Thickness as usize] =
                    self.process_texture(path, texture.texture());
                material_data.texcoords_set[TextureType::Thickness as usize] =
                    texture.tex_coord() as _;
            }
        }

        //let flags: u32 = material_data.flags.into();
        //println!("Flags = {} = {:b}", flags, material_data.flags);
        let name = format!("Material_{}", self.material_index);
        self.create_file(
            path,
            &material_data,
            primitive.material().name().unwrap_or(&name),
            "material",
            self.shared_data.serializable_registry(),
        )
    }

    fn process_node(
        &mut self,
        path: &Path,
        node: &Node,
        node_name: &str,
    ) -> Option<(NodeType, PathBuf)> {
        let (node_type, node_path) = self.process_object(path, node, node_name);
        self.node_index += 1;
        Some((node_type, node_path))
    }

    fn process_object(&mut self, path: &Path, node: &Node, node_name: &str) -> (NodeType, PathBuf) {
        let mut object_data = ObjectData::default();
        let object_transform: Matrix4 = Matrix4::from(node.transform().matrix());
        object_data.transform = object_transform;

        if let Some(mesh) = node.mesh() {
            for (primitive_index, primitive) in mesh.primitives().enumerate() {
                let name = format!("{node_name}_Primitive_{primitive_index}");
                let material_path = self.process_material_data(path, &primitive);
                let material_path = to_local_path(
                    material_path.as_path(),
                    self.data_raw_folder.as_path(),
                    self.data_folder.as_path(),
                );
                let mesh_path =
                    self.process_mesh_data(path, &name, &primitive, material_path.as_path());
                let mesh_path = to_local_path(
                    mesh_path.as_path(),
                    self.data_raw_folder.as_path(),
                    self.data_folder.as_path(),
                );
                object_data.components.push(mesh_path);
            }
        }
        if let Some(camera) = node.camera() {
            object_data.transform = object_data.transform.inverse();
            let (_, camera_path) = self.process_camera(path, &camera);
            object_data.components.push(to_local_path(
                camera_path.as_path(),
                self.data_raw_folder.as_path(),
                self.data_folder.as_path(),
            ));
        }
        if let Some(light) = node.light() {
            let (_, light_path) = self.process_light(path, &light, &object_transform);
            object_data.components.push(to_local_path(
                light_path.as_path(),
                self.data_raw_folder.as_path(),
                self.data_folder.as_path(),
            ));
        }
        if let Some(extras) = node.extras() {
            if let Ok(extras) = deserialize::<Extras>(
                extras.to_string().as_str(),
                self.shared_data.serializable_registry(),
            ) {
                if !extras.inox_properties.logic.name.is_empty() {
                    let mut path = path
                        .parent()
                        .unwrap()
                        .join(LogicData::extension())
                        .to_str()
                        .unwrap()
                        .to_string();
                    path.push_str(
                        format!(
                            "\\{}.{}",
                            extras.inox_properties.logic.name,
                            LogicData::extension()
                        )
                        .as_str(),
                    );
                    object_data.components.push(to_local_path(
                        PathBuf::from(path).as_path(),
                        self.data_raw_folder.as_path(),
                        self.data_folder.as_path(),
                    ));
                }
            }
        }

        for (child_index, child) in node.children().enumerate() {
            let name = format!("Node_{}_Child_{}", self.node_index, child_index);
            if let Some(camera) = child.camera() {
                object_data.transform =
                    object_data.transform * Matrix4::from(child.transform().matrix());
                let (_, camera_path) = self.process_camera(path, &camera);
                object_data.components.push(to_local_path(
                    camera_path.as_path(),
                    self.data_raw_folder.as_path(),
                    self.data_folder.as_path(),
                ));
            } else if let Some((node_type, node_path)) =
                self.process_node(path, &child, child.name().unwrap_or(&name))
            {
                if node_type == NodeType::Object {
                    let node_path = to_local_path(
                        node_path.as_path(),
                        self.data_raw_folder.as_path(),
                        self.data_folder.as_path(),
                    );
                    object_data.children.push(node_path);
                }
            }
        }

        (
            NodeType::Object,
            self.create_file(
                path,
                &object_data,
                node_name,
                "object",
                self.shared_data.serializable_registry(),
            ),
        )
    }

    fn process_light(
        &mut self,
        path: &Path,
        light: &Light,
        transform: &Matrix4,
    ) -> (NodeType, PathBuf) {
        let mut light_data = LightData {
            color: [light.color()[0], light.color()[1], light.color()[2]],
            direction: (-transform.forward()).into(),
            position: transform.translation().into(),
            intensity: light.intensity(),
            range: light.range().unwrap_or(0.),
            ..Default::default()
        };
        match light.kind() {
            Kind::Directional => {
                light_data.light_type = LightType::Directional.into();
                light_data.range = -1.;
            }
            Kind::Point => {
                light_data.light_type = LightType::Point.into();
            }
            Kind::Spot {
                inner_cone_angle,
                outer_cone_angle,
            } => {
                light_data.light_type = LightType::Spot.into();
                light_data.inner_cone_angle = inner_cone_angle.cos();
                light_data.outer_cone_angle = outer_cone_angle.cos();
            }
        }

        let name = format!("Node_{}_Light_{}", self.node_index, light.index());
        (
            NodeType::Light,
            self.create_file(
                path,
                &light_data,
                &name,
                "light",
                self.shared_data.serializable_registry(),
            ),
        )
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
        let name = format!("Node_{}_Camera_{}", self.node_index, camera.index());

        (
            NodeType::Camera,
            self.create_file(
                path,
                &camera_data,
                &name,
                "camera",
                self.shared_data.serializable_registry(),
            ),
        )
    }

    pub fn process_path(&mut self, path: &Path) {
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

                let new_path = self.compute_path_name::<SceneData>(path, scene_name, "");
                if need_to_binarize(path, new_path.as_path()) {
                    self.material_index = 0;
                    self.node_index = 0;
                    for node in scene.nodes() {
                        let name = format!("Node_{}", self.node_index);
                        if let Some((node_type, node_path)) =
                            self.process_node(path, &node, node.name().unwrap_or(&name))
                        {
                            let node_path = to_local_path(
                                node_path.as_path(),
                                self.data_raw_folder.as_path(),
                                self.data_folder.as_path(),
                            );
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

                    self.create_file(
                        path,
                        &scene_data,
                        scene_name,
                        "",
                        self.shared_data.serializable_registry(),
                    );
                }
            }
        }
    }

    fn compute_path_name<T>(&self, path: &Path, new_name: &str, folder: &str) -> PathBuf
    where
        T: Serialize + SerializeFile + Clone + 'static,
    {
        let filename = path.file_name().unwrap().to_str().unwrap();
        let destination_ext = format!("{}.{}", new_name, T::extension());
        let mut filepath = path.parent().unwrap().to_path_buf();
        if !folder.is_empty() {
            filepath = filepath.join(folder);
        }
        filepath = filepath.join(filename);
        let mut from_source_to_compiled = filepath.to_str().unwrap().to_string();
        from_source_to_compiled = from_source_to_compiled.replace(
            self.data_raw_folder
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap(),
            self.data_folder.canonicalize().unwrap().to_str().unwrap(),
        );
        from_source_to_compiled =
            from_source_to_compiled.replace(filename, destination_ext.as_str());

        PathBuf::from(from_source_to_compiled)
    }

    fn create_file<T>(
        &self,
        path: &Path,
        data: &T,
        new_name: &str,
        folder: &str,
        serializable_registry: &SerializableRegistryRc,
    ) -> PathBuf
    where
        T: Serialize + SerializeFile + Clone + 'static,
    {
        let new_path = self.compute_path_name::<T>(path, new_name, folder);
        if !new_path.exists() {
            let result = create_dir_all(new_path.parent().unwrap());
            debug_assert!(result.is_ok());
        }
        if need_to_binarize(path, new_path.as_path()) {
            debug_log!("Serializing {:?}", new_path);
            data.save_to_file(new_path.as_path(), serializable_registry);
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
