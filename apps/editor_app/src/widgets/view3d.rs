use nrg_camera::Camera;
use nrg_graphics::{
    utils::create_cube_from_min_max, DynamicImage, MaterialInstance, MeshData, MeshInstance,
    MeshRc, PipelineInstance, RenderPassId, RenderPassInstance, TextureInstance, TextureRc,
    ViewInstance, DEFAULT_AREA_SIZE,
};
use nrg_math::{
    compute_distance_between_ray_and_oob, InnerSpace, Mat4Ops, MatBase, Matrix4, SquareMatrix,
    Vector2, Vector3, Vector4, Zero,
};
use nrg_messenger::{MessageBox, MessengerRw};
use nrg_platform::{Key, KeyEvent};
use nrg_resources::{DataTypeResource, SerializableResource, SharedData, SharedDataRw};
use nrg_scene::{Hitbox, Object, ObjectId};
use nrg_serialize::{generate_uid_from_string, INVALID_UID};
use nrg_ui::{
    implement_widget_data, CentralPanel, Frame, Image, LayerId, Sense, TextureId as eguiTextureId,
    UIWidget, UIWidgetRc, Widget,
};

const VIEW3D_IMAGE_WIDTH: u32 = 1280;
const VIEW3D_IMAGE_HEIGHT: u32 = 768;

struct View3DData {
    shared_data: SharedDataRw,
    global_dispatcher: MessageBox,
    render_pass_id: RenderPassId,
    texture: TextureRc,
    camera: Camera,
    last_mouse_pos: Vector2,
    selected_object: ObjectId,
    mesh_instance: MeshRc,
}
implement_widget_data!(View3DData);

pub struct View3D {
    ui_page: UIWidgetRc,
    shared_data: SharedDataRw,
}

unsafe impl Send for View3D {}
unsafe impl Sync for View3D {}

impl View3D {
    pub fn new(shared_data: &SharedDataRw, global_messenger: &MessengerRw) -> Self {
        let render_pass_id = generate_uid_from_string("MainPass");
        let texture = Self::update_texture(
            shared_data,
            render_pass_id,
            VIEW3D_IMAGE_WIDTH,
            VIEW3D_IMAGE_HEIGHT,
        );

        let mut camera = Camera::new([20., 20., -20.].into(), [0., 0., 0.].into(), true);
        camera.set_projection(
            45.,
            VIEW3D_IMAGE_WIDTH as _,
            VIEW3D_IMAGE_HEIGHT as _,
            0.001,
            1000.,
        );

        let mesh_instance = MeshInstance::create_from_data(shared_data, MeshData::default());
        if let Some(pipeline) = PipelineInstance::find_from_name(shared_data, "3D") {
            let material = MaterialInstance::create_from_pipeline(shared_data, pipeline);
            mesh_instance.resource().get_mut().set_material(material);
        }

        let data = View3DData {
            shared_data: shared_data.clone(),
            global_dispatcher: global_messenger.read().unwrap().get_dispatcher().clone(),
            render_pass_id,
            texture,
            camera,
            last_mouse_pos: Vector2::zero(),
            selected_object: INVALID_UID,
            mesh_instance,
        };
        let ui_page = Self::create(shared_data, data);
        Self {
            ui_page,
            shared_data: shared_data.clone(),
        }
    }

