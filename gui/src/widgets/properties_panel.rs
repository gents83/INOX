use std::any::TypeId;

use nrg_math::{Vector2, Vector4};
use nrg_messenger::{implement_message, Message};
use nrg_serialize::{Deserialize, Serialize, Uid};

use crate::{
    implement_widget_with_data, InternalWidget, Panel, Text, TextBox, WidgetData,
    DEFAULT_WIDGET_HEIGHT, DEFAULT_WIDGET_WIDTH,
};

pub const DEFAULT_PROPERTIES_SIZE: [f32; 2] = [DEFAULT_WIDGET_WIDTH, DEFAULT_WIDGET_HEIGHT];

#[derive(Clone)]
pub enum PropertiesEvent {
    GetProperties(Uid),
    AddString(Uid, String, String, bool),
    AddVector2(Uid, String, Vector2, bool),
}
implement_message!(PropertiesEvent);

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct PropertiesPanel {
    data: WidgetData,
}
implement_widget_with_data!(PropertiesPanel);

impl PropertiesPanel {
    fn add_label(&mut self) -> &mut Self {
        let mut label = Text::new(&self.get_shared_data(), &self.get_global_messenger());
        label.editable(false).set_text("Properties:");
        self.add_child(Box::new(label));

        self
    }

    pub fn reset(&mut self) -> &mut Self {
        self.node_mut().remove_children();
        self.add_label();
        self
    }

    fn create_panel(&mut self, text: &str) -> (Panel, Panel) {
        let mut horizontal_panel =
            Panel::new(&self.get_shared_data(), &self.get_global_messenger());
        horizontal_panel
            .style(WidgetStyle::Default)
            .fill_type(ContainerFillType::Horizontal)
            .keep_fixed_height(false)
            .keep_fixed_width(false)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .vertical_alignment(VerticalAlignment::Top);

        let mut label = Text::new(&self.get_shared_data(), &self.get_global_messenger());
        label.editable(false).set_text(text);
        horizontal_panel.add_child(Box::new(label));

        let mut vertical_panel = Panel::new(&self.get_shared_data(), &self.get_global_messenger());
        vertical_panel
            .style(WidgetStyle::Default)
            .fill_type(ContainerFillType::Vertical)
            .keep_fixed_height(false)
            .keep_fixed_width(false)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .vertical_alignment(VerticalAlignment::Top);

        (horizontal_panel, vertical_panel)
    }

    pub fn add_string(&mut self, text: &str, string: &str, editable: bool) -> &mut Self {
        let (mut horizontal_panel, mut vertical_panel) = self.create_panel(text);

        let mut textbox_string =
            TextBox::new(&self.get_shared_data(), &self.get_global_messenger());
        textbox_string
            .editable(editable)
            .set_text(string)
            .horizontal_alignment(HorizontalAlignment::Left);
        vertical_panel.add_child(Box::new(textbox_string));

        horizontal_panel.add_child(Box::new(vertical_panel));
        self.add_child(Box::new(horizontal_panel));
        self
    }

    pub fn add_vec2(&mut self, text: &str, vec2: Vector2, editable: bool) -> &mut Self {
        let (mut horizontal_panel, mut vertical_panel) = self.create_panel(text);

        let mut textbox_x = TextBox::new(&self.get_shared_data(), &self.get_global_messenger());
        textbox_x
            .editable(editable)
            .with_label("X:")
            .set_text(format!("{}", vec2.x).as_str())
            .horizontal_alignment(HorizontalAlignment::Left);
        vertical_panel.add_child(Box::new(textbox_x));

        let mut textbox_y = TextBox::new(&self.get_shared_data(), &self.get_global_messenger());
        textbox_y
            .editable(editable)
            .with_label("Y:")
            .set_text(format!("{}", vec2.y).as_str())
            .horizontal_alignment(HorizontalAlignment::Left);
        vertical_panel.add_child(Box::new(textbox_y));

        horizontal_panel.add_child(Box::new(vertical_panel));
        self.add_child(Box::new(horizontal_panel));
        self
    }
}

impl InternalWidget for PropertiesPanel {
    fn widget_init(&mut self) {
        self.register_to_listen_event::<PropertiesEvent>();

        if self.is_initialized() {
            return;
        }
        let size: Vector2 = DEFAULT_PROPERTIES_SIZE.into();
        self.size(size * Screen::get_scale_factor())
            .selectable(false)
            .draggable(false)
            .fill_type(ContainerFillType::Vertical)
            .keep_fixed_height(false)
            .keep_fixed_width(false)
            .horizontal_alignment(HorizontalAlignment::Right)
            .vertical_alignment(VerticalAlignment::Bottom)
            .style(WidgetStyle::Default);

        self.reset();
    }

    fn widget_update(&mut self, _drawing_area_in_px: Vector4) {}

    fn widget_uninit(&mut self) {
        self.unregister_to_listen_event::<PropertiesEvent>();
    }
    fn widget_process_message(&mut self, msg: &dyn Message) {
        if msg.type_id() == TypeId::of::<PropertiesEvent>() {
            let e = msg.as_any().downcast_ref::<PropertiesEvent>().unwrap();
            match e {
                PropertiesEvent::AddString(_uid, label, text, editable) => {
                    self.add_string(label.as_str(), text.as_str(), *editable);
                }
                PropertiesEvent::AddVector2(_uid, label, vec2, editable) => {
                    self.add_vec2(label.as_str(), *vec2, *editable);
                }
                _ => {}
            }
        }
    }
    fn widget_on_layout_changed(&mut self) {}
}
