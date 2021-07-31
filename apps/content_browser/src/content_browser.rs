use std::{
    env,
    path::{Path, PathBuf},
};

use nrg_core::{System, SystemId};
use nrg_filesystem::{
    convert_from_local_path, for_each_file_in, for_each_folder_in, is_folder_empty,
};
use nrg_graphics::{
    FontInstance, FontRc, PipelineInstance, PipelineRc, RenderPassInstance, RenderPassRc,
    TextureInstance, TextureRc,
};
use nrg_messenger::{Message, MessengerRw};
use nrg_platform::WindowEvent;
use nrg_resources::{
    ConfigBase, DataTypeResource, FileResource, SharedData, SharedDataRw, DATA_FOLDER,
};
use nrg_serialize::{deserialize_from_file, serialize};
use nrg_ui::{
    implement_widget_data, Align, CentralPanel, CollapsingHeader, DialogEvent, DialogOp, Layout,
    ScrollArea, SidePanel, TextEdit, TextureId as eguiTextureId, UIWidget, UIWidgetRc, Ui, Widget,
};

use crate::config::Config;

#[allow(dead_code)]
struct FolderDialogData {
    shared_data: SharedDataRw,
    title: String,
    folder: PathBuf,
    selected_folder: PathBuf,
    selected_file: String,
    is_editable: bool,
    operation: DialogOp,
}
implement_widget_data!(FolderDialogData);

pub struct ContentBrowserSystem {
    id: SystemId,
    config: Config,
    shared_data: SharedDataRw,
    global_messenger: MessengerRw,
    pipelines: Vec<PipelineRc>,
    render_passes: Vec<RenderPassRc>,
    fonts: Vec<FontRc>,
    ui_page: UIWidgetRc,
    file_icon: TextureRc,
}

impl ContentBrowserSystem {
    pub fn new(shared_data: SharedDataRw, global_messenger: MessengerRw) -> Self {
        Self {
            id: SystemId::new(),
            config: Config::default(),
            shared_data,
            global_messenger,
            pipelines: Vec::new(),
            render_passes: Vec::new(),
            fonts: Vec::new(),
            ui_page: UIWidgetRc::default(),
            file_icon: TextureRc::default(),
        }
    }

    fn load_pipelines(&mut self) {
        for render_pass_data in self.config.render_passes.iter() {
            self.render_passes
                .push(RenderPassInstance::create_from_data(
                    &self.shared_data,
                    render_pass_data.clone(),
                ));
        }

        for pipeline_data in self.config.pipelines.iter() {
            self.pipelines.push(PipelineInstance::create_from_data(
                &self.shared_data,
                pipeline_data.clone(),
            ));
        }

        if let Some(default_font_path) = self.config.fonts.first() {
            self.fonts.push(FontInstance::create_from_file(
                &self.shared_data,
                default_font_path,
            ));
        }

        self.file_icon = TextureInstance::create_from_file(
            &self.shared_data,
            convert_from_local_path(
                PathBuf::from(DATA_FOLDER).as_path(),
                PathBuf::from("./icons/file.png").as_path(),
            )
            .as_path(),
        );
    }

    fn send_event(global_messenger: &MessengerRw, event: Box<dyn Message>) {
        global_messenger
            .read()
            .unwrap()
            .get_dispatcher()
            .write()
            .unwrap()
            .send(event)
            .ok();
    }

    fn window_init(&self) {
        Self::send_event(
            &self.global_messenger,
            WindowEvent::RequestChangeTitle(self.config.title.clone()).as_boxed(),
        );
        Self::send_event(
            &self.global_messenger,
            WindowEvent::RequestChangeSize(self.config.width, self.config.height).as_boxed(),
        );
        Self::send_event(
            &self.global_messenger,
            WindowEvent::RequestChangePos(self.config.pos_x, self.config.pos_y).as_boxed(),
        );
        Self::send_event(
            &self.global_messenger,
            WindowEvent::RequestChangeVisible(true).as_boxed(),
        );
    }

