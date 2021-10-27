use nrg_graphics::{
    utils::compute_id_from_color, DynamicImage, Mesh, MeshCategoryId, Pipeline, PipelineType,
    RenderPass, Texture, View, DEFAULT_MESH_CATEGORY_IDENTIFIER, TEXTURE_CHANNEL_COUNT,
};
use nrg_math::{
    raycast_oob, Degrees, InnerSpace, MatBase, Matrix4, NewAngle, Vector2, Vector3, Vector4, Zero,
};
use nrg_messenger::{Message, MessengerRw};
use nrg_platform::{Key, KeyEvent};
use nrg_resources::{DataTypeResource, Handle, Resource, SharedData, SharedDataRc};
use nrg_scene::{
    Camera, Hitbox, Object, ObjectId, DEFAULT_CAMERA_FAR, DEFAULT_CAMERA_FOV, DEFAULT_CAMERA_NEAR,
};
use nrg_serialize::{generate_random_uid, INVALID_UID};
use nrg_ui::{
    implement_widget_data, CentralPanel, Frame, Image, LayerId, Sense, TextureId as eguiTextureId,
    UIWidget, Widget,
};

use crate::{resources::Gizmo, systems::BoundingBoxDrawer, EditMode, EditorEvent};

const VIEW3D_IMAGE_WIDTH: u32 = 1280;
const VIEW3D_IMAGE_HEIGHT: u32 = 768;
const PICKING_TEXTURE_WIDTH: u32 = VIEW3D_IMAGE_WIDTH / 2;
const PICKING_TEXTURE_HEIGHT: u32 = VIEW3D_IMAGE_HEIGHT / 2;

struct View3DData {
    shared_data: SharedDataRc,
    global_messenger: MessengerRw,
    texture: Resource<Texture>,
    picking_texture: Resource<Texture>,
    camera_object: Resource<Object>,
    should_manage_input: bool,
    last_mouse_pos: Vector2,
    selected_object: ObjectId,
    hover_mesh: u32,
    view_width: u32,
    view_height: u32,
    gizmo: Handle<Gizmo>,
}
implement_widget_data!(View3DData);

pub struct View3D {
    ui_page: Resource<UIWidget>,
    shared_data: SharedDataRc,
    bounding_box_drawer: BoundingBoxDrawer,
}

unsafe impl Send for View3D {}
unsafe impl Sync for View3D {}

impl View3D {
    pub fn new(shared_data: &SharedDataRc, global_messenger: &MessengerRw) -> Self {
        let texture = Self::update_render_pass(
            shared_data,
            global_messenger,
            "MainPass",
            VIEW3D_IMAGE_WIDTH,
            VIEW3D_IMAGE_HEIGHT,
        );

        let picking_texture = Self::update_render_pass(
            shared_data,
            global_messenger,
            "PickingPass",
            PICKING_TEXTURE_WIDTH,
            PICKING_TEXTURE_HEIGHT,
        );

        let camera_object = SharedData::add_resource::<Object>(
            &shared_data,
            generate_random_uid(),
            Object::default(),
        );
        camera_object.get_mut(|o| {
            o.translate([10., 10., -10.].into());
            o.look_at([0., 0., 0.].into());
            let camera = o.add_default_component::<Camera>(&shared_data);
            camera.get_mut(|c| {
                c.set_parent(&camera_object)
                    .set_active(true)
                    .set_projection(
                        Degrees::new(DEFAULT_CAMERA_FOV),
                        VIEW3D_IMAGE_WIDTH as _,
                        VIEW3D_IMAGE_HEIGHT as _,
                        DEFAULT_CAMERA_NEAR,
                        DEFAULT_CAMERA_FAR,
                    );
            });
        });

        let data = View3DData {
            shared_data: shared_data.clone(),
            global_messenger: global_messenger.clone(),
            texture,
            picking_texture,
            camera_object,
            last_mouse_pos: Vector2::zero(),
            selected_object: INVALID_UID,
            hover_mesh: 0,
            view_width: VIEW3D_IMAGE_WIDTH,
            view_height: VIEW3D_IMAGE_HEIGHT,
            should_manage_input: false,
            gizmo: None,
        };
        let ui_page = Self::create(shared_data, data);
        Self {
            ui_page,
            bounding_box_drawer: BoundingBoxDrawer::new(shared_data, global_messenger),
            shared_data: shared_data.clone(),
        }
    }

    pub fn update(&mut self) -> &mut Self {
        self.update_camera().update_gizmo();
        self.bounding_box_drawer.update();

        self
    }

