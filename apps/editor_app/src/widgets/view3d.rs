use nrg_camera::Camera;
use nrg_graphics::{
    DynamicImage, RenderPassId, RenderPassInstance, TextureInstance, TextureRc, ViewInstance,
};
use nrg_math::{
    compute_distance_between_ray_and_oob, InnerSpace, MatBase, Matrix4, SquareMatrix, Vector2,
    Vector3, Vector4, Zero,
};
use nrg_messenger::{MessageBox, MessengerRw};
use nrg_platform::{Key, KeyEvent, MouseButton, MouseEvent, MouseState};
use nrg_resources::{DataTypeResource, SharedData, SharedDataRw};
use nrg_scene::ObjectId;
use nrg_serialize::{generate_uid_from_string, INVALID_UID};
use nrg_ui::{
    implement_widget_data, CentralPanel, TextureId as eguiTextureId, UIWidget, UIWidgetRc,
};

const VIEW3D_IMAGE_WIDTH: f32 = 1280.;
const VIEW3D_IMAGE_HEIGHT: f32 = 768.;

struct View3DData {
    shared_data: SharedDataRw,
    global_dispatcher: MessageBox,
    render_pass_id: RenderPassId,
    texture: TextureRc,
}
implement_widget_data!(View3DData);

pub struct View3D {
    ui_page: UIWidgetRc,
    shared_data: SharedDataRw,
    camera: Camera,
    move_camera_with_mouse: bool,
    last_mouse_pos: Vector2,
    selected_object: ObjectId,
}

unsafe impl Send for View3D {}
unsafe impl Sync for View3D {}

impl View3D {
    pub fn new(shared_data: &SharedDataRw, global_messenger: &MessengerRw) -> Self {
        let image = DynamicImage::new_rgba8(VIEW3D_IMAGE_WIDTH as _, VIEW3D_IMAGE_HEIGHT as _);
        let image_data = image.to_rgba8();
        let texture = TextureInstance::create_from_data(shared_data, image_data);

        let render_pass_id = generate_uid_from_string("MainPass");
        let render_pass =
            SharedData::get_resource::<RenderPassInstance>(shared_data, render_pass_id);
        render_pass
            .resource()
            .get_mut()
            .set_color_texture(texture.clone());

        let mut camera = Camera::new([20., 20., -20.].into(), [0., 0., 0.].into(), true);
        camera.set_projection(
            45.,
            VIEW3D_IMAGE_WIDTH as _,
            VIEW3D_IMAGE_HEIGHT as _,
            0.1,
            1000.,
        );

        let data = View3DData {
            shared_data: shared_data.clone(),
            global_dispatcher: global_messenger.read().unwrap().get_dispatcher().clone(),
            render_pass_id,
            texture,
        };
        let ui_page = Self::create(shared_data, data);
        Self {
            ui_page,
            shared_data: shared_data.clone(),
            camera,
            move_camera_with_mouse: false,
            last_mouse_pos: Vector2::zero(),
            selected_object: INVALID_UID,
        }
    }

    pub fn update(&mut self) -> &mut Self {
        self.update_camera();
        self
    }
    pub fn handle_keyboard_event(&mut self, event: &KeyEvent) {
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
        self.camera.translate(movement);
    }

    pub fn handle_mouse_event(&mut self, event: &MouseEvent) {
        if event.state == MouseState::Down && event.button == MouseButton::Left {
            self.move_camera_with_mouse = true;
            self.last_mouse_pos = [event.x as f32, event.y as f32].into();
        } else if event.state == MouseState::Up && event.button == MouseButton::Left {
            let mouse_pos = [event.x as f32, event.y as f32].into();
            self.update_selected_object(&mouse_pos);

            self.move_camera_with_mouse = false;
            self.last_mouse_pos = mouse_pos;
        }
        if event.state == MouseState::Move && self.move_camera_with_mouse {
            let mut rotation_angle = Vector3::zero();

            rotation_angle.x = event.y as f32 - self.last_mouse_pos.y;
            rotation_angle.y = self.last_mouse_pos.x - event.x as f32;

            self.camera.rotate(rotation_angle * 0.01);

            self.last_mouse_pos = [event.x as f32, event.y as f32].into();
        }
    }

    fn create(shared_data: &SharedDataRw, data: View3DData) -> UIWidgetRc {
        UIWidget::register(shared_data, data, |ui_data, ui_context| {
            if let Some(data) = ui_data.as_any().downcast_mut::<View3DData>() {
                CentralPanel::default().show(ui_context, |ui| {
                    let texture_id = data.texture.id();
                    let textures =
                        SharedData::get_resources_of_type::<TextureInstance>(&data.shared_data);
                    if let Some(index) = textures.iter().position(|t| t.id() == texture_id) {
                        let width = ui.max_rect_finite().width();
                        let height = ui.max_rect_finite().height();
                        ui.image(eguiTextureId::User(index as _), [width, height]);
                    }
                });
            }
        })
    }

    fn update_camera(&mut self) -> &mut Self {
        if SharedData::has_resources_of_type::<ViewInstance>(&self.shared_data) {
            let views = SharedData::get_resources_of_type::<ViewInstance>(&self.shared_data);
            let view = views.first().unwrap();
            let view_matrix = self.camera.get_view_matrix();
            let proj_matrix = self.camera.get_proj_matrix();
            view.resource()
                .get_mut()
                .update_view(view_matrix)
                .update_proj(proj_matrix)
                .update_size(VIEW3D_IMAGE_WIDTH, VIEW3D_IMAGE_HEIGHT);
        }
        self
    }

    fn update_selected_object(&mut self, mouse_pos: &Vector2) -> &mut Self {
        self.selected_object = INVALID_UID;
        let view = self.camera.get_view_matrix();
        let proj = self.camera.get_proj_matrix();

        let screen_size: Vector2 = [VIEW3D_IMAGE_WIDTH as f32, VIEW3D_IMAGE_HEIGHT as f32].into();
        // The ray Start and End positions, in Normalized Device Coordinates (Have you read Tutorial 4 ?)
        let ray_start = Vector4::new(0., 0., 0., 1.);
        let ray_end = Vector4::new(
            ((mouse_pos.x / screen_size.x) * 2.) - 1.,
            ((mouse_pos.y / screen_size.y) * 2.) - 1.,
            1.,
            1.,
        );

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

        if compute_distance_between_ray_and_oob(
            ray_start_world.xyz(),
            ray_dir_world.xyz(),
            [-5., -5., -5.].into(),
            [5., 5., 5.].into(),
            Matrix4::default_identity(),
        ) {
            println!("Inside");
        } else {
            println!("Outside");
        }

        self
    }
}
