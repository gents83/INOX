use nrg_graphics::{
    create_arrow, create_sphere, MaterialInstance, MeshData, MeshInstance, MeshRc, PipelineInstance,
};
use nrg_math::{
    compute_distance_between_ray_and_oob, Mat4Ops, Matrix4, VecBase, Vector3, Vector4, Zero,
};
use nrg_resources::{
    DataTypeResource, ResourceData, ResourceId, ResourceRef, SharedData, SharedDataRw,
};
use nrg_scene::{Transform, TransformRc};
use nrg_serialize::generate_random_uid;

pub type GizmoId = ResourceId;
pub type GizmoRc = ResourceRef<Gizmo>;

pub struct Gizmo {
    id: ResourceId,
    transform: TransformRc,
    mesh_center: MeshRc,
    mesh_x: MeshRc,
    mesh_y: MeshRc,
    mesh_z: MeshRc,
    axis: Vector3,
}

impl ResourceData for Gizmo {
    fn id(&self) -> ResourceId {
        self.id
    }
}

impl Gizmo {
    pub fn new_translation(shared_data: &SharedDataRw, position: Vector3) -> GizmoRc {
        let transform = Transform::default();
        let transform = SharedData::add_resource(shared_data, transform);

        let gizmo = Self {
            id: generate_random_uid(),
            transform,
            axis: Vector3::zero(),
            mesh_center: Self::create_center_mesh(shared_data, position),
            mesh_x: Self::create_arrow(
                shared_data,
                position,
                [10., 0., 0.].into(),
                [1., 0., 0., 1.].into(),
            ),
            mesh_y: Self::create_arrow(
                shared_data,
                position,
                [0., 10., 0.].into(),
                [0., 1., 0., 1.].into(),
            ),
            mesh_z: Self::create_arrow(
                shared_data,
                position,
                [0., 0., 10.].into(),
                [0., 0., 1., 1.].into(),
            ),
        };
        SharedData::add_resource(shared_data, gizmo)
    }

    fn add_material(shared_data: &SharedDataRw, mesh: &MeshRc) {
        if let Some(pipeline) = PipelineInstance::find_from_name(shared_data, "3D") {
            let material = MaterialInstance::create_from_pipeline(shared_data, pipeline);
            mesh.resource().get_mut().set_material(material);
        }
    }

    fn create_center_mesh(shared_data: &SharedDataRw, position: Vector3) -> MeshRc {
        let mut mesh_data = MeshData::default();
        let (mut vertices, indices) = create_sphere(0.5, 32, 16);
        vertices.iter_mut().for_each(|v| {
            v.pos += position;
        });
        mesh_data.append_mesh(vertices.as_slice(), indices.as_slice());
        let mesh = MeshInstance::create_from_data(shared_data, mesh_data);
        Self::add_material(shared_data, &mesh);
        mesh
    }
    fn create_arrow(
        shared_data: &SharedDataRw,
        position: Vector3,
        direction: Vector3,
        color: Vector4,
    ) -> MeshRc {
        let mut mesh_data = MeshData::default();

        let (vertices, indices) = create_arrow(position, direction, color);
        mesh_data.append_mesh(vertices.as_slice(), indices.as_slice());

        let mesh = MeshInstance::create_from_data(shared_data, mesh_data);
        Self::add_material(shared_data, &mesh);
        mesh
    }

    pub fn set_visible(&mut self, is_visible: bool) -> &mut Self {
        self.mesh_center
            .resource()
            .get_mut()
            .set_visible(is_visible);

        self.mesh_x.resource().get_mut().set_visible(is_visible);
        self.mesh_y.resource().get_mut().set_visible(is_visible);
        self.mesh_z.resource().get_mut().set_visible(is_visible);
        self
    }

    pub fn is_visible(&self) -> bool {
        self.mesh_center.resource().get().is_visible()
    }

    pub fn set_position(&mut self, position: Vector3) -> &mut Self {
        let matrix = Matrix4::from_translation(position);
        self.transform.resource().get_mut().set_matrix(matrix);

        self.mesh_center.resource().get_mut().set_matrix(matrix);

        self.mesh_x.resource().get_mut().set_matrix(matrix);
        self.mesh_y.resource().get_mut().set_matrix(matrix);
        self.mesh_z.resource().get_mut().set_matrix(matrix);

        self
    }

    pub fn position(&self) -> Vector3 {
        self.transform.resource().get().matrix().translation()
    }

    fn is_ray_inside(&self, start: Vector3, end: Vector3, mesh: &MeshRc) -> bool {
        let (mesh_min, mesh_max) = mesh.resource().get().mesh_data().compute_min_max();

        compute_distance_between_ray_and_oob(
            start,
            end,
            mesh_min,
            mesh_max,
            self.transform.resource().get().matrix(),
        )
    }

    pub fn start_drag(&mut self, start_ray: Vector3, end_ray: Vector3) -> bool {
        let mut is_dragging = false;
        if self.is_ray_inside(start_ray, end_ray, &self.mesh_center) {
            self.axis = Vector3::default_one();
            is_dragging = true;
        } else if self.is_ray_inside(start_ray, end_ray, &self.mesh_x) {
            self.axis = Vector3::unit_x();
            is_dragging = true;
        } else if self.is_ray_inside(start_ray, end_ray, &self.mesh_y) {
            self.axis = Vector3::unit_y();
            is_dragging = true;
        } else if self.is_ray_inside(start_ray, end_ray, &self.mesh_z) {
            self.axis = Vector3::unit_z();
            is_dragging = true;
        }
        is_dragging
    }
    pub fn end_drag(&mut self) {
        self.axis = Vector3::zero();
    }
    pub fn drag(&mut self, old_position: Vector3, new_position: Vector3) {
        let mut movement = new_position - old_position;
        movement.x *= self.axis.x;
        movement.y *= self.axis.y;
        movement.z *= self.axis.z;
        let position = self.position();
        self.set_position(position + movement);
    }
}
