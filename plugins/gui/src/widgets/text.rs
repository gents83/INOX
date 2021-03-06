use super::*;
use nrg_graphics::*;
use nrg_platform::*;

pub struct Text {
    font_id: FontId,
    text: String,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            font_id: INVALID_ID,
            text: String::new(),
        }
    }
}

impl Text {
    pub fn set_text(&mut self, text: &str) -> &mut Self {
        self.text = String::from(text);
        self
    }
}

impl WidgetTrait for Text {
    fn init(widget: &mut Widget<Self>, renderer: &mut Renderer) {
        widget.get_mut().font_id = renderer.get_default_font_id();

        let data = widget.get_data_mut();
        data.graphics.init(renderer, "Font");

        data.state.pos = [10.0, 10.0].into();
        data.state.set_size([0.0, 50.0 * DEFAULT_DPI].into());
        data.graphics.set_style(WidgetStyle::default_text());

        widget.update_layout();
    }

    fn update(
        widget: &mut Widget<Self>,
        _parent_data: Option<&WidgetState>,
        renderer: &mut Renderer,
        _input_handler: &InputHandler,
    ) {
        let screen = widget.get_screen();
        let pos = screen.convert_from_pixels_into_screen_space(widget.get_data_mut().state.pos);
        let scale = (widget.get_data_mut().state.get_size() / screen.get_size()).y;
        let color = widget.get_data_mut().graphics.get_color();
        renderer.add_text(
            widget.get_mut().font_id,
            widget.get_mut().text.as_str(),
            pos,
            scale,
            color,
        );
    }

    fn uninit(_widget: &mut Widget<Self>, _renderer: &mut Renderer) {}

    fn get_type(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}
