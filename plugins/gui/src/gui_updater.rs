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
    screen: Screen,
    widget: Widget<Container>,
    input_handler: InputHandler,
}

impl GuiUpdater {
    pub fn new(shared_data: &SharedDataRw, config: &Config) -> Self {
        let screen = Screen::default();
        Self {
            id: SystemId::new(),
            shared_data: shared_data.clone(),
            config: config.clone(),
            input_handler: InputHandler::default(),
            widget: Widget::<Container>::new(Container::default(), screen.clone()),
            screen,
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
        let window = &*read_data.get_unique_resource::<Window>();

        self.input_handler
            .init(window.get_width() as _, window.get_heigth() as _);

        self.screen.init(window);
        self.widget
            .init(renderer)
            .position([200.0, 200.0].into())
            .size([500.0, 500.0].into())
            .color(0.0, 0.0, 1.0);
    }
    fn run(&mut self) -> bool {
        self.screen.update();
        self.update_mouse_pos();

        {
            let read_data = self.shared_data.read().unwrap();
            let renderer = &mut *read_data.get_unique_resource_mut::<Renderer>();

            self.widget.update(renderer, &self.input_handler);

            if self.widget.is_hover() {
                self.widget.set_color(0.0, 1.0, 0.0);
            } else {
                self.widget.set_color(0.0, 1.0, 1.0);
            }
            /*
            if let Some(subpanel) = self.panel.get_child_mut(self.subpanel_id) {
                if subpanel.is_hover() {
                    subpanel.set_color(0.0, 1.0, 0.0);
                } else {
                    subpanel.set_color(0.0, 0.0, 1.0);
                }
            }*/
        }

        let mut line = 0.0;
        line = self.write_line(
            format!(
                "Mouse [{}, {}]",
                self.input_handler.get_mouse_data().get_x(),
                self.input_handler.get_mouse_data().get_y()
            ),
            line,
        );
        let pos: Vector2f = Vector2f {
            x: self.input_handler.get_mouse_data().get_x() as _,
            y: self.input_handler.get_mouse_data().get_y() as _,
        } * 2.0
            - [1.0, 1.0].into();
        line = self.write_line(format!("Screen mouse [{}, {}]", pos.x, pos.y), line);

        self.write_line(
            format!(
                "Widget in pixels Pos:[{}, {}] - Size:[{}, {}]",
                self.widget.get_position().x,
                self.widget.get_position().y,
                self.widget.get_size().x,
                self.widget.get_size().y
            ),
            line,
        );
        true
    }
    fn uninit(&mut self) {
        let read_data = self.shared_data.read().unwrap();
        let renderer = &mut *read_data.get_unique_resource_mut::<Renderer>();

        self.widget.uninit(renderer);
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
        self.input_handler.update(&window_events);
    }
}
