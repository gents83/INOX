use nrg_camera::Camera;
use nrg_graphics::{
    DynamicImage, MeshInstance, RenderPassInstance, TextureInstance, TextureRc, ViewInstance,
};
use nrg_math::{
    compute_distance_between_ray_and_oob, InnerSpace, Mat4Ops, MatBase, Matrix4, Vector2, Vector3,
    Zero,
};
use nrg_messenger::{implement_message, Message, MessageBox, MessengerRw};
use nrg_platform::{Key, KeyEvent};
use nrg_resources::{DataTypeResource, SharedData, SharedDataRw};
use nrg_scene::{Hitbox, Object, ObjectId, Transform};
use nrg_serialize::{generate_uid_from_string, INVALID_UID};
use nrg_ui::{
    implement_widget_data, CentralPanel, Frame, Image, LayerId, Sense, TextureId as eguiTextureId,
    UIWidget, UIWidgetRc, Widget,
};

use crate::{
    systems::BoundingBoxDrawer,
    tools::{Gizmo, GizmoRc},
};

const VIEW3D_IMAGE_WIDTH: u32 = 1280;
const VIEW3D_IMAGE_HEIGHT: u32 = 768;

#[derive(Clone)]
pub enum ViewEvent {
    Selected(ObjectId),
}
implement_message!(ViewEvent);

struct View3DData {
    shared_data: SharedDataRw,
    global_dispatcher: MessageBox,
    texture: TextureRc,
    camera: Camera,
    should_manage_input: bool,
    last_mouse_pos: Vector2,
    selected_object: ObjectId,
    view_width: u32,
    view_height: u32,
    gizmo: GizmoRc,
}
implement_widget_data!(View3DData);

pub struct View3D {
    ui_page: UIWidgetRc,
    shared_data: SharedDataRw,
    bounding_box_drawer: BoundingBoxDrawer,
}

unsafe impl Send for View3D {}
unsafe impl Sync for View3D {}

impl View3D {
    pub fn new(shared_data: &SharedDataRw, global_messenger: &MessengerRw) -> Self {
        let texture = Self::update_texture(shared_data, VIEW3D_IMAGE_WIDTH, VIEW3D_IMAGE_HEIGHT);

        let mut camera = Camera::new([20., 20., -20.].into(), [0., 0., 0.].into(), true);
        camera.set_projection(
            45.,
            VIEW3D_IMAGE_WIDTH as _,
            VIEW3D_IMAGE_HEIGHT as _,
            0.001,
            1000.,
        );

        let move_gizmo =
            Gizmo::new_translation(shared_data, global_messenger.clone(), [0., 0., 0.].into());

        let data = View3DData {
            shared_data: shared_data.clone(),
            global_dispatcher: global_messenger.read().unwrap().get_dispatcher().clone(),
            texture,
            camera,
            last_mouse_pos: Vector2::zero(),
            selected_object: INVALID_UID,
            view_width: VIEW3D_IMAGE_WIDTH,
            view_height: VIEW3D_IMAGE_HEIGHT,
            should_manage_input: false,
            gizmo: move_gizmo,
        };
        data.gizmo.resource().get_mut().set_visible(false);
        let ui_page = Self::create(shared_data, data);
        Self {
            ui_page,
            bounding_box_drawer: BoundingBoxDrawer::new(
                shared_data,
                global_messenger.clone(),
                "Wireframe",
            ),
            shared_data: shared_data.clone(),
        }
    }

    pub fn update(&mut self) -> &mut Self {
        self.update_camera().update_gizmo();
        self.bounding_box_drawer.update();
        self
    }

    fn update_gizmo(&mut self) -> &mut Self {
        if let Some(data) = self.ui_page.resource().get_mut().data_mut::<View3DData>() {
            data.gizmo.resource().get_mut().update_events();
        }
        self
    }
    pub fn handle_keyboard_event(&mut self, event: &KeyEvent) {
        if let Some(data) = self.ui_page.resource().get_mut().data_mut::<View3DData>() {
            let mut movement = Vector3::zero();
            if event.code == Key::W {
                movement.z += 1.;
            } else if event.code == Key::S {
                movement.z -= 1.;
            } else if event.code == Key::A {
                movement.x += 1.;
            } else if event.code == Key::D {
                movement.x -= 1.;
            } else if event.code == Key::Q {
                movement.y += 1.;
            } else if event.code == Key::E {
                movement.y -= 1.;
            }
            if data.should_manage_input {
                data.camera.translate(movement);
            }
        }
    }

    fn resize_view(
        data: &mut View3DData,
        view_width: u32,
        view_height: u32,
        is_using_pointer: bool,
    ) -> usize {
        let mut texture_index = 0;
        let texture_id = data.texture.id();
        let textures = SharedData::get_resources_of_type::<TextureInstance>(&data.shared_data);
        if let Some(index) = textures.iter().position(|t| t.id() == texture_id) {
            texture_index = index;
        }
        let texture_width = data.texture.resource().get().width();
        let texture_height = data.texture.resource().get().height();
        if !is_using_pointer
            && data.view_width == view_width
            && data.view_height == view_height
            && (texture_width != data.view_width || texture_height != data.view_height)
        {
            data.texture =
                Self::update_texture(&data.shared_data, data.view_width, data.view_height);
            data.camera.set_projection(
                45.,
                data.view_width as _,
                data.view_height as _,
                0.001,
                1000.,
            );
        }
        data.view_width = view_width;
        data.view_height = view_height;
        texture_index
    }

