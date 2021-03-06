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
            .position([300.0, 300.0].into())
            .size([800.0, 600.0].into())
            .color(0., 0., 1., 1.);
    }
    fn run(&mut self) -> bool {
        self.screen.update();
        self.update_mouse_pos();

        {
            let read_data = self.shared_data.read().unwrap();
            let renderer = &mut *read_data.get_unique_resource_mut::<Renderer>();

            self.widget.update(None, renderer, &self.input_handler);
        }

        let mut line = 0.0;
        let mouse_pos = Vector2f {
            x: self.input_handler.get_mouse_data().get_x() as _,
            y: self.input_handler.get_mouse_data().get_y() as _,
        };
        self.write_line(
            format!("Mouse Input [{}, {}]", mouse_pos.x, mouse_pos.y,),
            &mut line,
        );
        let pos: Vector2f = self.screen.convert_into_pixels(mouse_pos);
        self.write_line(format!("Mouse Pixels[{}, {}]", pos.x, pos.y), &mut line);
        let pos: Vector2f = self.screen.convert_into_screen_space(mouse_pos);
        self.write_line(
            format!("Mouse ScreenSpace[{}, {}]", pos.x, pos.y),
            &mut line,
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

        let pipeline_id = renderer.get_pipeline_id("Font");
        renderer.add_font(pipeline_id, self.config.fonts.first().unwrap());
    }

    fn write_line(&self, string: String, line: &mut f32) {
        let read_data = self.shared_data.read().unwrap();
        let renderer = &mut *read_data.get_unique_resource_mut::<Renderer>();

        renderer.add_text(
            renderer.get_default_font_id(),
            string.as_str(),
            [-0.9, 0.85 + *line].into(),
            1.,
            [0., 0.8, 1.0, 1.].into(),
        );
        *line += 0.05;
    }

    fn update_mouse_pos(&mut self) {
        let read_data = self.shared_data.read().unwrap();
        let window = &*read_data.get_unique_resource::<Window>();

        let window_events = window.get_events();
        self.input_handler.update(&window_events);
    }
}
