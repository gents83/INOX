use std::any::TypeId;

use nrg_camera::Camera;
use nrg_graphics::{
    create_arrow, create_hammer, create_sphere, create_torus, Material, Mesh, MeshData, Pipeline,
};

use nrg_math::{
    raycast_oob, Array, InnerSpace, Mat4Ops, MatBase, Matrix4, VecBase, Vector2, Vector3, Vector4,
    Zero,
};

use nrg_messenger::{read_messages, MessageBox, MessageChannel, MessengerRw};
use nrg_resources::{
    DataTypeResource, Resource, ResourceData, ResourceId, SharedData, SharedDataRw,
};
use nrg_scene::{Object, ObjectId, Transform};
use nrg_serialize::generate_random_uid;

use crate::EditorEvent;

pub type GizmoId = ResourceId;

const DEFAULT_DISTANCE_SCALE: f32 = 35.;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum GizmoType {
    Move,
    Scale,
    Rotate,
}

pub struct Gizmo {
    id: ResourceId,
    transform: Resource<Transform>,
    camera_scale: f32,
    mode_type: GizmoType,
    mesh_center: Resource<Mesh>,
    mesh_x: Resource<Mesh>,
    mesh_y: Resource<Mesh>,
    mesh_z: Resource<Mesh>,
    axis: Vector3,
    is_active: bool,
    shared_data: SharedDataRw,
    message_channel: MessageChannel,
    global_dispatcher: MessageBox,
}

impl ResourceData for Gizmo {
    fn id(&self) -> ResourceId {
        self.id
    }
}

unsafe impl Sync for Gizmo {}
unsafe impl Send for Gizmo {}

impl Gizmo {
    fn new(
        shared_data: &SharedDataRw,
        global_messenger: MessengerRw,
        mode_type: GizmoType,
        mesh_center: Resource<Mesh>,
        mesh_x: Resource<Mesh>,
        mesh_y: Resource<Mesh>,
        mesh_z: Resource<Mesh>,
    ) -> Resource<Self> {
        let transform = Transform::default();
        let transform = SharedData::add_resource(shared_data, transform);

        let message_channel = MessageChannel::default();

        global_messenger
            .write()
            .unwrap()
            .register_messagebox::<EditorEvent>(message_channel.get_messagebox());

        let gizmo = Self {
            id: generate_random_uid(),
            mode_type,
            transform,
            camera_scale: 1.,
            axis: Vector3::zero(),
            shared_data: shared_data.clone(),
            message_channel,
            global_dispatcher: global_messenger.read().unwrap().get_dispatcher(),
            is_active: false,
            mesh_center,
            mesh_x,
            mesh_y,
            mesh_z,
        };
        SharedData::add_resource(shared_data, gizmo)
    }

    pub fn new_translation(
        shared_data: &SharedDataRw,
        global_messenger: MessengerRw,
        default_material_pipeline: &Resource<Pipeline>,
    ) -> Resource<Self> {
        Self::new(
            shared_data,
            global_messenger,
            GizmoType::Move,
            Self::create_center_mesh(shared_data, Vector3::zero(), default_material_pipeline),
            Self::create_arrow(
                shared_data,
                [10., 0., 0.].into(),
                [1., 0., 0., 1.].into(),
                default_material_pipeline,
            ),
            Self::create_arrow(
                shared_data,
                [0., 10., 0.].into(),
                [0., 1., 0., 1.].into(),
                default_material_pipeline,
            ),
            Self::create_arrow(
                shared_data,
                [0., 0., 10.].into(),
                [0., 0., 1., 1.].into(),
                default_material_pipeline,
            ),
        )
    }

    pub fn new_scale(
        shared_data: &SharedDataRw,
        global_messenger: MessengerRw,
        default_material_pipeline: &Resource<Pipeline>,
    ) -> Resource<Self> {
        Self::new(
            shared_data,
            global_messenger,
            GizmoType::Scale,
            Self::create_center_mesh(shared_data, Vector3::zero(), default_material_pipeline),
            Self::create_hammer(
                shared_data,
                [10., 0., 0.].into(),
                [1., 0., 0., 1.].into(),
                default_material_pipeline,
            ),
            Self::create_hammer(
                shared_data,
                [0., 10., 0.].into(),
                [0., 1., 0., 1.].into(),
                default_material_pipeline,
            ),
            Self::create_hammer(
                shared_data,
                [0., 0., 10.].into(),
                [0., 0., 1., 1.].into(),
                default_material_pipeline,
            ),
        )
    }

