use std::any::TypeId;

use nrg_graphics::{
    create_arrow, create_hammer, create_sphere, create_torus, Material, Mesh, MeshData, Pipeline,
};

use nrg_math::{
    raycast_oob, Array, InnerSpace, Mat4Ops, MatBase, Matrix4, VecBase, Vector2, Vector3, Vector4,
    Zero,
};

use nrg_messenger::{read_messages, MessageBox, MessageChannel, MessengerRw};
use nrg_resources::{
    DataTypeResource, Resource, ResourceId, ResourceTrait, SharedData, SharedDataRc,
};
use nrg_scene::{Camera, Object, ObjectId};
use nrg_serialize::generate_random_uid;
use nrg_ui::hex_to_rgba;

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
    transform: Matrix4,
    camera_scale: f32,
    mode_type: GizmoType,
    mesh_center: Resource<Mesh>,
    mesh_x: Resource<Mesh>,
    mesh_y: Resource<Mesh>,
    mesh_z: Resource<Mesh>,
    axis: Vector3,
    is_active: bool,
    shared_data: SharedDataRc,
    message_channel: MessageChannel,
    global_dispatcher: MessageBox,
    highlight_color: Vector4,
    center_color: Vector4,
    axis_x_color: Vector4,
    axis_y_color: Vector4,
    axis_z_color: Vector4,
}
impl ResourceTrait for Gizmo {
    fn on_resource_swap(&mut self, _new: &Self)
    where
        Self: Sized,
    {
        //println!("Gizmo resource swapped");
    }
}

unsafe impl Sync for Gizmo {}
unsafe impl Send for Gizmo {}

impl Gizmo {
    fn new(
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        mode_type: GizmoType,
        mesh_center: Resource<Mesh>,
        mesh_x: Resource<Mesh>,
        mesh_y: Resource<Mesh>,
        mesh_z: Resource<Mesh>,
    ) -> Resource<Self> {
        let message_channel = MessageChannel::default();

        global_messenger
            .write()
            .unwrap()
            .register_messagebox::<EditorEvent>(message_channel.get_messagebox());

        let center_color = if let Some(material) = mesh_center.get().material() {
            material.get().base_color()
        } else {
            hex_to_rgba("#FFFFFF")
        };
        let axis_x_color = if let Some(material) = mesh_x.get().material() {
            material.get().base_color()
        } else {
            hex_to_rgba("#FF0000")
        };
        let axis_y_color = if let Some(material) = mesh_y.get().material() {
            material.get().base_color()
        } else {
            hex_to_rgba("#00FF00")
        };
        let axis_z_color = if let Some(material) = mesh_z.get().material() {
            material.get().base_color()
        } else {
            hex_to_rgba("#0000FF")
        };

        let gizmo = Self {
            mode_type,
            transform: Matrix4::default_identity(),
            camera_scale: 1.,
            axis: Vector3::zero(),
            shared_data: shared_data.clone(),
            message_channel,
            global_dispatcher: global_messenger.read().unwrap().get_dispatcher(),
            is_active: false,
            highlight_color: hex_to_rgba("#FFFF00"),
            center_color,
            axis_x_color,
            axis_y_color,
            axis_z_color,
            mesh_center,
            mesh_x,
            mesh_y,
            mesh_z,
        };
        SharedData::add_resource(shared_data, generate_random_uid(), gizmo)
    }

    pub fn new_translation(
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        default_material_pipeline: &Resource<Pipeline>,
    ) -> Resource<Self> {
        Self::new(
            shared_data,
            global_messenger,
            GizmoType::Move,
            Self::create_center_mesh(
                shared_data,
                global_messenger,
                Vector3::zero(),
                default_material_pipeline,
            ),
            Self::create_arrow(
                shared_data,
                global_messenger,
                [10., 0., 0.].into(),
                hex_to_rgba("#FF0000"),
                default_material_pipeline,
            ),
            Self::create_arrow(
                shared_data,
                global_messenger,
                [0., 10., 0.].into(),
                hex_to_rgba("#00FF00"),
                default_material_pipeline,
            ),
            Self::create_arrow(
                shared_data,
                global_messenger,
                [0., 0., 10.].into(),
                hex_to_rgba("#0000FF"),
                default_material_pipeline,
            ),
        )
    }

    pub fn new_scale(
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        default_material_pipeline: &Resource<Pipeline>,
    ) -> Resource<Self> {
        Self::new(
            shared_data,
            global_messenger,
            GizmoType::Scale,
            Self::create_center_mesh(
                shared_data,
                global_messenger,
                Vector3::zero(),
                default_material_pipeline,
            ),
            Self::create_hammer(
                shared_data,
                global_messenger,
                [10., 0., 0.].into(),
                hex_to_rgba("#FF0000"),
                default_material_pipeline,
            ),
            Self::create_hammer(
                shared_data,
                global_messenger,
                [0., 10., 0.].into(),
                hex_to_rgba("#00FF00"),
                default_material_pipeline,
            ),
            Self::create_hammer(
                shared_data,
                global_messenger,
                [0., 0., 10.].into(),
                hex_to_rgba("#0000FF"),
                default_material_pipeline,
            ),
        )
    }