    fn update_gizmo(&mut self) -> &mut Self {
        self.ui_page.get_mut(|w| {
            if let Some(data) = w.data_mut::<View3DData>() {
                if let Some(gizmo) = &data.gizmo {
                    gizmo.get_mut(|g| {
                        if let Some(camera) =
                            data.camera_object.get(|o| o.get_component::<Camera>())
                        {
                            camera.get(|c| {
                                g.update(c);
                            });
                        }
                    });
                }
            }
        });
        self
    }

    pub fn change_edit_mode(&mut self, mode: EditMode) -> &mut Self {
        let default_pipeline = self
            .shared_data
            .match_resource(|p: &Pipeline| p.data().pipeline_type == PipelineType::Default);
        self.ui_page.get_mut(|w| {
            if let Some(data) = w.data_mut::<View3DData>() {
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
                            &data.global_messenger,
                            default_pipeline.as_ref().unwrap(),
                        );
                        gizmo.get_mut(|g| g.select_object(&data.selected_object));
                        data.gizmo = Some(gizmo);
                    }
                    EditMode::Rotate => {
                        let gizmo = Gizmo::new_rotation(
                            &data.shared_data,
                            &data.global_messenger,
                            default_pipeline.as_ref().unwrap(),
                        );
                        gizmo.get_mut(|g| g.select_object(&data.selected_object));
                        data.gizmo = Some(gizmo);
                    }
                    EditMode::Scale => {
                        let gizmo = Gizmo::new_scale(
                            &data.shared_data,
                            &data.global_messenger,
                            default_pipeline.as_ref().unwrap(),
                        );
                        gizmo.get_mut(|g| g.select_object(&data.selected_object));
                        data.gizmo = Some(gizmo);
                    }
                }
            }
        });
        self
    }
    pub fn handle_keyboard_event(&mut self, event: &KeyEvent) {
        self.ui_page.get_mut(|w| {
            if let Some(data) = w.data_mut::<View3DData>() {
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
                    data.camera_object.get_mut(|o| {
                        o.translate(movement);
                    });
                }
            }
        });
    }

    fn resize_view(data: &mut View3DData, view_width: u32, view_height: u32) {
        if data.view_width != view_width && data.view_height != view_height {
            data.shared_data.for_each_resource_mut(|_, c: &mut Camera| {
                c.set_projection(
                    Degrees::new(45.),
                    view_width as _,
                    view_height as _,
                    0.001,
                    1000.,
                );
            });
        }
        data.view_width = view_width;
        data.view_height = view_height;
    }

    fn manage_picking_texture(data: &mut View3DData, normalized_pos: Vector2) {
        let texture = data.picking_texture.clone();
        texture.get(|t| {
            if let Some(image_buffer) = t.image_data() {
                let x = ((normalized_pos.x * PICKING_TEXTURE_WIDTH as f32) as u32)
                    .max(0)
                    .min(PICKING_TEXTURE_WIDTH - 1);
                let y = ((normalized_pos.y * PICKING_TEXTURE_HEIGHT as f32) as u32)
                    .max(0)
                    .min(PICKING_TEXTURE_HEIGHT - 1);
                let index = (TEXTURE_CHANNEL_COUNT * (y * PICKING_TEXTURE_WIDTH + x)) as usize;
                let color = Vector4::new(
                    image_buffer[index] as f32 / 255.,
                    image_buffer[index + 1] as f32 / 255.,
                    image_buffer[index + 2] as f32 / 255.,
                    image_buffer[index + 3] as f32 / 255.,
                );
                data.hover_mesh = compute_id_from_color(color);
                data.global_messenger
                    .read()
                    .unwrap()
                    .get_dispatcher()
                    .write()
                    .unwrap()
                    .send(EditorEvent::HoverMesh(data.hover_mesh).as_boxed())
                    .ok();
            }
        });
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
                gizmo.get_mut(|g| {
                    if let Some(camera) = data.camera_object.get(|o| o.get_component::<Camera>()) {
                        camera.get(|c| {
                            return g.manipulate(
                                c,
                                data.last_mouse_pos,
                                normalized_pos,
                                is_drag_started,
                                is_drag_ended,
                                &data.selected_object,
                            );
                        });
                    }
                    false
                })
            } else {
                false
            };
            if !is_manipulating_gizmo {
                let mut rotation_angle = Vector3::zero();

                rotation_angle.x = normalized_pos.y - data.last_mouse_pos.y;
                rotation_angle.y = data.last_mouse_pos.x - normalized_pos.x;
                data.camera_object.get_mut(|o| {
                    o.rotate(rotation_angle * 5.);
                });
            }
        }
        data.last_mouse_pos = normalized_pos;
    }

    fn create(shared_data: &SharedDataRc, data: View3DData) -> Resource<UIWidget> {
        UIWidget::register(shared_data, data, |ui_data, ui_context| {
            if let Some(data) = ui_data.as_any().downcast_mut::<View3DData>() {
                CentralPanel::default()
                    .frame(Frame::dark_canvas(ui_context.style().as_ref()))
                    .show(ui_context, |ui| {
                        data.should_manage_input = !ui.ctx().wants_keyboard_input();

                        let view_width = ui.max_rect().width() as u32;
                        let view_height = ui.max_rect().height() as u32;
                        Self::resize_view(data, view_width, view_height);

                        let texture_index = if let Some(texture_index) =
                            SharedData::get_index_of_resource::<Texture>(
                                &data.shared_data,
                                &data.texture.id(),
                            ) {
                            texture_index
                        } else {
                            0
                        };

                        ui.with_layer_id(LayerId::background(), |ui| {
                            let response = Image::new(
                                eguiTextureId::User(texture_index as _),
                                [data.view_width as _, data.view_height as _],
                            )
                            .sense(Sense::click_and_drag())
                            .ui(ui);

                            let rect = response.rect;

                            if let Some(pos) = response.interact_pointer_pos() {
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
                                if let Some(pos) = response.hover_pos() {
                                    let normalized_x = (pos.x - rect.min.x) / rect.width();
                                    let normalized_y = (pos.y - rect.min.y) / rect.height();
                                    let new_pos = [normalized_x, normalized_y].into();
                                    Self::manage_picking_texture(data, new_pos);
                                }
                                data.last_mouse_pos = [-1., -1.].into();
                            }
                            response
                        })
                    });
            }
        })
    }

    fn update_camera(&mut self) -> &mut Self {
        self.ui_page.get_mut(|w| {
            if let Some(data) = w.data_mut::<View3DData>() {
                if let Some(view) = SharedData::match_resource(&self.shared_data, |view: &View| {
                    view.view_index() == 0
                }) {
                    data.shared_data.for_each_resource(|_, c: &Camera| {
                        if c.is_active() {
                            view.get_mut(|v| {
                                v.update_view(c.view_matrix()).update_proj(c.proj_matrix());
                            });
                        }
                    });
                }
            }
        });
        self
    }

    fn update_render_pass(
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        render_pass_name: &str,
        width: u32,
        height: u32,
    ) -> Resource<Texture> {
        let image = DynamicImage::new_rgba8(width, height);
        let image_data = image.to_rgba8();
        let texture = Texture::create_from_data(
            shared_data,
            global_messenger,
            generate_random_uid(),
            image_data,
        );

        if let Some(render_pass) = SharedData::match_resource(shared_data, |r: &RenderPass| {
            r.data().name == render_pass_name
        }) {
            render_pass.get_mut(|r| {
                r.set_color_texture(texture.clone())
                    .add_category_to_draw(MeshCategoryId::new(DEFAULT_MESH_CATEGORY_IDENTIFIER));
            });
        }

        texture
    }

    fn update_selected_object(
        data: &mut View3DData,
        normalized_x: f32,
        normalized_y: f32,
    ) -> ObjectId {
        let mut selected_object = INVALID_UID;

        let mut ray_start_world = Vector3::zero();
        let mut ray_end_world = Vector3::zero();
        data.shared_data.for_each_resource(|_, c: &Camera| {
            if c.is_active() {
                let (start, end) = c.convert_in_3d([normalized_x, normalized_y].into());
                ray_start_world = start;
                ray_end_world = end;
            }
        });

        let ray_dir_world = ray_end_world - ray_start_world;
        let ray_dir_world = ray_dir_world.normalize();

        let mut min = [-5., -5., -5.].into();
        let mut max = [5., 5., 5.].into();
        let mut matrix = Matrix4::default_identity();
        SharedData::for_each_resource(&data.shared_data, |object_handle, obj: &Object| {
            matrix = obj.transform();
            if let Some(hitbox) = obj.get_component::<Hitbox>() {
                min = hitbox.get(|h| h.min());
                max = hitbox.get(|h| h.max());
            } else if let Some(mesh) = obj.get_component::<Mesh>() {
                let (mesh_min, mesh_max) = mesh.get(|m| m.mesh_data().compute_min_max());
                min = mesh_min;
                max = mesh_max;
            }
            if raycast_oob(ray_start_world.xyz(), ray_dir_world.xyz(), min, max, matrix) {
                selected_object = *object_handle.id();
            }
        });

        selected_object
    }

    pub fn select_object(&mut self, object_id: ObjectId) {
        self.ui_page.get_mut(|w| {
            if let Some(data) = w.data_mut::<View3DData>() {
                data.selected_object = object_id;
            }
        });
    }
}
