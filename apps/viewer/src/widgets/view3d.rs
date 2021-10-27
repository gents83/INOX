use nrg_graphics::{
    DynamicImage, MeshCategoryId, RenderPass, Texture, DEFAULT_MESH_CATEGORY_IDENTIFIER,
};
use nrg_messenger::MessengerRw;
use nrg_resources::{DataTypeResource, Resource, SharedData, SharedDataRc};
use nrg_serialize::generate_random_uid;
use nrg_ui::{
    implement_widget_data, CentralPanel, Frame, Image, LayerId, Sense, TextureId as eguiTextureId,
    UIWidget, Widget,
};
const VIEW3D_IMAGE_WIDTH: u32 = 1920;
const VIEW3D_IMAGE_HEIGHT: u32 = 1080;

struct View3DData {
    shared_data: SharedDataRc,
    texture: Resource<Texture>,
    is_interacting: bool,
}
implement_widget_data!(View3DData);

pub struct View3D {
    _ui_page: Resource<UIWidget>,
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

        let data = View3DData {
            shared_data: shared_data.clone(),
            texture,
            is_interacting: false,
        };
        let ui_page = Self::create(shared_data, data);
        Self { _ui_page: ui_page }
    }

    pub fn is_interacting(&self) -> bool {
        self._ui_page.get(|p| {
            if let Some(data) = p.data::<View3DData>() {
                return data.is_interacting;
            }
            false
        })
    }

    fn create(shared_data: &SharedDataRc, data: View3DData) -> Resource<UIWidget> {
        UIWidget::register(shared_data, data, |ui_data, ui_context| {
            if let Some(data) = ui_data.as_any_mut().downcast_mut::<View3DData>() {
                CentralPanel::default()
                    .frame(Frame::dark_canvas(ui_context.style().as_ref()))
                    .show(ui_context, |ui| {
                        let view_width = ui.max_rect().width() as u32;
                        let view_height = ui.max_rect().height() as u32;

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
                                [view_width as _, view_height as _],
                            )
                            .sense(Sense::click_and_drag())
                            .ui(ui);
                            data.is_interacting = response.is_pointer_button_down_on();
                        })
                    });
            }
        })
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
}
