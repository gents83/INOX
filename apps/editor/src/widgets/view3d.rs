use nrg_camera::Camera;
use nrg_graphics::{
    DynamicImage, MaterialRc, Mesh, MeshCategoryId, PipelineRc, RenderPass, Texture, TextureId,
    TextureRc, View, DEFAULT_MESH_CATEGORY_IDENTIFIER,
};
use nrg_math::{raycast_oob, InnerSpace, MatBase, Matrix4, Vector2, Vector3, Zero};
use nrg_messenger::{Message, MessengerRw};
use nrg_platform::{Key, KeyEvent};
use nrg_resources::{DataTypeResource, Resource, ResourceData, SharedData, SharedDataRw};
use nrg_scene::{Hitbox, Object, ObjectId, Transform};
use nrg_serialize::{generate_uid_from_string, INVALID_UID};
use nrg_ui::{
    implement_widget_data, CentralPanel, Frame, Image, LayerId, Sense, TextureId as eguiTextureId,
    UIWidget, UIWidgetRc, Widget,
};

use crate::{
    resources::{Gizmo, GizmoRc},
    systems::{BoundingBoxDrawer, DebugDrawer},
    EditMode, EditorEvent,
};

const VIEW3D_IMAGE_WIDTH: u32 = 1280;
const VIEW3D_IMAGE_HEIGHT: u32 = 768;

struct View3DData {
    shared_data: SharedDataRw,
    global_messenger: MessengerRw,
    texture: TextureRc,
    //picking_texture: TextureRc,
    camera: Camera,
    should_manage_input: bool,
    last_mouse_pos: Vector2,
    selected_object: ObjectId,
    view_width: u32,
    view_height: u32,
    gizmo: Option<GizmoRc>,
}
implement_widget_data!(View3DData);

pub struct View3D {
    ui_page: UIWidgetRc,
    shared_data: SharedDataRw,
    debug_drawer: DebugDrawer,
    bounding_box_drawer: BoundingBoxDrawer,
}

unsafe impl Send for View3D {}
unsafe impl Sync for View3D {}

impl View3D {
    pub fn new(
        shared_data: &SharedDataRw,
        global_messenger: &MessengerRw,
        default_material: &MaterialRc,
        wireframe_material: &MaterialRc,
    ) -> Self {
        let texture = Self::update_render_pass(
            shared_data,
            "MainPass",
            VIEW3D_IMAGE_WIDTH,
            VIEW3D_IMAGE_HEIGHT,
        );
        /*
        let picking_texture = Self::update_texture(
            shared_data,
            "PrePass",
            (VIEW3D_IMAGE_WIDTH as f32 * 0.5) as _,
            (VIEW3D_IMAGE_HEIGHT as f32 * 0.5) as _,
        );
        */
        let mut camera = Camera::new([10., 10., -10.].into(), [0., 0., 0.].into(), true);
        camera.set_projection(
            45.,
            VIEW3D_IMAGE_WIDTH as _,
            VIEW3D_IMAGE_HEIGHT as _,
            0.001,
            1000.,
        );

        let data = View3DData {
            shared_data: shared_data.clone(),
            global_messenger: global_messenger.clone(),
            texture,
            //picking_texture,
            camera,
            last_mouse_pos: Vector2::zero(),
            selected_object: INVALID_UID,
            view_width: VIEW3D_IMAGE_WIDTH,
            view_height: VIEW3D_IMAGE_HEIGHT,
            should_manage_input: false,
            gizmo: None,
        };
        let ui_page = Self::create(shared_data, data);
        Self {
            ui_page,
            debug_drawer: DebugDrawer::new(
                shared_data,
                global_messenger,
                default_material.resource().get().pipeline(),
                wireframe_material.resource().get().pipeline(),
            ),
            bounding_box_drawer: BoundingBoxDrawer::new(shared_data, global_messenger),
            shared_data: shared_data.clone(),
        }
    }

    pub fn update(&mut self) -> &mut Self {
        self.update_camera().update_gizmo();
        self.bounding_box_drawer.update();
        self.debug_drawer.update();

        self
    }

    fn update_gizmo(&mut self) -> &mut Self {
        if let Some(data) = self.ui_page.resource().get_mut().data_mut::<View3DData>() {
            if let Some(gizmo) = &data.gizmo {
                gizmo.resource().get_mut().update(&data.camera);
            }
        }
        self
    }

