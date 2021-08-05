use std::{
    path::{Path, PathBuf},
    process::Command,
};

use nrg_filesystem::{
    convert_from_local_path, for_each_file_in, for_each_folder_in, is_folder_empty,
};
use nrg_graphics::{TextureId, TextureInstance, TextureRc};
use nrg_messenger::{get_events_from_string, Message, MessageBox, MessengerRw};

use nrg_resources::{FileResource, SharedData, SharedDataRw, DATA_FOLDER};
use nrg_serialize::deserialize;
use nrg_ui::{
    implement_widget_data, menu, Align, CentralPanel, CollapsingHeader, DialogEvent, DialogOp,
    Layout, ScrollArea, SidePanel, TextEdit, TextureId as eguiTextureId, TopBottomPanel, UIWidget,
    UIWidgetRc, Ui, Widget, Window,
};

#[allow(dead_code)]
struct ContentBrowserData {
    shared_data: SharedDataRw,
    global_dispatcher: MessageBox,
    title: String,
    folder: PathBuf,
    selected_folder: PathBuf,
    selected_file: String,
    is_editable: bool,
    operation: DialogOp,
    icon_file_texture_id: TextureId,
}
implement_widget_data!(ContentBrowserData);

pub struct ContentBrowser {
    ui_page: UIWidgetRc,
    file_icon: TextureRc,
}

impl ContentBrowser {
    pub fn new(
        shared_data: &SharedDataRw,
        global_messenger: &MessengerRw,
        operation: DialogOp,
        path: &Path,
    ) -> Self {
        let file_icon = TextureInstance::create_from_file(
            shared_data,
            convert_from_local_path(
                PathBuf::from(DATA_FOLDER).as_path(),
                PathBuf::from("./icons/file.png").as_path(),
            )
            .as_path(),
        );
        let mut selected_folder = PathBuf::from(DATA_FOLDER);
        let mut selected_file = String::new();
        if path.to_path_buf().is_file() {
            if let Some(folder) = path.parent() {
                selected_folder = folder.to_path_buf();
            }
            if let Some(filename) = path.file_name() {
                selected_file = filename.to_str().unwrap().to_string();
            }
        }
        let data = ContentBrowserData {
            shared_data: shared_data.clone(),
            title: match operation {
                DialogOp::Open => "Open".to_string(),
                DialogOp::Save => "Save".to_string(),
                DialogOp::New => "New".to_string(),
            },
            folder: selected_folder.clone(),
            selected_folder,
            selected_file,
            is_editable: !matches!(operation, DialogOp::Open),
            operation,
            global_dispatcher: global_messenger.read().unwrap().get_dispatcher().clone(),
            icon_file_texture_id: file_icon.id(),
        };
        let ui_page = Self::create(shared_data, data);
        Self { ui_page, file_icon }
    }

    fn populate_with_folders_tree(ui: &mut Ui, root: &Path, data: &mut ContentBrowserData) {
        for_each_folder_in(root, |path| {
            let selected = data.selected_folder == path.to_path_buf();
            if is_folder_empty(path) {
                if ui
                    .selectable_label(selected, path.file_stem().unwrap().to_str().unwrap())
                    .clicked()
                {
                    data.selected_folder = path.to_path_buf();
                    data.selected_file = String::new();
                }
            } else {
                let collapsing = CollapsingHeader::new(path.file_stem().unwrap().to_str().unwrap())
                    .selectable(true)
                    .selected(selected);
                let header_response = collapsing
                    .show(ui, |ui| {
                        Self::populate_with_folders_tree(ui, path, data);
                    })
                    .header_response;
                if header_response.clicked() {
                    data.selected_folder = path.to_path_buf();
                    data.selected_file = String::new();
                }
            }
        });
    }

    fn populate_with_files(
        ui: &mut Ui,
        root: &Path,
        data: &mut ContentBrowserData,
        icon_file_texture_id: nrg_graphics::TextureId,
    ) {
        for_each_file_in(root, |path| {
            let filename = path.file_name().unwrap().to_str().unwrap().to_string();
            let selected = data.selected_file == filename;
            ui.horizontal(|ui| {
                let textures =
                    SharedData::get_resources_of_type::<TextureInstance>(&data.shared_data);
                if let Some(index) = textures.iter().position(|t| t.id() == icon_file_texture_id) {
                    ui.image(eguiTextureId::User(index as _), [16., 16.]);
                }
                if ui.selectable_label(selected, filename.clone()).clicked() {
                    data.selected_file = filename;
                }
            });
        });
    }