    pub fn new_rotation(
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        default_material_pipeline: &Resource<Pipeline>,
    ) -> Resource<Self> {
        Self::new(
            shared_data,
            global_messenger,
            GizmoType::Rotate,
            Self::create_center_mesh(
                shared_data,
                global_messenger,
                Vector3::zero(),
                default_material_pipeline,
            ),
            Self::create_torus(
                shared_data,
                global_messenger,
                [1., 0., 0.].into(),
                hex_to_rgba("#FF0000"),
                default_material_pipeline,
            ),
            Self::create_torus(
                shared_data,
                global_messenger,
                [0., 1., 0.].into(),
                hex_to_rgba("#00FF00"),
                default_material_pipeline,
            ),
            Self::create_torus(
                shared_data,
                global_messenger,
                [0., 0., 1.].into(),
                hex_to_rgba("#0000FF"),
                default_material_pipeline,
            ),
        )
    }

    fn create_center_mesh(
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        position: Vector3,
        default_material_pipeline: &Resource<Pipeline>,
    ) -> Resource<Mesh> {
        let mut mesh_data = MeshData::default();
        let (mut vertices, indices) = create_sphere(0.5, 32, 16);
        vertices.iter_mut().for_each(|v| {
            v.pos += position;
        });
        mesh_data.append_mesh(vertices.as_slice(), indices.as_slice());
        let mesh = Mesh::new_resource(
            shared_data,
            global_messenger,
            generate_random_uid(),
            mesh_data,
        );
        mesh.get_mut()
            .set_material(Material::duplicate_from_pipeline(
                shared_data,
                default_material_pipeline,
            ));
        mesh
    }
    fn create_arrow(
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        direction: Vector3,
        color: Vector4,
        default_material_pipeline: &Resource<Pipeline>,
    ) -> Resource<Mesh> {
        let mut mesh_data = MeshData::default();

        let (vertices, indices) = create_arrow(Vector3::zero(), direction);
        mesh_data.append_mesh(vertices.as_slice(), indices.as_slice());

        let mesh = Mesh::new_resource(
            shared_data,
            global_messenger,
            generate_random_uid(),
            mesh_data,
        );
        let material = Material::duplicate_from_pipeline(shared_data, default_material_pipeline);
        material.get_mut().set_base_color(color);
        mesh.get_mut().set_material(material);
        mesh
    }
    fn create_hammer(
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        direction: Vector3,
        color: Vector4,
        default_material_pipeline: &Resource<Pipeline>,
    ) -> Resource<Mesh> {
        let mut mesh_data = MeshData::default();

        let (vertices, indices) = create_hammer(Vector3::zero(), direction);
        mesh_data.append_mesh(vertices.as_slice(), indices.as_slice());

        let mesh = Mesh::new_resource(
            shared_data,
            global_messenger,
            generate_random_uid(),
            mesh_data,
        );
        let material = Material::duplicate_from_pipeline(shared_data, default_material_pipeline);
        material.get_mut().set_base_color(color);
        mesh.get_mut().set_material(material);
        mesh
    }
    fn create_torus(
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        direction: Vector3,
        color: Vector4,
        default_material_pipeline: &Resource<Pipeline>,
    ) -> Resource<Mesh> {
        let mut mesh_data = MeshData::default();

        let (vertices, indices) = create_torus(Vector3::zero(), 10., 0.2, 32, 32, direction);
        mesh_data.append_mesh(vertices.as_slice(), indices.as_slice());

        let mesh = Mesh::new_resource(
            shared_data,
            global_messenger,
            generate_random_uid(),
            mesh_data,
        );
        let material = Material::duplicate_from_pipeline(shared_data, default_material_pipeline);
        material.get_mut().set_base_color(color);
        mesh.get_mut().set_material(material);
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
        let (translation, rotation, _) = self.transform.get_translation_rotation_scale();
        self.transform.from_translation_rotation_scale(
            translation,
            rotation,
            Vector3::from_value(camera_scale),
        );
        self.mesh_center.get_mut().set_matrix(self.transform);

        self.mesh_x.get_mut().set_matrix(self.transform);
        self.mesh_y.get_mut().set_matrix(self.transform);
        self.mesh_z.get_mut().set_matrix(self.transform);
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
        let is_dragging = if self.is_ray_inside(start_ray, end_ray, &self.mesh_center) {
            self.axis = Vector3::default_one();
            true
        } else if self.axis != Vector3::zero() {
            true
        } else {
            false
        };
        is_dragging
    }
    pub fn end_drag(&mut self) {
        self.axis = Vector3::zero();
    }
    pub fn drag(&mut self, intensity: f32, object_id: &ObjectId) {
        if object_id.is_nil() {
            return;
        }
        let mut delta = Vector3::new(
            self.axis.x * intensity,
            self.axis.y * intensity,
            self.axis.z * intensity,
        );
        if self.axis == Vector3::default_one() {
            let min = delta.x.min(delta.y).min(delta.z);
            let max = delta.x.max(delta.y).max(delta.z);
            if min.abs() >= max.abs() {
                delta = Vector3::from_value(min);
            } else {
                delta = Vector3::from_value(max);
            }
        }

        if self.mode_type == GizmoType::Move {
            self.transform.add_translation(delta);
            if let Some(object) = SharedData::get_resource::<Object>(&self.shared_data, object_id) {
                object.get_mut().translate(delta);
            }
        } else if self.mode_type == GizmoType::Scale {
            if let Some(object) = SharedData::get_resource::<Object>(&self.shared_data, object_id) {
                object.get_mut().scale(delta);
            }
        } else if self.mode_type == GizmoType::Rotate {
            delta *= -1.;
            if let Some(object) = SharedData::get_resource::<Object>(&self.shared_data, object_id) {
                object.get_mut().rotate(delta);
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
        selected_object: &ObjectId,
    ) -> bool {
        if !self.is_visible() {
            return false;
        }
        let (new_cam_start, new_cam_end) = camera.convert_in_3d(new_pos);
        if is_drag_started {
            self.is_active = self.start_drag(new_cam_start, new_cam_end);
        } else if is_drag_ended {
            self.end_drag();
            self.is_active = false;
        } else if self.is_active {
            let dir = old_pos - new_pos;
            let x = dir.x.abs();
            let y = dir.y.abs();
            let mut intensity = x.max(y);
            intensity = if x > y {
                intensity * -dir.x.signum()
            } else {
                intensity * dir.y.signum()
            };
            self.drag(intensity * 10., selected_object);
        }

        self.is_active
    }

    pub fn update(&mut self, camera: &Camera) {
        self.update_events();

        if self.is_visible() {
            let cam_pos = camera.position();
            let pos = self.transform.translation();
            let direction = pos - cam_pos;
            self.camera_scale = direction.length() / DEFAULT_DISTANCE_SCALE;
            self.update_meshes(self.camera_scale);
        }
    }

    fn highlight_mesh(
        mesh: &Resource<Mesh>,
        mesh_u32: u32,
        highlight_color: Vector4,
        default_color: Vector4,
    ) -> bool {
        if let Some(material) = mesh.get().material() {
            if mesh.id().as_u128() as u32 == mesh_u32 {
                material.get_mut().set_base_color(highlight_color);
                return true;
            } else {
                material.get_mut().set_base_color(default_color);
            }
        }
        return false;
    }

    fn update_events(&mut self) {
        nrg_profiler::scoped_profile!("update_events");

        read_messages(self.message_channel.get_listener(), |msg| {
            if msg.type_id() == TypeId::of::<EditorEvent>() {
                let event = msg.as_any().downcast_ref::<EditorEvent>().unwrap();
                match *event {
                    EditorEvent::Selected(object_id) => {
                        self.select_object(&object_id);
                    }
                    EditorEvent::HoverMesh(mesh) => {
                        if Self::highlight_mesh(
                            &self.mesh_center,
                            mesh,
                            self.highlight_color,
                            self.center_color,
                        ) {
                            self.axis = Vector3::default_one();
                        } else if Self::highlight_mesh(
                            &self.mesh_x,
                            mesh,
                            self.highlight_color,
                            self.axis_x_color,
                        ) {
                            self.axis = Vector3::unit_x();
                        } else if Self::highlight_mesh(
                            &self.mesh_y,
                            mesh,
                            self.highlight_color,
                            self.axis_y_color,
                        ) {
                            self.axis = Vector3::unit_y();
                        } else if Self::highlight_mesh(
                            &self.mesh_z,
                            mesh,
                            self.highlight_color,
                            self.axis_z_color,
                        ) {
                            self.axis = Vector3::unit_z();
                        }
                    }
                    _ => {}
                }
            }
        });
    }

    pub fn select_object(&mut self, object_id: &ObjectId) {
        if object_id.is_nil() {
            self.set_visible(false);
            self.transform.set_translation(Vector3::zero());
        } else {
            if let Some(object) = SharedData::get_resource::<Object>(&self.shared_data, object_id) {
                self.transform.set_translation(object.get().get_position());
                self.update_meshes(self.camera_scale);
                self.set_visible(true);
            }
        }
    }
}