    pub fn new_rotation(
        shared_data: &SharedDataRw,
        global_messenger: MessengerRw,
        default_material_pipeline: &Resource<Pipeline>,
    ) -> Resource<Self> {
        Self::new(
            shared_data,
            global_messenger,
            GizmoType::Rotate,
            Self::create_center_mesh(shared_data, Vector3::zero(), default_material_pipeline),
            Self::create_torus(
                shared_data,
                [1., 0., 0.].into(),
                [1., 0., 0., 1.].into(),
                default_material_pipeline,
            ),
            Self::create_torus(
                shared_data,
                [0., 1., 0.].into(),
                [0., 1., 0., 1.].into(),
                default_material_pipeline,
            ),
            Self::create_torus(
                shared_data,
                [0., 0., 1.].into(),
                [0., 0., 1., 1.].into(),
                default_material_pipeline,
            ),
        )
    }

    fn create_center_mesh(
        shared_data: &SharedDataRw,
        position: Vector3,
        default_material_pipeline: &Resource<Pipeline>,
    ) -> Resource<Mesh> {
        let mut mesh_data = MeshData::default();
        let (mut vertices, indices) = create_sphere(0.5, 32, 16);
        vertices.iter_mut().for_each(|v| {
            v.pos += position;
        });
        mesh_data.append_mesh(vertices.as_slice(), indices.as_slice());
        let mesh = Mesh::create_from_data(shared_data, mesh_data);
        mesh.get_mut().set_material(Material::create_from_pipeline(
            shared_data,
            default_material_pipeline,
        ));
        mesh
    }
    fn create_arrow(
        shared_data: &SharedDataRw,
        direction: Vector3,
        color: Vector4,
        default_material_pipeline: &Resource<Pipeline>,
    ) -> Resource<Mesh> {
        let mut mesh_data = MeshData::default();

        let (vertices, indices) = create_arrow(Vector3::zero(), direction, color);
        mesh_data.append_mesh(vertices.as_slice(), indices.as_slice());

        let mesh = Mesh::create_from_data(shared_data, mesh_data);
        mesh.get_mut().set_material(Material::create_from_pipeline(
            shared_data,
            default_material_pipeline,
        ));
        mesh
    }
    fn create_hammer(
        shared_data: &SharedDataRw,
        direction: Vector3,
        color: Vector4,
        default_material_pipeline: &Resource<Pipeline>,
    ) -> Resource<Mesh> {
        let mut mesh_data = MeshData::default();

        let (vertices, indices) = create_hammer(Vector3::zero(), direction, color);
        mesh_data.append_mesh(vertices.as_slice(), indices.as_slice());

        let mesh = Mesh::create_from_data(shared_data, mesh_data);
        mesh.get_mut().set_material(Material::create_from_pipeline(
            shared_data,
            default_material_pipeline,
        ));
        mesh
    }
    fn create_torus(
        shared_data: &SharedDataRw,
        direction: Vector3,
        color: Vector4,
        default_material_pipeline: &Resource<Pipeline>,
    ) -> Resource<Mesh> {
        let mut mesh_data = MeshData::default();

        let (vertices, indices) = create_torus(Vector3::zero(), 10., 0.1, 32, 32, direction, color);
        mesh_data.append_mesh(vertices.as_slice(), indices.as_slice());

        let mesh = Mesh::create_from_data(shared_data, mesh_data);
        mesh.get_mut().set_material(Material::create_from_pipeline(
            shared_data,
            default_material_pipeline,
        ));
        mesh
    }

    pub fn set_visible(&mut self, is_visible: bool) -> &mut Self {
        self.mesh_center.get_mut().set_visible(is_visible);

        self.mesh_x.get_mut().set_visible(is_visible);
        self.mesh_y.get_mut().set_visible(is_visible);
        self.mesh_z.get_mut().set_visible(is_visible);
        self
    }

    pub fn is_visible(&self) -> bool {
        self.mesh_center.get().is_visible()
    }

    pub fn update_meshes(&mut self, camera_scale: f32) -> &mut Self {
        let mut matrix = self.transform.get().matrix();
        let (translation, rotation, _) = matrix.get_translation_rotation_scale();
        matrix.from_translation_rotation_scale(
            translation,
            rotation,
            Vector3::from_value(camera_scale),
        );
        self.mesh_center.get_mut().set_matrix(matrix);

        self.mesh_x.get_mut().set_matrix(matrix);
        self.mesh_y.get_mut().set_matrix(matrix);
        self.mesh_z.get_mut().set_matrix(matrix);
        self
    }

