use std::{
    path::{Path, PathBuf},
    process::Command,
};

use nrg_graphics::{Texture, TextureId};
use nrg_messenger::{get_events_from_string, Message, MessageBox, MessengerRw};

use nrg_resources::{Resource, SerializableResource, SharedData, SharedDataRc, DATA_FOLDER};
use nrg_serialize::deserialize;
use nrg_ui::{
    implement_widget_data, CentralPanel, CollapsingHeader, DialogEvent, DialogOp, ScrollArea,
    SidePanel, TextEdit, TextureId as eguiTextureId, TopBottomPanel, UIWidget, Ui, Widget, Window,
};

struct File {
    path: PathBuf,
}
struct Dir {
    path: PathBuf,
    subdirs: Vec<Dir>,
    files: Vec<File>,
}

#[allow(dead_code)]
struct ContentBrowserData {
    shared_data: SharedDataRc,
    global_dispatcher: MessageBox,
    title: String,
    folder: PathBuf,
    selected_folder: PathBuf,
    selected_file: String,
    is_editable: bool,
    operation: DialogOp,
    icon_file_texture_id: TextureId,
    dir: Dir,
    extension: String,
}
implement_widget_data!(ContentBrowserData);

pub struct ContentBrowser {
    ui_page: Resource<UIWidget>,
    file_icon: Resource<Texture>,
}

impl ContentBrowser {
    pub fn new(
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        operation: DialogOp,
        path: &Path,
        extension: String,
    ) -> Self {
        let file_icon = Texture::load_from_file(
            shared_data,
            global_messenger,
            PathBuf::from("./icons/file.png").as_path(),
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
        let mut dir = Dir {
            path: selected_folder.clone(),
            subdirs: Vec::new(),
            files: Vec::new(),
        };
        Self::fill_dir(&mut dir, selected_folder.as_path());

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
            icon_file_texture_id: *file_icon.id(),
            dir,
            extension,
        };
        let ui_page = Self::create(shared_data, data);
        Self { ui_page, file_icon }
    }

    fn fill_dir(dir: &mut Dir, root: &Path) {
        if let Ok(directory) = std::fs::read_dir(root) {
            directory.for_each(|entry| {
                if let Ok(dir_entry) = entry {
                    let path = dir_entry.path();
                    if path.is_file() {
                        dir.files.push(File { path });
                    } else if path.is_dir() {
                        let mut subdir = Dir {
                            path: dir_entry.path(),
                            subdirs: Vec::new(),
                            files: Vec::new(),
                        };
                        Self::fill_dir(&mut subdir, path.as_path());
                        dir.subdirs.push(subdir);
                    }
                }
            });
        }
    }
    fn get_files<'a>(dir: &'a Dir, path: &Path) -> &'a Vec<File> {
        if dir.path.as_path() != path {
            for d in dir.subdirs.iter() {
                if dir.path.as_path() == path {
                    return &d.files;
                } else if path.starts_with(&d.path) {
                    return Self::get_files(d, path);
                }
            }
        }
        &dir.files
    }

    fn populate_with_folders_tree(
        ui: &mut Ui,
        directory: &Dir,
        selected_folder: &mut PathBuf,
        selected_file: &mut String,
    ) {
        nrg_profiler::scoped_profile!("populate_with_folders_tree");
        let selected = selected_folder == &directory.path;
        if directory.subdirs.is_empty() {
            if ui
                .selectable_label(
                    selected,
                    directory.path.file_stem().unwrap().to_str().unwrap(),
                )
                .clicked()
            {
                *selected_folder = directory.path.to_path_buf();
                *selected_file = String::new();
            }
        } else {
            let collapsing =
                CollapsingHeader::new(directory.path.file_stem().unwrap().to_str().unwrap())
                    .selectable(true)
                    .default_open(selected)
                    .selected(selected);
            let header_response = collapsing
                .show(ui, |ui| {
                    for subdir in directory.subdirs.iter() {
                        Self::populate_with_folders_tree(
                            ui,
                            subdir,
                            selected_folder,
                            selected_file,
                        );
                    }
                })
                .header_response;
            if header_response.clicked() {
                *selected_folder = directory.path.to_path_buf();
                *selected_file = String::new();
            }
        }
    }

    fn populate_with_files(
        ui: &mut Ui,
        files: &[File],
        selected_file: &mut String,
        selected_extension: &str,
        texture_index: u64,
    ) {
        nrg_profiler::scoped_profile!("populate_with_files");
        ui.vertical(|ui| {
            for file in files.iter() {
                let filename = file.path.file_name().unwrap().to_str().unwrap().to_string();
                let extension = file.path.extension().unwrap().to_str().unwrap().to_string();
                if extension == selected_extension {
                    let selected = selected_file == &filename;
                    ui.horizontal(|ui| {
                        ui.image(eguiTextureId::User(texture_index as _), [16., 16.]);
                        if ui.selectable_label(selected, filename.clone()).clicked() {
                            *selected_file = filename;
                        }
                    });
                }
            }
        });
    }

    fn create(shared_data: &SharedDataRc, data: ContentBrowserData) -> Resource<UIWidget> {
        let left_panel_min_width = 100.;
        let left_panel_max_width = left_panel_min_width * 4.;
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
                    .vscroll(false)
                    .title_bar(true)
                    .collapsible(false)
                    .resizable(true)
                    .open(&mut open)
                    .default_rect(rect)
                    .show(ui_context, |ui| {
                        nrg_profiler::scoped_profile!("Window");
                        SidePanel::left("Folders")
                            .resizable(true)
                            .width_range(left_panel_min_width..=left_panel_max_width)
                            .show_inside(ui, |ui| {
                                nrg_profiler::scoped_profile!("SidePanel");
                                ScrollArea::vertical().show(ui, |ui| {
                                    Self::populate_with_folders_tree(
                                        ui,
                                        &data.dir,
                                        &mut data.selected_folder,
                                        &mut data.selected_file,
                                    );
                                })
                            });

                        TopBottomPanel::bottom("bottom_panel")
                            .resizable(false)
                            .min_height(0.0)
                            .show_inside(ui, |ui| {
                                nrg_profiler::scoped_profile!("BottomPanel");
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

                        CentralPanel::default().show_inside(ui, |ui| {
                            nrg_profiler::scoped_profile!("CentralPanel");
                            let rect = ui.max_rect();
                            ScrollArea::vertical()
                                .max_height(rect.height())
                                .show(ui, |ui| {
                                    if data.selected_folder.is_dir() {
                                        if let Some(texture_index) =
                                            SharedData::get_index_of_resource::<Texture>(
                                                &data.shared_data,
                                                &data.icon_file_texture_id,
                                            )
                                        {
                                            let path = data.selected_folder.as_path().to_path_buf();
                                            let files = Self::get_files(&data.dir, path.as_path());
                                            Self::populate_with_files(
                                                ui,
                                                files,
                                                &mut data.selected_file,
                                                data.extension.as_str(),
                                                texture_index as _,
                                            );
                                        }
                                    }
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
