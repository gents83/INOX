use nrg_messenger::{Message, MessageBox, MessengerRw};
use nrg_resources::{Resource, SerializableResource, SharedData, SharedDataRc};
use nrg_scene::{Object, ObjectId, Scene, SceneId};
use nrg_serialize::INVALID_UID;
use nrg_ui::{
    implement_widget_data, CollapsingHeader, ScrollArea, SelectableLabel, SidePanel, UIWidget, Ui,
};

use crate::EditorEvent;

struct HierarchyData {
    shared_data: SharedDataRc,
    global_dispatcher: MessageBox,
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
        global_messenger: &MessengerRw,
        scene_id: &SceneId,
    ) -> Self {
        if let Some(scene) = SharedData::get_resource::<Scene>(shared_data, scene_id) {
            let data = HierarchyData {
                shared_data: shared_data.clone(),
                global_dispatcher: global_messenger.read().unwrap().get_dispatcher().clone(),
                selected_object: INVALID_UID,
                scene,
            };
            return Self {
                ui_page: Self::create(shared_data, data),
            };
        }
        panic!("Hierarchy scene {:?} not found", scene_id);
    }

    pub fn select_object(&mut self, object_id: ObjectId) -> &mut Self {
        if let Some(data) = self.ui_page.get_mut().data_mut::<HierarchyData>() {
            data.selected_object = object_id;
        }
        self
    }

    fn create(shared_data: &SharedDataRc, data: HierarchyData) -> Resource<UIWidget> {
        UIWidget::register(shared_data, data, |ui_data, ui_context| {
            if let Some(data) = ui_data.as_any_mut().downcast_mut::<HierarchyData>() {
                SidePanel::left("Hierarchy")
                    .resizable(true)
                    .show(ui_context, |ui| {
                        let _ = &data;
                        ui.heading("Hierarchy:");

                        CollapsingHeader::new("Scene")
                            .selected(false)
                            .selectable(false)
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
                                            &data.global_dispatcher,
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
        global_dispatcher: &MessageBox,
    ) {
        nrg_profiler::scoped_profile!("object_hierarchy");

        let mut object_name = format!("Object [{:?}]", object.id().to_simple().to_string());
        if let Some(name) = object.get().path().file_stem() {
            if let Some(name) = name.to_str() {
                object_name = name.to_string();
            }
        }
        let is_selected = object.id() == selected_id;
        let is_child_recursive = object.get().is_child_recursive(selected_id);
        let has_children = object.get().has_children();

        let response = if has_children {
            let response = CollapsingHeader::new(object_name.as_str())
                .selected(is_selected || is_child_recursive)
                .selectable(true)
                .show_background(is_selected || is_child_recursive)
                .default_open(true)
                .show(ui, |ui| {
                    object.get().children().iter().for_each(|child| {
                        Self::object_hierarchy(ui, child, selected_id, global_dispatcher);
                    });
                });
            response.header_response
        } else {
            ui.add(SelectableLabel::new(is_selected, object_name.as_str()))
        };
        if response.clicked() {
            global_dispatcher
                .write()
                .unwrap()
                .send(EditorEvent::Selected(*object.id()).as_boxed())
                .ok();
        }
    }
}
