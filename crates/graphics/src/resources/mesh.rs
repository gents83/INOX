use std::{
    ops::Range,
    path::{Path, PathBuf},
};

use crate::{
    GraphicsData, Material, MeshData, VertexFormat, VertexFormatBits, GRAPHICS_DATA_UID,
    INVALID_INDEX,
};
use inox_math::{MatBase, Matrix4, Vector4};
use inox_messenger::MessageHubRc;
use inox_resources::{
    DataTypeResource, Handle, Resource, ResourceEvent, ResourceId, ResourceTrait,
    SerializableResource, SharedData, SharedDataRc,
};
use inox_serialize::{inox_serializable::SerializableRegistryRc, read_from_file, SerializeFile};

pub type MeshId = ResourceId;

#[derive(Clone)]
pub struct OnMeshCreateData {
    pub parent_matrix: Matrix4,
}

#[derive(Clone)]
pub struct Mesh {
    id: MeshId,
    message_hub: MessageHubRc,
    shared_data: SharedDataRc,
    path: PathBuf,
    matrix: Matrix4,
    material: Handle<Material>,
    draw_area: Vector4, //pos (x,y) - size(z,w)
    is_visible: bool,
    draw_index: i32,
    graphics_mesh: Handle<GraphicsData>,
    vertex_format: VertexFormatBits,
    vertices_range: Range<usize>,
    indices_range: Range<usize>,
}

impl ResourceTrait for Mesh {
    type OnCreateData = OnMeshCreateData;

    fn on_create(
        &mut self,
        _shared_data_rc: &SharedDataRc,
        _message_hub: &MessageHubRc,
        _id: &MeshId,
        on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    ) {
        if let Some(on_create_data) = on_create_data {
            self.set_matrix(on_create_data.parent_matrix);
        }
    }
    fn on_destroy(&mut self, _shared_data: &SharedData, _message_hub: &MessageHubRc, id: &MeshId) {
        if let Some(graphics_mesh) = &self.graphics_mesh {
            graphics_mesh.get_mut().remove_mesh(id);
        }
    }
    fn on_copy(&mut self, other: &Self)
    where
        Self: Sized,
    {
        *self = other.clone();
    }
}

impl SerializableResource for Mesh {
    fn set_path(&mut self, path: &Path) -> &mut Self {
        self.path = path.to_path_buf();
        self
    }
    fn path(&self) -> &Path {
        self.path.as_path()
    }

    fn extension() -> &'static str {
        MeshData::extension()
    }
}

impl DataTypeResource for Mesh {
    type DataType = MeshData;
    type OnCreateData = <Self as ResourceTrait>::OnCreateData;

    fn new(id: ResourceId, shared_data: &SharedDataRc, message_hub: &MessageHubRc) -> Self {
        Self {
            id,
            shared_data: shared_data.clone(),
            message_hub: message_hub.clone(),
            path: PathBuf::new(),
            matrix: Matrix4::default_identity(),
            material: None,
            draw_area: [0., 0., f32::MAX, f32::MAX].into(),
            is_visible: true,
            draw_index: INVALID_INDEX,
            graphics_mesh: shared_data.get_resource::<GraphicsData>(&GRAPHICS_DATA_UID),
            vertices_range: 0..0,
            indices_range: 0..0,
            vertex_format: VertexFormat::to_bits(&VertexFormat::pbr()),
        }
    }
    fn is_initialized(&self) -> bool {
        !self.vertices_range.is_empty()
    }

    fn invalidate(&mut self) -> &mut Self {
        self.vertices_range = 0..0;
        self.indices_range = 0..0;
        self.mark_as_dirty();
        self
    }
    fn deserialize_data(
        path: &std::path::Path,
        registry: &SerializableRegistryRc,
        f: Box<dyn FnMut(Self::DataType) + 'static>,
    ) {
        read_from_file::<Self::DataType>(path, registry, f);
    }

