#![allow(dead_code)]

use inox_messenger::MessageHubRc;
use inox_resources::{Resource, SerializableResource, SharedData, SharedDataRc};
use inox_scene::{Object, ObjectId, Scene, SceneId};
use inox_ui::{
    collapsing_header::CollapsingState, implement_widget_data, CollapsingHeader, ScrollArea,
    SelectableLabel, UIWidget, Ui, Window,
};
use inox_uid::INVALID_UID;

struct HierarchyData {
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
    selected_object: ObjectId,
    scene: Resource<Scene>,
}
implement_widget_data!(HierarchyData);

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

    pub fn select_object(&mut self, object_id: ObjectId) -> &mut Self {
        if let Some(data) = self.ui_page.get_mut().data_mut::<HierarchyData>() {
            data.selected_object = object_id;
        }
        self
    }

    fn create(
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        data: HierarchyData,
    ) -> Resource<UIWidget> {
        UIWidget::register(shared_data, message_hub, data, |ui_data, ui_context| {
            if let Some(data) = ui_data.as_any_mut().downcast_mut::<HierarchyData>() {
                Window::new("Hierarchy")
                    .vscroll(true)
                    .title_bar(true)
                    .resizable(true)
                    .show(ui_context, |ui| {
                        let _ = &data;
                        CollapsingHeader::new("Scene")
                            .show_background(false)
                            .default_open(true)
                            .show(ui, |ui| {
                                let _ = &data;
                                ScrollArea::vertical().show(ui, |ui| {
                                    let _ = &data;
                                    data.scene.get().objects().iter().for_each(|object| {
                                        let _ = &data;
                                        Self::object_hierarchy(
                                            ui,
                                            object,
                                            &data.selected_object,
                                            &data.message_hub,
                                        );
                                    });
                                });
                            });
                    });
            }
        })
    }

    fn object_hierarchy(
        ui: &mut Ui,
        object: &Resource<Object>,
        selected_id: &ObjectId,
        message_hub: &MessageHubRc,
    ) {
        inox_profiler::scoped_profile!("object_hierarchy");

        let mut object_name = format!("Object [{:?}]", object.id().as_simple().to_string());
        if let Some(name) = object.get().path().file_stem() {
            if let Some(name) = name.to_str() {
                object_name = name.to_string();
            }
        }
        let is_selected = object.id() == selected_id;
        let is_child_recursive = object.get().is_child_recursive(selected_id);
        let has_children = object.get().has_children();

        if has_children {
            let id = ui.make_persistent_id(object_name.as_str());
            CollapsingState::load_with_default_open(ui.ctx(), id, true)
                .show_header(ui, |ui| {
                    let clicked = ui
                        .selectable_label(is_selected || is_child_recursive, object_name.as_str())
                        .clicked();
                    if clicked {
                        //change selected object
                    }
                })
                .body(|ui| {
                    object.get().children().iter().for_each(|child| {
                        Self::object_hierarchy(ui, child, selected_id, message_hub);
                    });
                });
            /*
            CollapsingHeader::new(object_name.as_str())
                //.selected(is_selected || is_child_recursive)
                .default_open(true)
                .show(ui, |ui| {
                    object.get().children().iter().for_each(|child| {
                        Self::object_hierarchy(ui, child, selected_id, message_hub);
                    });
                });
                response.header_response
                */
        } else {
            ui.add(SelectableLabel::new(is_selected, object_name.as_str()));
        }
    }
}
