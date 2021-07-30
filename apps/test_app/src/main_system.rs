use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use nrg_core::{System, SystemId};
use nrg_graphics::{
    FontInstance, FontRc, MaterialInstance, MeshInstance, PipelineInstance, PipelineRc,
    RenderPassInstance, RenderPassRc, TextureInstance,
};
use nrg_messenger::{Message, MessengerRw};
use nrg_platform::WindowEvent;
use nrg_resources::{
    ConfigBase, DataTypeResource, FileResource, ResourceData, SharedData, SharedDataRw,
};
use nrg_scene::{Object, Scene, Transform};
use nrg_serialize::deserialize_from_file;
use nrg_ui::{implement_widget_data, UIWidget, UIWidgetRc, Ui, Window};

use crate::config::Config;

pub struct MainSystem {
    id: SystemId,
    config: Config,
    shared_data: SharedDataRw,
    global_messenger: MessengerRw,
    pipelines: Vec<PipelineRc>,
    render_passes: Vec<RenderPassRc>,
    fonts: Vec<FontRc>,
    ui_page: UIWidgetRc,
}

impl MainSystem {
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

    fn resource_ui<R>(shared_data: &SharedDataRw, ui: &mut Ui, title: &str)
    where
        R: ResourceData,
    {
        ui.collapsing(
            format!(
                "{}: {}",
                title,
                SharedData::get_num_resources_of_type::<R>(shared_data)
            ),
            |ui| {
                let resources = SharedData::get_resources_of_type::<R>(shared_data);
                for r in resources.iter() {
                    ui.label(r.id().to_string());
                }
            },
        );
    }

    fn create_ui(&mut self) -> &mut Self {
        struct FPSData {
            frame_seconds: VecDeque<Instant>,
            shared_data: SharedDataRw,
        }
        implement_widget_data!(FPSData);
        let data = FPSData {
            frame_seconds: VecDeque::default(),
            shared_data: self.shared_data.clone(),
        };
        self.ui_page = UIWidget::register(&self.shared_data, data, |ui_data, ui_context| {
            if let Some(fps_data) = ui_data.as_any().downcast_mut::<FPSData>() {
                let now = Instant::now();
                let one_sec_before = now - Duration::from_secs(1);
                fps_data.frame_seconds.push_back(now);
                fps_data.frame_seconds.retain(|t| *t >= one_sec_before);

                Window::new("Stats")
                    .scroll(true)
                    .title_bar(true)
                    .resizable(true)
                    .show(ui_context, |ui| {
                        ui.label(format!("FPS: {}", fps_data.frame_seconds.len()));
                        ui.separator();
                        Self::resource_ui::<PipelineInstance>(
                            &fps_data.shared_data,
                            ui,
                            "Pipeline",
                        );
                        Self::resource_ui::<FontInstance>(&fps_data.shared_data, ui, "Font");
                        Self::resource_ui::<MaterialInstance>(
                            &fps_data.shared_data,
                            ui,
                            "Material",
                        );
                        Self::resource_ui::<TextureInstance>(&fps_data.shared_data, ui, "Texture");
                        Self::resource_ui::<MeshInstance>(&fps_data.shared_data, ui, "Mesh");
                        ui.separator();
                        Self::resource_ui::<Scene>(&fps_data.shared_data, ui, "Scene");
                        Self::resource_ui::<Object>(&fps_data.shared_data, ui, "Object");
                        Self::resource_ui::<Transform>(&fps_data.shared_data, ui, "Transform");
                    });
            }
        });
        self
    }
}

impl System for MainSystem {
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

        self.create_ui();
    }

    fn run(&mut self) -> bool {
        true
    }

    fn uninit(&mut self) {}
}