    fn manage_input(
        data: &mut View3DData,
        normalized_pos: Vector2,
        is_clicked: bool,
        is_drag_started: bool,
        is_drag_ended: bool,
    ) {
        if data.last_mouse_pos.x < 0. || data.last_mouse_pos.y < 0. {
            data.last_mouse_pos = normalized_pos;
        }

        if is_clicked {
            data.selected_object =
                Self::update_selected_object(data, normalized_pos.x, normalized_pos.y);

            data.global_dispatcher
                .write()
                .unwrap()
                .send(ViewEvent::Selected(data.selected_object).as_boxed())
                .ok();
        } else {
            let is_manipulating_gizmo = data.gizmo.resource().get_mut().update(
                &data.camera,
                data.last_mouse_pos,
                normalized_pos,
                is_drag_started,
                is_drag_ended,
                data.selected_object,
            );
            if !is_manipulating_gizmo {
                let mut rotation_angle = Vector3::zero();

                rotation_angle.x = normalized_pos.y - data.last_mouse_pos.y;
                rotation_angle.y = data.last_mouse_pos.x - normalized_pos.x;
                data.camera.rotate(rotation_angle * 5.);
            }
        }
        data.last_mouse_pos = normalized_pos;
    }

    fn create(shared_data: &SharedDataRw, data: View3DData) -> UIWidgetRc {
        UIWidget::register(shared_data, data, |ui_data, ui_context| {
            if let Some(data) = ui_data.as_any().downcast_mut::<View3DData>() {
                CentralPanel::default()
                    .frame(Frame::dark_canvas(ui_context.style().as_ref()))
                    .show(ui_context, |ui| {
                        data.should_manage_input = !ui.ctx().wants_keyboard_input();
                        let is_using_pointer =
                            ui.ctx().is_using_pointer() || ui.ctx().wants_pointer_input();

                        let view_width = ui.max_rect_finite().width() as u32;
                        let view_height = ui.max_rect_finite().height() as u32;
                        let texture_index =
                            Self::resize_view(data, view_width, view_height, is_using_pointer);

                        ui.with_layer_id(LayerId::background(), |ui| {
                            let response = Image::new(
                                eguiTextureId::User(texture_index as _),
                                [data.view_width as _, data.view_height as _],
                            )
                            .sense(Sense::click_and_drag())
                            .ui(ui);
                            if let Some(pos) = response.interact_pointer_pos() {
                                let rect = response.rect;
                                let normalized_x = (pos.x - rect.min.x) / rect.width();
                                let normalized_y = (pos.y - rect.min.y) / rect.height();
                                let new_pos = [normalized_x, normalized_y].into();
                                Self::manage_input(
                                    data,
                                    new_pos,
                                    response.clicked(),
                                    response.drag_started(),
                                    response.drag_released(),
                                );
                            } else {
                                data.last_mouse_pos = [-1., -1.].into();
                            }
                            response
                        })
                    });
            }
        })
    }

    fn update_camera(&mut self) -> &mut Self {
        if let Some(data) = self.ui_page.resource().get_mut().data_mut::<View3DData>() {
            if SharedData::has_resources_of_type::<ViewInstance>(&self.shared_data) {
                let views = SharedData::get_resources_of_type::<ViewInstance>(&self.shared_data);
                let view = views.first().unwrap();
                let view_matrix = data.camera.get_view_matrix();
                let proj_matrix = data.camera.get_proj_matrix();

                let texture_width = data.texture.resource().get().width();
                let texture_height = data.texture.resource().get().height();

                view.resource()
                    .get_mut()
                    .update_view(view_matrix)
                    .update_proj(proj_matrix)
                    .update_size(texture_width, texture_height);
            }
        }
        self
    }

    fn update_texture(shared_data: &SharedDataRw, width: u32, height: u32) -> TextureRc {
        let image = DynamicImage::new_rgba8(width, height);
        let image_data = image.to_rgba8();
        let texture = TextureInstance::create_from_data(shared_data, image_data);

        {
            let render_pass_id = generate_uid_from_string("MainPass");
            let render_pass =
                SharedData::get_resource::<RenderPassInstance>(shared_data, render_pass_id);
            render_pass
                .resource()
                .get_mut()
                .set_color_texture(texture.clone());
        }

        texture
    }

    fn update_selected_object(
        data: &mut View3DData,
        normalized_x: f32,
        normalized_y: f32,
    ) -> ObjectId {
        let mut selected_object = INVALID_UID;

        let (ray_start_world, ray_end_world) = data
            .camera
            .convert_in_3d([normalized_x, normalized_y].into());

        let ray_dir_world = ray_end_world - ray_start_world;
        let ray_dir_world = ray_dir_world.normalize();

        if SharedData::has_resources_of_type::<Object>(&data.shared_data) {
            let mut min = [-5., -5., -5.].into();
            let mut max = [5., 5., 5.].into();
            let mut matrix = Matrix4::default_identity();
            let objects = SharedData::get_resources_of_type::<Object>(&data.shared_data);
            for obj in objects {
                if let Some(transform) = obj.resource().get().get_component::<Transform>() {
                    matrix = transform.resource().get().matrix();
                }
                if let Some(hitbox) = obj.resource().get().get_component::<Hitbox>() {
                    min = hitbox.resource().get().min();
                    max = hitbox.resource().get().max();
                } else if let Some(mesh) = obj.resource().get().get_component::<MeshInstance>() {
                    let transform = mesh.resource().get().matrix();
                    let (mesh_min, mesh_max) = mesh.resource().get().mesh_data().compute_min_max();
                    min = transform.transform(mesh_min);
                    max = transform.transform(mesh_max);
                }
                if compute_distance_between_ray_and_oob(
                    ray_start_world.xyz(),
                    ray_dir_world.xyz(),
                    min,
                    max,
                    matrix,
                ) {
                    selected_object = obj.id();
                }
            }
        }
        selected_object
    }
}
