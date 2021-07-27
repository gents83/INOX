use std::{
    env,
    path::{Path, PathBuf},
};

use nrg_core::{System, SystemId};
use nrg_filesystem::{for_each_file_in, for_each_folder_in, is_folder_empty};
use nrg_graphics::{
    FontInstance, FontRc, PipelineInstance, PipelineRc, RenderPassInstance, RenderPassRc,
};
use nrg_messenger::{Message, MessengerRw};
use nrg_platform::WindowEvent;
use nrg_resources::{ConfigBase, DataTypeResource, FileResource, SharedDataRw, DATA_FOLDER};
use nrg_serialize::{deserialize_from_file, Uid};
use nrg_ui::{
    implement_widget_data, CollapsingHeader, ScrollArea, SidePanel, UIWidget, UIWidgetRc, Ui,
};

use crate::config::Config;

#[allow(dead_code)]
struct FolderDialogData {
    shared_data: SharedDataRw,
    title: String,
    folder: PathBuf,
    selected_folder: PathBuf,
    selected_file: PathBuf,
    is_editable: bool,
    requester_uid: Option<Uid>,
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
    }

    fn send_event(&self, event: Box<dyn Message>) {
        self.global_messenger
            .read()
            .unwrap()
            .get_dispatcher()
            .write()
            .unwrap()
            .send(event)
            .ok();
    }

    fn window_init(&self) {
        self.send_event(WindowEvent::RequestChangeTitle(self.config.title.clone()).as_boxed());
        self.send_event(
            WindowEvent::RequestChangeSize(self.config.width, self.config.height).as_boxed(),
        );
        self.send_event(
            WindowEvent::RequestChangePos(self.config.pos_x, self.config.pos_y).as_boxed(),
        );
        self.send_event(WindowEvent::RequestChangeVisible(true).as_boxed());
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
            requester_uid: if args.len() > 3 {
                Uid::parse_str(args[3].as_str()).ok()
            } else {
                None
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
                PathBuf::from(args[6].as_str())
            } else {
                PathBuf::new()
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
                    data.selected_file = PathBuf::new();
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
                    data.selected_file = PathBuf::new();
                }
            }
        });
    }

    fn populate_with_files(ui: &mut Ui, root: &Path, data: &mut FolderDialogData) {
        for_each_file_in(root, |path| {
            let selected = data.selected_file == path.to_path_buf();
            if ui
                .selectable_label(selected, path.file_name().unwrap().to_str().unwrap())
                .clicked()
            {
                data.selected_file = path.to_path_buf();
            }
        });
    }

    fn add_content(&mut self) -> &mut Self {
        let data = self.create_data_from_args(env::args().collect());
        let left_panel_width = 200.;
        let right_panel_width = self.config.width as f32 - left_panel_width;
        self.ui_page = UIWidget::register(&self.shared_data, data, move |ui_data, ui_context| {
            if let Some(data) = ui_data.as_any().downcast_mut::<FolderDialogData>() {
                SidePanel::left("Folders")
                    .resizable(true)
                    .min_width(left_panel_width)
                    .show(ui_context, |ui| {
                        ScrollArea::auto_sized().show(ui, |ui| {
                            let path = data.folder.as_path().to_path_buf();
                            Self::populate_with_folders_tree(ui, path.as_path(), data);
                        })
                    });
                SidePanel::right("Files")
                    .resizable(false)
                    .min_width(right_panel_width)
                    .show(ui_context, |ui| {
                        ScrollArea::auto_sized().show(ui, |ui| {
                            if data.selected_folder.is_dir() {
                                let path = data.selected_folder.as_path().to_path_buf();
                                Self::populate_with_files(ui, path.as_path(), data);
                            }
                        })
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