    pub fn update(&mut self) -> &mut Self {
        self.update_camera();
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
            }
            data.camera.translate(movement);
        }
    }

    fn create(shared_data: &SharedDataRw, data: View3DData) -> UIWidgetRc {
        UIWidget::register(shared_data, data, |ui_data, ui_context| {
            if let Some(data) = ui_data.as_any().downcast_mut::<View3DData>() {
                CentralPanel::default()
                    .frame(Frame::dark_canvas(ui_context.style().as_ref()))
                    .show(ui_context, |ui| {
                        let texture_id = data.texture.id();
                        let textures =
                            SharedData::get_resources_of_type::<TextureInstance>(&data.shared_data);
                        if let Some(index) = textures.iter().position(|t| t.id() == texture_id) {
                            let width = ui.max_rect_finite().width() as u32;
                            let height = ui.max_rect_finite().height() as u32;
                            let texture_width = data.texture.resource().get().width();
                            let texture_height = data.texture.resource().get().height();
                            if width <= DEFAULT_AREA_SIZE
                                && height <= DEFAULT_AREA_SIZE
                                && (texture_width != width || texture_height != height)
                            {
                                data.texture = Self::update_texture(
                                    &data.shared_data,
                                    data.render_pass_id,
                                    width,
                                    height,
                                );
                                data.camera.set_projection(
                                    45.,
                                    width as _,
                                    height as _,
                                    0.001,
                                    1000.,
                                );
                            }

                            ui.with_layer_id(LayerId::background(), |ui| {
                                let response = Image::new(
                                    eguiTextureId::User(index as _),
                                    [width as _, height as _],
                                )
                                .sense(Sense::click_and_drag())
                                .ui(ui);
                                if let Some(pos) = response.interact_pointer_pos() {
                                    let rect = response.rect;
                                    let normalized_x = (pos.x - rect.min.x) / rect.width();
                                    let normalized_y = (pos.y - rect.min.y) / rect.height();

                                    if data.last_mouse_pos.x < 0. || data.last_mouse_pos.y < 0. {
                                        data.last_mouse_pos = [normalized_x, normalized_y].into();
                                    }

                                    let mut rotation_angle = Vector3::zero();

                                    rotation_angle.x = normalized_y - data.last_mouse_pos.y;
                                    rotation_angle.y = data.last_mouse_pos.x - normalized_x;
                                    data.camera.rotate(rotation_angle * 5.);

                                    data.last_mouse_pos = [normalized_x, normalized_y].into();

                                    data.selected_object = Self::update_selected_object(
                                        data,
                                        normalized_x,
                                        normalized_y,
                                    );
                                } else {
                                    data.last_mouse_pos = [-1., -1.].into();
                                }
                            });
                        }
                    });
            }
        })
    }

    fn update_camera(&mut self) -> &mut Self {
        if let Some(data) = self.ui_page.resource().get_mut().data::<View3DData>() {
            if SharedData::has_resources_of_type::<ViewInstance>(&self.shared_data) {
                let views = SharedData::get_resources_of_type::<ViewInstance>(&self.shared_data);
                let view = views.first().unwrap();
                let view_matrix = data.camera.get_view_matrix();
                let proj_matrix = data.camera.get_proj_matrix();
                let width = data.texture.resource().get().width();
                let height = data.texture.resource().get().height();
                view.resource()
                    .get_mut()
                    .update_view(view_matrix)
                    .update_proj(proj_matrix)
                    .update_size(width, height);
            }
        }
        self
    }

    fn update_texture(
        shared_data: &SharedDataRw,
        render_pass_id: RenderPassId,
        width: u32,
        height: u32,
    ) -> TextureRc {
        let image = DynamicImage::new_rgba8(width, height);
        let image_data = image.to_rgba8();
        let texture = TextureInstance::create_from_data(shared_data, image_data);

        let render_pass =
            SharedData::get_resource::<RenderPassInstance>(shared_data, render_pass_id);
        render_pass
            .resource()
            .get_mut()
            .set_color_texture(texture.clone());

        texture
    }
    fn update_selected_object(
        data: &mut View3DData,
        normalized_x: f32,
        normalized_y: f32,
    ) -> ObjectId {
        let mut selected_object = INVALID_UID;

        let view = data.camera.get_view_matrix();
        let proj = data.camera.get_proj_matrix();

        // The ray Start and End positions, in Normalized Device Coordinates (Have you read Tutorial 4 ?)
        let ray_start = Vector4::new(0., 0., 0., 1.);
        let ray_end = Vector4::new(normalized_x * 2. - 1., normalized_y * 2. - 1., 1., 1.);

        let inv_proj = proj.invert().unwrap();
        let inv_view = view.invert().unwrap();

        let mut ray_start_camera = inv_proj * ray_start;
        ray_start_camera /= ray_start_camera.w;
        let mut ray_start_world = inv_view * ray_start_camera;
        ray_start_world /= ray_start_world.w;

        let mut ray_end_camera = inv_proj * ray_end;
        ray_end_camera /= ray_end_camera.w;
        let mut ray_end_world = inv_view * ray_end_camera;
        ray_end_world /= ray_end_world.w;

        let ray_dir_world = ray_end_world - ray_start_world;
        let ray_dir_world = ray_dir_world.normalize();

        if SharedData::has_resources_of_type::<Object>(&data.shared_data) {
            let objects = SharedData::get_resources_of_type::<Object>(&data.shared_data);
            for obj in objects {
                let mut min = [-5., -5., -5.].into();
                let mut max = [5., 5., 5.].into();
                if let Some(hitbox) = obj.resource().get().get_component::<Hitbox>() {
                    min = hitbox.resource().get().min();
                    max = hitbox.resource().get().max();
                } else if let Some(mesh) = obj.resource().get().get_component::<MeshInstance>() {
                    let transform = mesh.resource().get().transform();
                    let (mesh_min, mesh_max) = mesh.resource().get().mesh_data().compute_min_max();
                    min = transform.transform(mesh_min);
                    max = transform.transform(mesh_max);
                    /*
                    let mut mesh_data = MeshData::default();
                    let (vertices, indices) = create_cube_from_min_max(min, max);
                    mesh_data.append_mesh(&vertices, &indices);
                    data.mesh_instance
                        .resource()
                        .get_mut()
                        .set_mesh_data(mesh_data);
                    */
                }
                if compute_distance_between_ray_and_oob(
                    ray_start_world.xyz(),
                    ray_dir_world.xyz(),
                    min,
                    max,
                    Matrix4::default_identity(),
                ) {
                    println!("Inside {:?}", obj.resource().get().path());
                    selected_object = obj.id();
                    return selected_object;
                }
            }
        }
        selected_object
    }
}
