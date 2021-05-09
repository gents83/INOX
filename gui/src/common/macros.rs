#[macro_export]
macro_rules! implement_widget {
    ($Type:ident) => {
        use nrg_serialize::typetag;
        use $crate::{
            BaseWidget, ContainerFillType, HorizontalAlignment, Screen, VerticalAlignment, Widget,
            WidgetDataGetter, WidgetGraphics, WidgetNode, WidgetState, WidgetStyle,
        };

        #[typetag::serde]
        impl WidgetDataGetter for $Type {
            #[inline]
            fn get_shared_data(&self) -> &nrg_resources::SharedDataRw {
                self.data.get_shared_data()
            }
            #[inline]
            fn get_events(&self) -> &nrg_events::EventsRw {
                self.data.get_events()
            }
            #[inline]
            fn node(&self) -> &WidgetNode {
                self.data.node()
            }
            #[inline]
            fn node_mut(&mut self) -> &mut WidgetNode {
                self.data.node_mut()
            }
            #[inline]
            fn graphics(&self) -> &WidgetGraphics {
                self.data.graphics()
            }
            #[inline]
            fn graphics_mut(&mut self) -> &mut WidgetGraphics {
                self.data.graphics_mut()
            }
            #[inline]
            fn state(&self) -> &WidgetState {
                self.data.state()
            }
            #[inline]
            fn state_mut(&mut self) -> &mut WidgetState {
                self.data.state_mut()
            }
            #[inline]
            fn is_initialized(&self) -> bool {
                self.data.is_initialized()
            }
            #[inline]
            fn mark_as_initialized(&mut self) {
                self.data.mark_as_initialized()
            }
        }

        unsafe impl Send for $Type {}
        unsafe impl Sync for $Type {}
        impl BaseWidget for $Type {}

        #[typetag::serde]
        impl Widget for $Type {}

        impl $Type {
            #[inline]
            pub fn stroke(&mut self, stroke: u32) -> &mut Self {
                let stroke = Screen::convert_size_from_pixels([stroke as _, stroke as _].into()).x;
                self.graphics_mut().set_stroke(stroke);
                self
            }
            #[inline]
            pub fn selectable(&mut self, selectable: bool) -> &mut Self {
                self.state_mut().set_selectable(selectable);
                self
            }
            #[inline]
            pub fn draggable(&mut self, draggable: bool) -> &mut Self {
                self.state_mut().set_draggable(draggable);
                self
            }
            #[inline]
            pub fn position(&mut self, pos_in_px: nrg_math::Vector2) -> &mut Self {
                self.set_position(pos_in_px);
                self
            }
            #[inline]
            pub fn size(&mut self, size_in_px: nrg_math::Vector2) -> &mut Self {
                self.set_size(size_in_px);
                self
            }
            #[inline]
            pub fn horizontal_alignment(&mut self, alignment: HorizontalAlignment) -> &mut Self {
                self.state_mut().set_horizontal_alignment(alignment);
                self
            }
            #[inline]
            pub fn vertical_alignment(&mut self, alignment: VerticalAlignment) -> &mut Self {
                self.state_mut().set_vertical_alignment(alignment);
                self
            }
            #[inline]
            pub fn visible(&mut self, visible: bool) -> &mut Self {
                self.set_visible(visible);
                self
            }
            #[inline]
            pub fn style(&mut self, style: WidgetStyle) -> &mut Self {
                self.state_mut().set_style(style);
                self
            }
            #[inline]
            pub fn border_style(&mut self, style: WidgetStyle) -> &mut Self {
                self.state_mut().set_border_style(style);
                self
            }
            #[inline]
            pub fn fill_type(&mut self, fill_type: ContainerFillType) -> &mut Self {
                self.state_mut().fill_type(fill_type);
                self
            }
            #[inline]
            pub fn keep_fixed_height(&mut self, keep_fixed_height: bool) -> &mut Self {
                self.state_mut().keep_fixed_height(keep_fixed_height);
                self
            }
            #[inline]
            pub fn keep_fixed_width(&mut self, keep_fixed_width: bool) -> &mut Self {
                self.state_mut().keep_fixed_width(keep_fixed_width);
                self
            }
            #[inline]
            pub fn space_between_elements(&mut self, space_in_px: u32) -> &mut Self {
                self.state_mut().space_between_elements(space_in_px);
                self
            }
            #[inline]
            pub fn use_space_before_and_after(
                &mut self,
                use_space_before_after: bool,
            ) -> &mut Self {
                self.state_mut()
                    .use_space_before_and_after(use_space_before_after);
                self
            }
        }
    };
}

#[macro_export]
macro_rules! implement_widget_with_data {
    ($Type:ident) => {
        use nrg_events::EventsRw;
        use nrg_resources::SharedDataRw;

        $crate::implement_widget!($Type);

        impl $Type {
            pub fn new(shared_data: &SharedDataRw, events_rw: &EventsRw) -> $Type {
                let mut w = $Type {
                    data: WidgetData::new(shared_data, events_rw),
                };
                w.init();
                w
            }
        }
    };
}

#[macro_export]
macro_rules! implement_widget_with_custom_members {
    ($Type:ident { $($field:ident : $value:expr),+ }) => {
        use nrg_events::EventsRw;
        use nrg_resources::SharedDataRw;

        $crate::implement_widget!($Type);

        impl $Type {
            pub fn new(shared_data: &SharedDataRw, events_rw: &EventsRw) -> $Type {
                let mut w = $Type {
                    data: WidgetData::new(shared_data, events_rw),
                    $($field: $value),+
                };
                w.init();
                w
            }
        }
    };
}
