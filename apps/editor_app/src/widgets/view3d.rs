use nrg_graphics::{DynamicImage, RenderPassId, RenderPassInstance, TextureInstance, TextureRc};
use nrg_messenger::{MessageBox, MessengerRw};
use nrg_resources::{DataTypeResource, SharedData, SharedDataRw};
use nrg_serialize::generate_uid_from_string;
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
}

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

        let data = View3DData {
            shared_data: shared_data.clone(),
            global_dispatcher: global_messenger.read().unwrap().get_dispatcher().clone(),
            render_pass_id,
            texture,
        };
        let ui_page = Self::create(shared_data, data);
        Self { ui_page }
    }

    fn create(shared_data: &SharedDataRw, data: View3DData) -> UIWidgetRc {
        UIWidget::register(shared_data, data, |ui_data, ui_context| {
            if let Some(data) = ui_data.as_any().downcast_mut::<View3DData>() {
                CentralPanel::default().show(ui_context, |ui| {
                    let texture_id = data.texture.id();
                    let textures =
                        SharedData::get_resources_of_type::<TextureInstance>(&data.shared_data);
                    if let Some(index) = textures.iter().position(|t| t.id() == texture_id) {
                        ui.image(
                            eguiTextureId::User(index as _),
                            [VIEW3D_IMAGE_WIDTH, VIEW3D_IMAGE_HEIGHT],
                        );
                    }
                });
            }
        })
    }
}
