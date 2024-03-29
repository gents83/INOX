#![allow(dead_code)]

use inox_messenger::MessageHubRc;
use inox_resources::{Resource, SerializableResource, SharedData, SharedDataRc};
use inox_scene::{Object, ObjectId, Scene, SceneId};
use inox_ui::{
    collapsing_header::CollapsingState, implement_widget_data, CollapsingHeader, ScrollArea,
    UIWidget, Ui, Window,
};
use inox_uid::INVALID_UID;

use crate::events::WidgetEvent;

#[derive(Clone)]
struct HierarchyData {
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
    selected_object: ObjectId,
    scene: Resource<Scene>,
}
implement_widget_data!(HierarchyData);

#[derive(Clone)]
pub struct Hierarchy {
    ui_page: Resource<UIWidget>,
}

impl Hierarchy {
    pub fn new(
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        scene_id: &SceneId,
    ) -> Option<Self> {
        if let Some(scene) = SharedData::get_resource::<Scene>(shared_data, scene_id) {
            let data = HierarchyData {
                shared_data: shared_data.clone(),
                message_hub: message_hub.clone(),
                selected_object: INVALID_UID,
                scene,
            };
            return Some(Self {
                ui_page: Self::create(shared_data, message_hub, data),
            });
        }
        None
    }

    fn create(
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        data: HierarchyData,
    ) -> Resource<UIWidget> {
        UIWidget::register(shared_data, message_hub, data, |ui_data, ui_context| {
            if let Some(data) = ui_data.as_any_mut().downcast_mut::<HierarchyData>() {
                if let Some(response) = Window::new("Hierarchy")
                    .vscroll(true)
                    .title_bar(true)
                    .resizable(true)
                    .show(ui_context, |ui| {
                        CollapsingHeader::new("Scene")
                            .show_background(false)
                            .default_open(true)
                            .show(ui, |ui| {
                                ScrollArea::vertical().show(ui, |ui| {
                                    let objects = data.scene.get().objects().clone();
                                    objects.iter().for_each(|object| {
                                        Self::object_hierarchy(ui, object, data);
                                    });
                                })
                            })
                    })
                {
                    return response.response.is_pointer_button_down_on();
                }
            }
            false
        })
    }

    fn select_object(data: &mut HierarchyData, object_id: &ObjectId) {
        if data.selected_object == *object_id {
            data.selected_object = INVALID_UID;
        } else {
            data.selected_object = *object_id;
        }
        data.message_hub
            .send_event(WidgetEvent::Selected(data.selected_object))
    }

    fn object_hierarchy(ui: &mut Ui, object: &Resource<Object>, data: &mut HierarchyData) {
        inox_profiler::scoped_profile!("object_hierarchy");

        let mut object_name = format!("Object [{:?}]", object.id().as_simple().to_string());
        if let Some(name) = object.get().path().file_stem() {
            if let Some(name) = name.to_str() {
                object_name = name.to_string();
            }
        }
        let is_selected = object.id() == &data.selected_object;
        let has_children = object.get().has_children();

        if has_children {
            let id = ui.make_persistent_id(object_name.as_str());
            CollapsingState::load_with_default_open(ui.ctx(), id, true)
                .show_header(ui, |ui| {
                    let mut s = is_selected;
                    if ui.toggle_value(&mut s, object_name.as_str()).clicked() {
                        //change selected object
                        Self::select_object(data, object.id());
                    }
                })
                .body(|ui| {
                    object.get().children().iter().for_each(|child| {
                        Self::object_hierarchy(ui, child, data);
                    });
                });
        } else {
            let mut s = is_selected;
            if ui.toggle_value(&mut s, object_name.as_str()).clicked() {
                //change selected object
                Self::select_object(data, object.id());
            }
        }
    }
}
