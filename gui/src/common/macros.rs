#[macro_export]
macro_rules! implement_widget {
    ($Type:ident) => {
        use crate::{
            BaseWidget, ContainerFillType, ContainerTrait, HorizontalAlignment, Screen,
            VerticalAlignment, Widget, WidgetDataGetter, WidgetStyle,
        };
        use nrg_serialize::typetag;

        #[typetag::serde]
        impl WidgetDataGetter for $Type {
            #[inline]
            fn get_data(&self) -> &WidgetData {
                &self.data
            }
            #[inline]
            fn get_data_mut(&mut self) -> &mut WidgetData {
                &mut self.data
            }
        }

        unsafe impl Send for $Type {}
        unsafe impl Sync for $Type {}
        impl BaseWidget for $Type {}
        impl ContainerTrait for $Type {}

        #[typetag::serde]
        impl Widget for $Type {}

        impl $Type {
            pub fn stroke(&mut self, stroke: u32) -> &mut Self {
                let stroke: nrg_math::Vector3f =
                    Screen::convert_size_from_pixels([stroke, stroke].into()).into();
                self.get_data_mut().graphics.set_stroke(stroke.x);
                self
            }
            pub fn selectable(&mut self, selectable: bool) -> &mut Self {
                self.get_data_mut().state.set_selectable(selectable);
                self
            }
            pub fn draggable(&mut self, draggable: bool) -> &mut Self {
                self.get_data_mut().state.set_draggable(draggable);
                self
            }
            pub fn position(&mut self, pos_in_px: nrg_math::Vector2u) -> &mut Self {
                let offset: nrg_math::Vector2i =
                    pos_in_px.convert() - self.get_data().state.get_position().convert();
                self.translate(offset);
                self
            }
            pub fn size(&mut self, size_in_px: nrg_math::Vector2u) -> &mut Self {
                let scale: nrg_math::Vector2f = [
                    size_in_px.x as f32 / self.get_data().state.get_size().x as f32,
                    size_in_px.y as f32 / self.get_data().state.get_size().y as f32,
                ]
                .into();
                self.scale(scale);
                self
            }
            pub fn horizontal_alignment(&mut self, alignment: HorizontalAlignment) -> &mut Self {
                self.get_data_mut()
                    .state
                    .set_horizontal_alignment(alignment);
                self
            }
            pub fn vertical_alignment(&mut self, alignment: VerticalAlignment) -> &mut Self {
                self.get_data_mut().state.set_vertical_alignment(alignment);
                self
            }
            pub fn visible(&mut self, visible: bool) -> &mut Self {
                self.set_visible(visible);
                self
            }
            pub fn style(&mut self, style: WidgetStyle) -> &mut Self {
                self.get_data_mut().graphics.set_style(style);
                self
            }
            pub fn border_style(&mut self, style: WidgetStyle) -> &mut Self {
                self.get_data_mut().graphics.set_border_style(style);
                self
            }
            pub fn fill_type(&mut self, fill_type: ContainerFillType) -> &mut Self {
                self.get_data_mut().state.fill_type(fill_type);
                self
            }
            pub fn keep_fixed_height(&mut self, keep_fixed_height: bool) -> &mut Self {
                self.get_data_mut()
                    .state
                    .keep_fixed_height(keep_fixed_height);
                self
            }
            pub fn keep_fixed_width(&mut self, keep_fixed_width: bool) -> &mut Self {
                self.get_data_mut().state.keep_fixed_width(keep_fixed_width);
                self
            }
            pub fn space_between_elements(&mut self, space_in_px: u32) -> &mut Self {
                self.get_data_mut()
                    .state
                    .space_between_elements(space_in_px);
                self
            }
            pub fn use_space_before_and_after(
                &mut self,
                use_space_before_after: bool,
            ) -> &mut Self {
                self.get_data_mut()
                    .state
                    .use_space_before_and_after(use_space_before_after);
                self
            }
        }
    };
}
