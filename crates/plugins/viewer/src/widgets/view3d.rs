use sabi_graphics::{RenderPass, Texture};

use sabi_messenger::MessageHubRc;
use sabi_resources::{Handle, Resource, SharedData, SharedDataRc};
use sabi_ui::{
    implement_widget_data, CentralPanel, Frame, Image, LayerId, Sense, TextureId as eguiTextureId,
    UIWidget, Widget,
};

struct View3DData {
    shared_data: SharedDataRc,
    texture: Handle<Texture>,
    is_interacting: bool,
}
implement_widget_data!(View3DData);

pub struct View3D {
    _ui_page: Resource<UIWidget>,
}

unsafe impl Send for View3D {}
unsafe impl Sync for View3D {}

impl View3D {
    pub fn new(shared_data: &SharedDataRc, message_hub: &MessageHubRc) -> Self {
        let texture = Self::get_render_pass_output_texture(shared_data, "MainPass");

        let data = View3DData {
            shared_data: shared_data.clone(),
            texture,
            is_interacting: false,
        };
        let ui_page = Self::create(shared_data, message_hub, data);
        Self { _ui_page: ui_page }
    }

    pub fn is_interacting(&self) -> bool {
        if let Some(data) = self._ui_page.get().data::<View3DData>() {
            data.is_interacting
        } else {
            false
        }
    }

    fn create(
        shared_data: &SharedDataRc,
        messenger: &MessageHubRc,
        data: View3DData,
    ) -> Resource<UIWidget> {
        UIWidget::register(shared_data, messenger, data, |ui_data, ui_context| {
            if let Some(data) = ui_data.as_any_mut().downcast_mut::<View3DData>() {
                CentralPanel::default()
                    .frame(Frame::dark_canvas(ui_context.style().as_ref()))
                    .show(ui_context, |ui| {
                        let view_width = ui.max_rect().width() as u32;
                        let view_height = ui.max_rect().height() as u32;

                        let texture_uniform_index = if let Some(t) = &data.texture {
                            t.get().uniform_index()
                        } else {
                            0
                        };

                        ui.with_layer_id(LayerId::background(), |ui| {
                            let response = Image::new(
                                eguiTextureId::User(texture_uniform_index as _),
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

    fn get_render_pass_output_texture(
        shared_data: &SharedDataRc,
        render_pass_name: &str,
    ) -> Handle<Texture> {
        if let Some(render_pass) = SharedData::match_resource(shared_data, |r: &RenderPass| {
            r.data().name == render_pass_name
        }) {
            return render_pass.get().render_target().clone();
        }
        None
    }
}