    fn create(shared_data: &SharedDataRw, data: ContentBrowserData) -> UIWidgetRc {
        let left_panel_min_width = 100.;
        let left_panel_max_width = left_panel_min_width * 4.;
        let bottom_panel_height = 25.;
        let button_size = 50.;
        UIWidget::register(shared_data, data, move |ui_data, ui_context| {
            if let Some(data) = ui_data.as_any().downcast_mut::<ContentBrowserData>() {
                let mut open = true;
                let mut rect = ui_context.available_rect();
                rect.min.x += rect.size().x * 0.05;
                rect.min.y += rect.size().y * 0.05;
                rect.max.x -= rect.size().x * 0.05;
                rect.max.y -= rect.size().y * 0.1;
                Window::new(data.title.clone())
                    .scroll(false)
                    .title_bar(true)
                    .collapsible(false)
                    .resizable(true)
                    .open(&mut open)
                    .default_rect(rect)
                    .show(ui_context, |ui| {
                        ui.expand_to_include_rect(ui.max_rect()); // Expand frame to include it all

                        let mut top_rect = ui.available_rect_before_wrap_finite();
                        top_rect.min.y += ui.spacing().item_spacing.y;
                        let mut top_ui = ui.child_ui(top_rect, Layout::top_down(Align::Max));

                        let top_response = TopBottomPanel::top("window_menu")
                            .resizable(false)
                            .show_inside(&mut top_ui, |ui| {
                                menu::bar(ui, |ui| {
                                    menu::menu(ui, "File", |ui| {
                                        if ui.button("Exit").clicked() {
                                            data.global_dispatcher
                                                .write()
                                                .unwrap()
                                                .send(
                                                    DialogEvent::Canceled(data.operation)
                                                        .as_boxed(),
                                                )
                                                .ok();
                                        }
                                    });
                                });
                            });

                        let mut left_rect = ui.available_rect_before_wrap_finite();
                        left_rect.min.y =
                            top_response.response.rect.max.y + ui.spacing().item_spacing.y;
                        let mut left_ui = ui.child_ui(left_rect, Layout::top_down(Align::Max));

                        let left_response = SidePanel::left("Folders")
                            .resizable(true)
                            .min_width(left_panel_min_width)
                            .max_width(left_panel_max_width)
                            .show_inside(&mut left_ui, |ui| {
                                ScrollArea::auto_sized().show(ui, |ui| {
                                    let path = data.folder.as_path().to_path_buf();
                                    Self::populate_with_folders_tree(ui, path.as_path(), data);
                                })
                            });

                        let mut right_rect = ui.available_rect_before_wrap_finite();
                        right_rect.min.x = left_response.response.rect.max.x;
                        right_rect.min.y =
                            top_response.response.rect.max.y + ui.spacing().item_spacing.y;
                        let mut right_ui = ui.child_ui(right_rect, Layout::top_down(Align::Max));

                        CentralPanel::default().show_inside(&mut right_ui, |ui| {
                            let mut rect = ui.min_rect();
                            let mut bottom_rect = rect;
                            bottom_rect.min.y = ui.max_rect_finite().max.y - bottom_panel_height;
                            rect.max.y = bottom_rect.min.y - ui.spacing().indent;
                            let mut child_ui = ui.child_ui(rect, Layout::top_down(Align::Min));
                            let mut bottom_ui =
                                ui.child_ui(bottom_rect, Layout::bottom_up(Align::Max));
                            ScrollArea::auto_sized().show(&mut child_ui, |ui| {
                                if data.selected_folder.is_dir() {
                                    let path = data.selected_folder.as_path().to_path_buf();
                                    Self::populate_with_files(
                                        ui,
                                        path.as_path(),
                                        data,
                                        data.icon_file_texture_id,
                                    );
                                }
                            });
                            bottom_ui.vertical(|ui| {
                                ui.separator();
                                ui.horizontal(|ui| {
                                    ui.label("Filename: ");
                                    TextEdit::singleline(&mut data.selected_file)
                                        .hint_text("File name here")
                                        .enabled(data.is_editable)
                                        .frame(data.is_editable)
                                        .desired_width(ui.available_width() - 2. * button_size)
                                        .ui(ui);
                                    if ui.button("Ok").clicked() {
                                        let path = data.selected_folder.clone();
                                        let path = path.join(data.selected_file.clone());
                                        data.global_dispatcher
                                            .write()
                                            .unwrap()
                                            .send(
                                                DialogEvent::Confirmed(data.operation, path)
                                                    .as_boxed(),
                                            )
                                            .ok();
                                    }
                                    if ui.button("Cancel").clicked() {
                                        data.global_dispatcher
                                            .write()
                                            .unwrap()
                                            .send(DialogEvent::Canceled(data.operation).as_boxed())
                                            .ok();
                                    }
                                });
                            });
                        });
                    });

                if !open {
                    data.global_dispatcher
                        .write()
                        .unwrap()
                        .send(DialogEvent::Canceled(data.operation).as_boxed())
                        .ok();
                }
            }
        })
    }

    fn process_command_result(command: &mut Command, dispatcher: MessageBox) {
        let result = command.output();
        match result {
            Ok(output) => {
                let string = String::from_utf8(output.stdout).unwrap();
                println!("{}", string);
                for e in get_events_from_string(string) {
                    let event: DialogEvent = deserialize(e);
                    dispatcher.write().unwrap().send(event.as_boxed()).ok();
                }
            }
            Err(_) => {
                println!("Failed to execute process");
            }
        }
    }
}