    fn create_from_data(
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        id: ResourceId,
        data: Self::DataType,
    ) -> Self
    where
        Self: Sized,
    {
        let material = if !data.material.to_str().unwrap_or_default().is_empty() {
            let material =
                Material::request_load(shared_data, message_hub, data.material.as_path(), None);
            Some(material)
        } else {
            None
        };
        let mut mesh = Mesh::new(id, shared_data, message_hub);
        mesh.material = material;
        mesh.vertex_format = VertexFormat::to_bits(data.vertex_format.as_slice());
        if let Some(graphics_mesh) = &mesh.graphics_mesh {
            let (vertex_range, index_range) = graphics_mesh.get_mut().add_mesh_data(&id, &data);
            mesh.vertices_range = vertex_range;
            mesh.indices_range = index_range;
        }
        mesh
    }
}

impl Mesh {
    pub fn mark_as_dirty(&self) -> &Self {
        self.message_hub
            .send_event(ResourceEvent::<Self>::Changed(self.id));
        self
    }
    pub fn vertex_format(&self) -> u32 {
        self.vertex_format
    }
    pub fn find_from_path(shared_data: &SharedDataRc, path: &Path) -> Handle<Self> {
        SharedData::match_resource(shared_data, |m: &Mesh| m.path() == path)
    }
    pub fn vertices_range(&self) -> &Range<usize> {
        &self.vertices_range
    }
    pub fn set_vertices_range(&mut self, vertices_range: Range<usize>) -> &mut Self {
        if self.vertices_range != vertices_range {
            self.vertices_range = vertices_range;
            self.mark_as_dirty();
        }
        self
    }
    pub fn indices_range(&self) -> &Range<usize> {
        &self.indices_range
    }
    pub fn set_indices_range(&mut self, indices_range: Range<usize>) -> &mut Self {
        if self.indices_range != indices_range {
            self.indices_range = indices_range;
            self.mark_as_dirty();
        }
        self
    }
    pub fn is_visible(&self) -> bool {
        self.is_visible && !self.vertices_range.is_empty()
    }
    pub fn set_visible(&mut self, is_visible: bool) -> &mut Self {
        if self.is_visible != is_visible {
            self.is_visible = is_visible;
            self.mark_as_dirty();
        }
        self
    }
    pub fn set_draw_area(&mut self, draw_area: Vector4) -> &mut Self {
        if self.draw_area != draw_area {
            self.draw_area = draw_area;
            self.mark_as_dirty();
        }
        self
    }
    pub fn set_matrix(&mut self, transform: Matrix4) -> &mut Self {
        if self.matrix != transform {
            self.matrix = transform;
            self.mark_as_dirty();
        }
        self
    }
    pub fn set_material(&mut self, material: Resource<Material>) -> &mut Self {
        if self.material.is_none() || self.material.as_ref().unwrap().id() != material.id() {
            self.material = Some(material);
            self.mark_as_dirty();
        }
        self
    }
    pub fn material(&self) -> &Handle<Material> {
        &self.material
    }
    pub fn set_mesh_data(&mut self, mesh_data: MeshData) -> &mut Self {
        if let Some(graphics_mesh) = &self.graphics_mesh {
            let (vertex_range, index_range) =
                graphics_mesh.get_mut().add_mesh_data(&self.id, &mesh_data);
            if self.vertices_range != vertex_range && self.indices_range != index_range {
                self.vertices_range = vertex_range;
                self.indices_range = index_range;
                self.mark_as_dirty();
            }
        }
        self
    }
    pub fn draw_index(&self) -> i32 {
        self.draw_index
    }
    pub fn set_draw_index(&mut self, draw_index: u32) -> &mut Self {
        if self.draw_index != draw_index as i32 {
            self.draw_index = draw_index as _;
            self.mark_as_dirty();
        }
        self
    }
    pub fn matrix(&self) -> Matrix4 {
        self.matrix
    }
    pub fn draw_area(&self) -> Vector4 {
        self.draw_area
    }
}
