use crate::widgets::*;

use super::config::*;

use nrg_core::*;
use nrg_graphics::*;
use nrg_math::*;
use nrg_platform::*;

pub struct GuiUpdater {
    id: SystemId,
    shared_data: SharedDataRw,
    config: Config,
    panel: Panel,
    mouse_pos: Vector2f,
}

impl GuiUpdater {
    pub fn new(shared_data: &SharedDataRw, config: &Config) -> Self {
        Self {
            id: SystemId::new(),
            shared_data: shared_data.clone(),
            config: config.clone(),
            mouse_pos: Vector2f::default(),
            panel: Panel::default(),
        }
    }
}

impl System for GuiUpdater {
    fn id(&self) -> SystemId {
        self.id
    }

    fn init(&mut self) {
        self.load_pipelines();

        let read_data = self.shared_data.read().unwrap();
        let renderer = &mut *read_data.get_unique_resource_mut::<Renderer>();

        self.panel
            .init(renderer)
            .set_position(0.25, 0.25)
            .set_size(0.5, 0.5)
            .set_color(1.0, 0.0, 0.0);
    }
    fn run(&mut self) -> bool {
        self.update_mouse_pos();

        if self.panel.is_inside(self.mouse_pos.x, self.mouse_pos.y) {
            self.panel.set_color(0.0, 1.0, 0.0);
        } else {
            self.panel.set_color(1.0, 0.0, 0.0);
        }

        let mut line = 0.0;
        line = self.write_line(
            format!("Mouse [{}, {}]", self.mouse_pos.x, self.mouse_pos.y),
            line,
        );
        let pos: Vector2f = Vector2f {
            x: self.mouse_pos.x,
            y: self.mouse_pos.y,
        } * 2.0
            - [1.0, 1.0].into();
        line = self.write_line(format!("Screen mouse [{}, {}]", pos.x, pos.y), line);
        line = self.write_line(
            format!(
                "Panel [{}, {}]",
                self.panel.get_position().x,
                self.panel.get_position().y
            ),
            line,
        );

        let mut i = 0;
        let indices_count = self.panel.get_widget().mesh_data.indices.len();
        while i < indices_count {
            line = self.write_line(
                format!(
                    "Triangle [{},{}] [{},{}] [{},{}]",
                    self.panel.get_widget().mesh_data.vertices
                        [self.panel.get_widget().mesh_data.indices[i] as usize]
                        .pos
                        .x,
                    self.panel.get_widget().mesh_data.vertices
                        [self.panel.get_widget().mesh_data.indices[i] as usize]
                        .pos
                        .y,
                    self.panel.get_widget().mesh_data.vertices
                        [self.panel.get_widget().mesh_data.indices[i + 1] as usize]
                        .pos
                        .x,
                    self.panel.get_widget().mesh_data.vertices
                        [self.panel.get_widget().mesh_data.indices[i + 1] as usize]
                        .pos
                        .y,
                    self.panel.get_widget().mesh_data.vertices
                        [self.panel.get_widget().mesh_data.indices[i + 2] as usize]
                        .pos
                        .x,
                    self.panel.get_widget().mesh_data.vertices
                        [self.panel.get_widget().mesh_data.indices[i + 2] as usize]
                        .pos
                        .y,
                ),
                line,
            );
            i += 3;
        }

        {
            let read_data = self.shared_data.read().unwrap();
            let renderer = &mut *read_data.get_unique_resource_mut::<Renderer>();
            self.panel.update(renderer);
        }

        true
    }
    fn uninit(&mut self) {
        let read_data = self.shared_data.read().unwrap();
        let renderer = &mut *read_data.get_unique_resource_mut::<Renderer>();

        self.panel.uninit(renderer);
    }
}

impl GuiUpdater {
    fn load_pipelines(&mut self) {
        let read_data = self.shared_data.read().unwrap();
        let renderer = &mut *read_data.get_unique_resource_mut::<Renderer>();

        for pipeline_data in self.config.pipelines.iter() {
            renderer.add_pipeline(pipeline_data);
        }
    }

    fn write_line(&self, string: String, mut line: f32) -> f32 {
        let read_data = self.shared_data.read().unwrap();
        let renderer = &mut *read_data.get_unique_resource_mut::<Renderer>();

        let pipeline_id = renderer.get_pipeline_id("Font");
        let font_id = renderer.add_font(pipeline_id, self.config.fonts.first().unwrap());

        renderer.add_text(
            font_id,
            string.as_str(),
            [-0.9, 0.65 + line].into(),
            1.0,
            [0.0, 0.8, 1.0].into(),
        );
        line += 0.05;
        line
    }

    fn update_mouse_pos(&mut self) {
        let read_data = self.shared_data.read().unwrap();
        let window = &*read_data.get_unique_resource::<Window>();

        let window_events = window.get_events();
        let events = window_events.read().unwrap();
        let mouse_events = events.read_events::<MouseEvent>();
        if let Some(&event) = mouse_events.last() {
            self.mouse_pos = Vector2f {
                x: event.x as f32 / window.get_width() as f32,
                y: event.y as f32 / window.get_heigth() as f32,
            };
        }
    }
}
