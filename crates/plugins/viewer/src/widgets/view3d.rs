use inox_graphics::BLIT_PASS_NAME;
use inox_render::RenderPass;

use inox_messenger::MessageHubRc;
use inox_resources::{Resource, SharedData, SharedDataRc};
use inox_ui::{
    implement_widget_data, load::SizedTexture, CentralPanel, Image, LayerId, Sense,
    TextureId as eguiTextureId, UIWidget, UiBuilder, Widget,
};

#[derive(Clone)]
struct View3DData {
    shared_data: SharedDataRc,
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
        let data = View3DData {
            shared_data: shared_data.clone(),
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
        message_hub: &MessageHubRc,
        data: View3DData,
    ) -> Resource<UIWidget> {
        UIWidget::register(shared_data, message_hub, data, |ui_data, ui_context| {
            if let Some(data) = ui_data.as_any_mut().downcast_mut::<View3DData>() {
                let response = CentralPanel::default().show(ui_context, |ui| {
                    let view_width = ui.max_rect().width() as u32;
                    let view_height = ui.max_rect().height() as u32;

                    let texture_uniform_index =
                        Self::get_render_pass_texture_index(&data.shared_data, 0, BLIT_PASS_NAME);

                    let response =
                        ui.scope_builder(UiBuilder::new().layer_id(LayerId::background()), |ui| {
                            let response = Image::from_texture(SizedTexture::new(
                                eguiTextureId::User(texture_uniform_index as _),
                                [view_width as _, view_height as _],
                            ))
                            .sense(Sense::click_and_drag())
                            .ui(ui);
                            data.is_interacting = response.is_pointer_button_down_on();
                            response
                        });
                    data.is_interacting |= response.response.is_pointer_button_down_on();
                    response
                });
                data.is_interacting |= response.response.is_pointer_button_down_on();
                return data.is_interacting;
            }
            false
        })
    }

    fn get_render_pass_texture_index(
        shared_data: &SharedDataRc,
        render_target_index: usize,
        render_pass_name: &str,
    ) -> i32 {
        if let Some(render_pass) =
            SharedData::match_resource(shared_data, |r: &RenderPass| r.name() == render_pass_name)
        {
            let render_pass = render_pass.get();
            let render_targets = render_pass.render_textures();
            if render_target_index < render_targets.len() {
                return render_targets[render_target_index].get().texture_index();
            }
        }
        0
    }
}
