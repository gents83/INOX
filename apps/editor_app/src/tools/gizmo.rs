use nrg_graphics::{
    create_arrow, create_sphere, MaterialInstance, MeshData, MeshInstance, MeshRc, PipelineInstance,
};
use nrg_math::Vector3;
use nrg_resources::{
    DataTypeResource, ResourceData, ResourceId, ResourceRef, SharedData, SharedDataRw,
};
use nrg_scene::{Transform, TransformRc};
use nrg_serialize::generate_random_uid;

pub type GizmoId = ResourceId;
pub type GizmoRc = ResourceRef<Gizmo>;

pub enum GizmoType {
    Translation,
    Scale,
    Rotation,
}

pub struct Gizmo {
    id: ResourceId,
    gizmo_type: GizmoType,
    transform: TransformRc,
    mesh: MeshRc,
}

impl ResourceData for Gizmo {
    fn id(&self) -> ResourceId {
        self.id
    }
}

impl Gizmo {
    pub fn new(shared_data: &SharedDataRw, position: Vector3, gizmo_type: GizmoType) -> GizmoRc {
        let transform = Transform::default();
        let transform = SharedData::add_resource(shared_data, transform);
        let mut mesh_data = MeshData::default();

        let (mut vertices, indices) = create_sphere(0.5, 32, 16);
        vertices.iter_mut().for_each(|v| {
            v.pos += position;
        });
        mesh_data.append_mesh(vertices.as_slice(), indices.as_slice());

        let (vertices, indices) =
            create_arrow(position, [10., 0., 0.].into(), [1., 0., 0., 1.].into());
        mesh_data.append_mesh(vertices.as_slice(), indices.as_slice());
        let (vertices, indices) =
            create_arrow(position, [0., 10., 0.].into(), [0., 1., 0., 1.].into());
        mesh_data.append_mesh(vertices.as_slice(), indices.as_slice());
        let (vertices, indices) =
            create_arrow(position, [0., 0., 10.].into(), [0., 0., 1., 1.].into());
        mesh_data.append_mesh(vertices.as_slice(), indices.as_slice());

        let mesh = MeshInstance::create_from_data(shared_data, mesh_data);
        if let Some(pipeline) = PipelineInstance::find_from_name(shared_data, "3D") {
            let material = MaterialInstance::create_from_pipeline(shared_data, pipeline);
            mesh.resource().get_mut().set_material(material);
        }
        let gizmo = Self {
            id: generate_random_uid(),
            gizmo_type,
            transform,
            mesh,
        };
        SharedData::add_resource(shared_data, gizmo)
    }
}