    fn create_data_from_args(&self, args: Vec<String>) -> FolderDialogData {
        FolderDialogData {
            shared_data: self.shared_data.clone(),
            title: if args.len() > 1 {
                String::from(args[1].as_str())
            } else {
                String::from("Title")
            },
            folder: if args.len() > 2 {
                PathBuf::from(args[2].as_str())
            } else {
                PathBuf::from(DATA_FOLDER)
            },
            operation: if args.len() > 3 {
                DialogOp::from(args[3].as_str())
            } else {
                DialogOp::Open
            },
            is_editable: if args.len() > 4 {
                args[4] == "true"
            } else {
                false
            },
            selected_folder: if args.len() > 5 {
                PathBuf::from(args[5].as_str())
            } else {
                PathBuf::new()
            },
            selected_file: if args.len() > 6 {
                String::from(args[6].as_str())
            } else {
                String::new()
            },
        }
    }

    fn populate_with_folders_tree(ui: &mut Ui, root: &Path, data: &mut FolderDialogData) {
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
        data: &mut FolderDialogData,
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

    fn add_content(&mut self) -> &mut Self {
        let data = self.create_data_from_args(env::args().collect());
        let left_panel_min_width = 100.;
        let left_panel_max_width = left_panel_min_width * 4.;
        let bottom_panel_height = 25.;
        let button_size = 50.;
        let global_messenger = self.global_messenger.clone();
        let icon_file_texture_id = self.file_icon.id();
        self.ui_page = UIWidget::register(&self.shared_data, data, move |ui_data, ui_context| {
            if let Some(data) = ui_data.as_any().downcast_mut::<FolderDialogData>() {
                SidePanel::left("Folders")
                    .resizable(true)
                    .min_width(left_panel_min_width)
                    .max_width(left_panel_max_width)
                    .show(ui_context, |ui| {
                        ScrollArea::auto_sized().show(ui, |ui| {
                            let path = data.folder.as_path().to_path_buf();
                            Self::populate_with_folders_tree(ui, path.as_path(), data);
                        })
                    });
                CentralPanel::default().show(ui_context, |ui| {
                    let mut rect = ui.min_rect();
                    let mut bottom_rect = rect;
                    bottom_rect.min.y = ui.max_rect_finite().max.y - bottom_panel_height;
                    rect.max.y = bottom_rect.min.y - ui.spacing().indent;
                    let mut child_ui = ui.child_ui(rect, Layout::top_down(Align::Min));
                    let mut bottom_ui = ui.child_ui(bottom_rect, Layout::bottom_up(Align::Max));
                    ScrollArea::auto_sized().show(&mut child_ui, |ui| {
                        if data.selected_folder.is_dir() {
                            let path = data.selected_folder.as_path().to_path_buf();
                            Self::populate_with_files(
                                ui,
                                path.as_path(),
                                data,
                                icon_file_texture_id,
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
                                Self::send_event(&global_messenger, WindowEvent::Close.as_boxed());

                                let path = data.selected_folder.clone();
                                let path = path.join(data.selected_file.clone());
                                let event = DialogEvent::Confirmed(data.operation, path);
                                let serialized_event = serialize(&event);
                                println!("[[[{}]]]", serialized_event);
                            }
                            if ui.button("Cancel").clicked() {
                                Self::send_event(&global_messenger, WindowEvent::Close.as_boxed());
                                let event = DialogEvent::Canceled(data.operation);
                                let serialized_event = serialize(&event);
                                println!("[[[{}]]]", serialized_event);
                            }
                        });
                    });
                });
            }
        });
        self
    }
}

impl System for ContentBrowserSystem {
    fn id(&self) -> nrg_core::SystemId {
        self.id
    }

    fn should_run_when_not_focused(&self) -> bool {
        false
    }
    fn init(&mut self) {
        let path = self.config.get_filepath();
        deserialize_from_file(&mut self.config, path);

        self.window_init();
        self.load_pipelines();

        self.add_content();
    }

    fn run(&mut self) -> bool {
        true
    }

    fn uninit(&mut self) {}
}
