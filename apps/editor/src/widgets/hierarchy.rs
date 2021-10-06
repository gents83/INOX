use nrg_messenger::{Message, MessageBox, MessengerRw};
use nrg_resources::{Resource, ResourceData, SerializableResource, SharedData, SharedDataRw};
use nrg_scene::{Object, ObjectId, Scene, SceneId};
use nrg_serialize::INVALID_UID;
use nrg_ui::{
    implement_widget_data, CollapsingHeader, ScrollArea, SelectableLabel, SidePanel, UIWidget, Ui,
};

use crate::EditorEvent;

struct HierarchyData {
    shared_data: SharedDataRw,
    global_dispatcher: MessageBox,
    selected_object: ObjectId,
    scene_id: SceneId,
}
implement_widget_data!(HierarchyData);

pub struct Hierarchy {
    ui_page: Resource<UIWidget>,
}

impl Hierarchy {
    pub fn new(
        shared_data: &SharedDataRw,
        global_messenger: &MessengerRw,
        scene_id: SceneId,
    ) -> Self {
        let data = HierarchyData {
            shared_data: shared_data.clone(),
            global_dispatcher: global_messenger.read().unwrap().get_dispatcher().clone(),
            selected_object: INVALID_UID,
            scene_id,
        };
        Self {
            ui_page: Self::create(shared_data, data),
        }
    }

    pub fn select_object(&mut self, object_id: ObjectId) -> &mut Self {
        if let Some(data) = self.ui_page.get_mut().data_mut::<HierarchyData>() {
            data.selected_object = object_id;
        }
        self
    }

    fn create(shared_data: &SharedDataRw, data: HierarchyData) -> Resource<UIWidget> {
        UIWidget::register(shared_data, data, |ui_data, ui_context| {
            if let Some(data) = ui_data.as_any().downcast_mut::<HierarchyData>() {
                SidePanel::left("Hierarchy")
                    .resizable(true)
                    .show(ui_context, |ui| {
                        ui.heading("Hierarchy:");

                        CollapsingHeader::new("Scene")
                            .selected(false)
                            .selectable(false)
                            .show_background(false)
                            .default_open(true)
                            .show(ui, |ui| {
                                ScrollArea::vertical().show(ui, |ui| {
                                    let scene = SharedData::get_resource::<Scene>(
                                        &data.shared_data,
                                        data.scene_id,
                                    );
                                    let scene = scene.get();
                                    let objects = scene.objects();
                                    objects.iter().for_each(|object| {
                                        Self::object_hierarchy(
                                            ui,
                                            object,
                                            data.selected_object,
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
        selected_id: ObjectId,
        global_dispatcher: &MessageBox,
    ) {
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
                    let object = object.get();
                    let children = object.children();
                    children.iter().for_each(|child| {
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
                .send(EditorEvent::Selected(object.id()).as_boxed())
                .ok();
        }
    }
}