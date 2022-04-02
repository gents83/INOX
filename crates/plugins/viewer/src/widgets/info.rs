use inox_core::ContextRc;
use inox_resources::Resource;
use inox_scene::SceneId;
use inox_ui::{implement_widget_data, UIWidget, Window};

use super::{Hierarchy, Meshes};

pub struct InfoParams {
    pub is_active: bool,
    pub scene_id: SceneId,
}

struct Data {
    context: ContextRc,
    params: InfoParams,
    hierarchy: (bool, Option<Hierarchy>),
    meshes: (bool, Option<Meshes>),
    fps: u32,
    dt: u128,
}
implement_widget_data!(Data);

pub struct Info {
    ui_page: Resource<UIWidget>,
}

impl Info {
    pub fn new(context: &ContextRc, params: InfoParams) -> Self {
        let data = Data {
            context: context.clone(),
            params,
            hierarchy: (false, None),
            meshes: (false, None),
            fps: 0,
            dt: 0,
        };
        Self {
            ui_page: Self::create(data),
        }
    }

    pub fn is_active(&self) -> bool {
        if let Some(data) = self.ui_page.get().data::<Data>() {
            return data.params.is_active;
        }
        false
    }
    pub fn set_active(&self, is_active: bool) {
        if let Some(data) = self.ui_page.get_mut().data_mut::<Data>() {
            data.params.is_active = is_active;
        }
    }
    pub fn set_scene_id(&self, scene_id: &SceneId) {
        if let Some(data) = self.ui_page.get_mut().data_mut::<Data>() {
            data.params.scene_id = *scene_id;
        }
    }

    pub fn update(&mut self) {
        inox_profiler::scoped_profile!("Info::update");

        if let Some(data) = self.ui_page.get_mut().data_mut::<Data>() {
            data.fps = data.context.global_timer().fps();
            data.dt = data.context.global_timer().dt().as_millis();

            if data.hierarchy.0 && data.hierarchy.1.is_none() {
                data.hierarchy.1 = Some(Hierarchy::new(
                    data.context.shared_data(),
                    data.context.message_hub(),
                    &data.params.scene_id,
                ));
            } else if !data.hierarchy.0 && data.hierarchy.1.is_some() {
                data.hierarchy.1 = None;
            }

            if data.meshes.0 && data.meshes.1.is_none() {
                data.meshes.1 = Some(Meshes::new(&data.context));
            } else if data.meshes.1.is_some() {
                if !data.meshes.0 {
                    data.meshes.1 = None;
                } else {
                    data.meshes.1.as_mut().unwrap().update();
                }
            }
        }
    }

    fn create(data: Data) -> Resource<UIWidget> {
        let shared_data = data.context.shared_data().clone();
        let message_hub = data.context.message_hub().clone();
        UIWidget::register(&shared_data, &message_hub, data, |ui_data, ui_context| {
            if let Some(data) = ui_data.as_any_mut().downcast_mut::<Data>() {
                if !data.params.is_active {
                    return;
                }
                Window::new("Stats")
                    .vscroll(true)
                    .title_bar(true)
                    .resizable(true)
                    .show(ui_context, |ui| {
                        ui.label(format!("FPS: {} - ms: {:?}", data.fps, data.dt));
                        ui.checkbox(&mut data.hierarchy.0, "Hierarchy");
                        ui.checkbox(&mut data.meshes.0, "Meshes");
                    });
            }
        })
    }
}
