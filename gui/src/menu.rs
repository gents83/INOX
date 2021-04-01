use super::*;
use nrg_graphics::*;
use nrg_math::*;
use nrg_platform::*;
use nrg_serialize::*;

const DEFAULT_MENU_SIZE: Vector2u = Vector2u { x: 0, y: 20 };
const DEFAULT_MENU_ITEM_SIZE: Vector2u = Vector2u { x: 200, y: 100 };

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Menu {
    #[serde(skip)]
    container: ContainerData,
    data: WidgetData,
}
implement_widget!(Menu);
implement_container!(Menu);

impl Default for Menu {
    fn default() -> Self {
        Self {
            container: ContainerData::default(),
            data: WidgetData::default(),
        }
    }
}

impl Menu {
    pub fn add_menu_item(&mut self, renderer: &mut Renderer, label: &str) {
        let mut button = Button::default();
        button.init(renderer);
        button
            .size(DEFAULT_MENU_ITEM_SIZE * Screen::get_scale_factor())
            .set_text(label);
        button
            .get_data_mut()
            .graphics
            .set_style(WidgetStyle::DefaultBackground);

        self.add_child(Box::new(button));
    }
}

impl InternalWidget for Menu {
    fn widget_init(&mut self, renderer: &mut Renderer) {
        self.get_data_mut().graphics.init(renderer, "UI");
        if self.is_initialized() {
            return;
        }

        self.size(DEFAULT_MENU_SIZE * Screen::get_scale_factor())
            .selectable(false)
            .vertical_alignment(VerticalAlignment::Top)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .fill_type(ContainerFillType::Horizontal)
            .fit_to_content(false);

        let data = self.get_data_mut();
        data.graphics.set_style(WidgetStyle::DefaultBackground);
    }

    fn widget_update(
        &mut self,
        _drawing_area_in_px: Vector4u,
        _renderer: &mut Renderer,
        _events: &mut EventsRw,
        _input_handler: &InputHandler,
    ) {
        let data = self.get_data_mut();
        let pos = Screen::convert_from_pixels_into_screen_space(data.state.get_position());
        let size = Screen::convert_size_from_pixels(data.state.get_size());
        let mut mesh_data = MeshData::default();
        mesh_data
            .add_quad_default([0.0, 0.0, size.x, size.y].into(), data.state.get_layer())
            .set_vertex_color(data.graphics.get_color());
        mesh_data.translate([pos.x, pos.y, 0.0].into());
        data.graphics.set_mesh_data(mesh_data);
    }

    fn widget_uninit(&mut self, _renderer: &mut Renderer) {}
}