    pub fn change_edit_mode(
        &mut self,
        mode: EditMode,
        default_material_pipeline: &PipelineRc,
    ) -> &mut Self {
        if let Some(data) = self.ui_page.resource().get_mut().data_mut::<View3DData>() {
            match mode {
                EditMode::View => {
                    data.gizmo = None;
                }
                EditMode::Select => {
                    data.gizmo = None;
                }
                EditMode::Move => {
                    let gizmo = Gizmo::new_translation(
                        &data.shared_data,
                        data.global_messenger.clone(),
                        default_material_pipeline,
                    );
                    gizmo
                        .resource()
                        .get_mut()
                        .select_object(data.selected_object);
                    data.gizmo = Some(gizmo);
                }
                EditMode::Rotate => {
                    let gizmo = Gizmo::new_rotation(
                        &data.shared_data,
                        data.global_messenger.clone(),
                        default_material_pipeline,
                    );
                    gizmo
                        .resource()
                        .get_mut()
                        .select_object(data.selected_object);
                    data.gizmo = Some(gizmo);
                }
                EditMode::Scale => {
                    let gizmo = Gizmo::new_scale(
                        &data.shared_data,
                        data.global_messenger.clone(),
                        default_material_pipeline,
                    );
                    gizmo
                        .resource()
                        .get_mut()
                        .select_object(data.selected_object);
                    data.gizmo = Some(gizmo);
                }
            }
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
                movement.y -= 1.;
            } else if event.code == Key::E {
                movement.y += 1.;
            }
            if data.should_manage_input {
                data.camera.translate(movement);
            }
        }
    }

    fn resize_view(data: &mut View3DData, view_width: u32, view_height: u32) {
        if data.view_width != view_width && data.view_height != view_height {
            data.camera
                .set_projection(45., view_width as _, view_height as _, 0.001, 1000.);
        }
        data.view_width = view_width;
        data.view_height = view_height;
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

            data.global_messenger
                .read()
                .unwrap()
                .get_dispatcher()
                .write()
                .unwrap()
                .send(EditorEvent::Selected(data.selected_object).as_boxed())
                .ok();
        } else {
            let is_manipulating_gizmo = if let Some(gizmo) = &data.gizmo {
                gizmo.resource().get_mut().manipulate(
                    &data.camera,
                    data.last_mouse_pos,
                    normalized_pos,
                    is_drag_started,
                    is_drag_ended,
                    data.selected_object,
                )
            } else {
                false
            };
            if !is_manipulating_gizmo {
                let mut rotation_angle = Vector3::zero();

                rotation_angle.x = normalized_pos.y - data.last_mouse_pos.y;
                rotation_angle.y = data.last_mouse_pos.x - normalized_pos.x;
                data.camera.rotate(rotation_angle * 5.);
            }
        }
        data.last_mouse_pos = normalized_pos;
    }

    fn get_texture_index(shared_data: &SharedDataRw, texture_id: TextureId) -> usize {
        let mut texture_index = 0;
        if let Some(index) = SharedData::get_index_of_handle::<Texture>(shared_data, texture_id) {
            texture_index = index;
        }
        texture_index
    }

    fn create(shared_data: &SharedDataRw, data: View3DData) -> UIWidgetRc {
        UIWidget::register(shared_data, data, |ui_data, ui_context| {
            if let Some(data) = ui_data.as_any().downcast_mut::<View3DData>() {
                CentralPanel::default()
                    .frame(Frame::dark_canvas(ui_context.style().as_ref()))
                    .show(ui_context, |ui| {
                        data.should_manage_input = !ui.ctx().wants_keyboard_input();

                        let view_width = ui.max_rect_finite().width() as u32;
                        let view_height = ui.max_rect_finite().height() as u32;
                        let texture_index =
                            Self::get_texture_index(&data.shared_data, data.texture.id());
                        Self::resize_view(data, view_width, view_height);

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
            if let Some(view) =
                SharedData::match_resource(&self.shared_data, |view: &View| !view.id().is_nil())
            {
                let view_matrix = data.camera.view_matrix();
                let proj_matrix = data.camera.proj_matrix();

                view.resource()
                    .get_mut()
                    .update_view(view_matrix)
                    .update_proj(proj_matrix);
            }
        }
        self
    }

    fn update_render_pass(
        shared_data: &SharedDataRw,
        render_pass_name: &str,
        width: u32,
        height: u32,
    ) -> TextureRc {
        let image = DynamicImage::new_rgba8(width, height);
        let image_data = image.to_rgba8();
        let texture = Texture::create_from_data(shared_data, image_data);

        let render_pass_id = generate_uid_from_string(render_pass_name);
        if SharedData::has::<RenderPass>(shared_data, render_pass_id) {
            let render_pass = SharedData::get_handle::<RenderPass>(shared_data, render_pass_id);
            render_pass
                .resource()
                .get_mut()
                .set_color_texture(texture.clone())
                .add_category_to_draw(MeshCategoryId::new(DEFAULT_MESH_CATEGORY_IDENTIFIER));
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

        let mut min = [-5., -5., -5.].into();
        let mut max = [5., 5., 5.].into();
        let mut matrix = Matrix4::default_identity();
        SharedData::for_each_resource(&data.shared_data, |obj: &Resource<Object>| {
            if let Some(transform) = obj.get().get_component::<Transform>() {
                matrix = transform.resource().get().matrix();
            }
            if let Some(hitbox) = obj.get().get_component::<Hitbox>() {
                min = hitbox.resource().get().min();
                max = hitbox.resource().get().max();
            } else if let Some(mesh) = obj.get().get_component::<Mesh>() {
                let (mesh_min, mesh_max) = mesh.resource().get().mesh_data().compute_min_max();
                min = mesh_min;
                max = mesh_max;
            }
            if raycast_oob(ray_start_world.xyz(), ray_dir_world.xyz(), min, max, matrix) {
                selected_object = obj.id();
            }
        });

        selected_object
    }

    pub fn select_object(&mut self, object_id: ObjectId) {
        if let Some(data) = self.ui_page.resource().get_mut().data_mut::<View3DData>() {
            data.selected_object = object_id;
        }
    }
}
