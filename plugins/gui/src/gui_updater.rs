use crate::widgets::*;

use super::config::*;

use nrg_core::*;
use nrg_graphics::*;
use nrg_math::*;
use nrg_platform::*;
use nrg_serialize::*;

pub struct GuiUpdater {
    id: SystemId,
    shared_data: SharedDataRw,
    config: Config,
    screen: Screen,
    widget: Widget<Panel>,
    input_handler: InputHandler,
    fps_text_widget_id: UID,
    button_widget_id: UID,
    button_text_id: UID,
    time_per_fps: f64,
}

impl GuiUpdater {
    pub fn new(shared_data: &SharedDataRw, config: &Config) -> Self {
        let screen = Screen::default();
        Self {
            id: SystemId::new(),
            shared_data: shared_data.clone(),
            config: config.clone(),
            input_handler: InputHandler::default(),
            widget: Widget::<Panel>::new(Panel::default(), screen.clone()),
            screen,
            fps_text_widget_id: INVALID_ID,
            button_widget_id: INVALID_ID,
            button_text_id: INVALID_ID,
            time_per_fps: 0.,
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

        let events = &mut *read_data.get_unique_resource_mut::<EventsRw>();
        GUI::register_widget_events(events);

        self.input_handler
            .init(window.get_width() as _, window.get_heigth() as _);

        self.screen.init(window);
        self.widget
            .init(renderer)
            .position([300.0, 300.0].into())
            .size([800.0, 600.0].into())
            .draggable(true)
            .vertical_alignment(VerticalAlignment::Top)
            .horizontal_alignment(HorizontalAlignment::Right)
            .get_mut()
            .set_fill_type(ContainerFillType::Vertical)
            .set_fit_to_content(true)
            .set_space_between_elements(20.);

        let mut fps_text = Widget::<Text>::new(Text::default(), self.screen.clone());
        fps_text
            .init(renderer)
            .size([400.0, 20.0].into())
            .vertical_alignment(VerticalAlignment::Top)
            .horizontal_alignment(HorizontalAlignment::Left)
            .get_mut()
            .set_text("FPS: ");

        let mut button = Widget::<Button>::new(Button::default(), self.screen.clone());
        button
            .init(renderer)
            .size([550., 150.].into())
            .stroke(10.)
            .get_mut()
            .set_fill_type(ContainerFillType::Horizontal)
            .set_fit_to_content(false);

        let mut text = Widget::<Text>::new(Text::default(), self.screen.clone());
        text.init(renderer)
            .size([400.0, 50.0].into())
            .vertical_alignment(VerticalAlignment::Center)
            .horizontal_alignment(HorizontalAlignment::Center)
            .get_mut()
            .set_text("Click on me");

        self.fps_text_widget_id = self.widget.add_child(fps_text);
        self.button_text_id = button.add_child(text);
        self.button_widget_id = self.widget.add_child(button);
    }

    fn run(&mut self) -> bool {
        let time = std::time::Instant::now();

        self.screen.update();
        self.update_mouse_pos();

        {
            let read_data = self.shared_data.read().unwrap();
            let events = &mut *read_data.get_unique_resource_mut::<EventsRw>();
            let renderer = &mut *read_data.get_unique_resource_mut::<Renderer>();

            self.widget
                .update(None, renderer, events, &self.input_handler);
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
        let pos: Vector2f = self.screen.convert_position_into_pixels(mouse_pos);
        self.write_line(format!("Mouse Pixels[{}, {}]", pos.x, pos.y), &mut line);
        let pos: Vector2f = self.screen.convert_into_screen_space(mouse_pos);
        self.write_line(
            format!("Mouse ScreenSpace[{}, {}]", pos.x, pos.y),
            &mut line,
        );

        self.time_per_fps = time.elapsed().as_secs_f64();
        if let Some(widget) = self.widget.get_child::<Text>(self.fps_text_widget_id) {
            let str = format!("FPS: {:.3}", (60. * self.time_per_fps / 0.001) as u32);
            let fps_text = widget.get_mut();
            fps_text.set_text(str.as_str());
        }

        {
            let read_data = self.shared_data.read().unwrap();
            let events = &mut *read_data.get_unique_resource_mut::<EventsRw>();
            let events = events.read().unwrap();

            let button = self
                .widget
                .get_child::<Button>(self.button_widget_id)
                .unwrap();
            let title = button.get_child::<Text>(self.button_text_id).unwrap();

            for event in events.read_events::<WidgetEvent>() {
                match event {
                    WidgetEvent::Entering(widget_id) => {
                        if *widget_id == self.button_widget_id {
                            title.get_mut().set_text("Entered!!!");
                        }
                    }
                    WidgetEvent::Exiting(widget_id) => {
                        if *widget_id == self.button_widget_id {
                            title.get_mut().set_text("Exited!!!");
                        }
                    }
                    WidgetEvent::Released(widget_id) => {
                        if *widget_id == self.button_widget_id {
                            title.get_mut().set_text("Released!!!");
                        }
                    }
                    WidgetEvent::Pressed(widget_id) => {
                        if *widget_id == self.button_widget_id {
                            title.get_mut().set_text("Pressed!!!");
                        }
                    }
                }
            }
        }

        true
    }
    fn uninit(&mut self) {
        let read_data = self.shared_data.read().unwrap();
        let renderer = &mut *read_data.get_unique_resource_mut::<Renderer>();

        self.widget.uninit(renderer);

        let events = &mut *read_data.get_unique_resource_mut::<EventsRw>();
        GUI::unregister_widget_events(events);
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
        let window = &*read_data.get_unique_resource::<Window>();

        renderer.add_text(
            renderer.get_default_font_id(),
            string.as_str(),
            [-0.9, 0.8 + *line].into(),
            25. * window.get_scale_factor(),
            [0., 0.8, 1., 1.].into(),
            Vector2f { x: 0., y: 0. } * window.get_scale_factor(),
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