    fn is_ray_inside(&self, start: Vector3, end: Vector3, mesh: &Resource<Mesh>) -> bool {
        let (min, max) = mesh.get().mesh_data().compute_min_max();
        let matrix = mesh.get().matrix();
        let min = matrix.transform(min);
        let max = matrix.transform(max);
        raycast_oob(
            start,
            (end - start).normalize(),
            min,
            max,
            Matrix4::default_identity(),
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
    pub fn drag(&mut self, old_position: Vector3, new_position: Vector3, object_id: ObjectId) {
        if object_id.is_nil() {
            return;
        }
        let mut delta = new_position - old_position;
        delta.x *= self.axis.x;
        delta.y *= self.axis.y;
        delta.z *= self.axis.z;
        if self.mode_type == GizmoType::Move {
            self.transform.get_mut().translate(delta);
            let object = SharedData::get_resource::<Object>(&self.shared_data, object_id);
            let object = object.get();
            if let Some(transform) = object.get_component::<Transform>() {
                transform.get_mut().translate(delta);
            }
        } else if self.mode_type == GizmoType::Scale {
            let object = SharedData::get_resource::<Object>(&self.shared_data, object_id);
            let object = object.get();
            if let Some(transform) = object.get_component::<Transform>() {
                if self.axis == Vector3::default_one() {
                    let min = delta.x.min(delta.y).min(delta.z);
                    let max = delta.x.max(delta.y).max(delta.z);
                    if min.abs() >= max.abs() {
                        delta = Vector3::from_value(min);
                    } else {
                        delta = Vector3::from_value(max);
                    }
                }
                transform.get_mut().add_scale(delta);
            }
        } else if self.mode_type == GizmoType::Rotate {
            delta.x *= -1.;
            delta.y *= -1.;
            let object = SharedData::get_resource::<Object>(&self.shared_data, object_id);
            let object = object.get();
            if let Some(transform) = object.get_component::<Transform>() {
                transform.get_mut().rotate(delta);
            }
        }
        self.update_meshes(self.camera_scale);
    }

    pub fn manipulate(
        &mut self,
        camera: &Camera,
        old_pos: Vector2,
        new_pos: Vector2,
        is_drag_started: bool,
        is_drag_ended: bool,
        selected_object: ObjectId,
    ) -> bool {
        if !self.is_visible() {
            return false;
        }
        let pos = self.transform.get().position();
        let (old_cam_start, old_cam_end) = camera.convert_in_3d(old_pos);
        let (new_cam_start, new_cam_end) = camera.convert_in_3d(new_pos);
        let old_dir = pos - old_cam_start;
        let new_dir = pos - new_cam_start;
        let old_position =
            old_cam_start + (old_cam_end - old_cam_start).normalize() * old_dir.length();
        let new_position =
            new_cam_start + (new_cam_end - new_cam_start).normalize() * new_dir.length();
        if is_drag_started {
            self.is_active = self.start_drag(new_cam_start, new_cam_end);
        } else if is_drag_ended {
            self.end_drag();
            self.is_active = false;
        } else if self.is_active {
            self.drag(old_position, new_position, selected_object);
        }

        self.is_active
    }

    pub fn update(&mut self, camera: &Camera) {
        self.update_events();

        if self.is_visible() {
            let cam_pos = camera.position();
            let pos = self.transform.get().position();
            let direction = pos - cam_pos;
            self.camera_scale = direction.length() / DEFAULT_DISTANCE_SCALE;
            self.update_meshes(self.camera_scale);
        }
    }

    fn update_events(&mut self) {
        nrg_profiler::scoped_profile!("update_events");

        read_messages(self.message_channel.get_listener(), |msg| {
            if msg.type_id() == TypeId::of::<EditorEvent>() {
                let event = msg.as_any().downcast_ref::<EditorEvent>().unwrap();
                if let EditorEvent::Selected(object_id) = *event {
                    self.select_object(object_id);
                }
            }
        });
    }

    pub fn select_object(&mut self, object_id: ObjectId) {
        if object_id.is_nil() {
            self.set_visible(false);
            self.transform.get_mut().set_position(Vector3::zero());
        } else {
            let object = SharedData::get_resource::<Object>(&self.shared_data, object_id);
            if let Some(transform) = object.get().get_component::<Transform>() {
                self.transform
                    .get_mut()
                    .set_position(transform.get().position());
                self.update_meshes(self.camera_scale);
            }
            self.set_visible(true);
        }
    }
}
