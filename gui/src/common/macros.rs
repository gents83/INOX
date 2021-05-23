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
            fn load_override(
                &mut self,
                shared_data: nrg_resources::SharedDataRw,
                global_messenger: nrg_messenger::MessengerRw,
            ) {
                self.data.load_override(shared_data, global_messenger);
            }
            #[inline]
            fn get_shared_data(&self) -> &nrg_resources::SharedDataRw {
                self.data.get_shared_data()
            }
            #[inline]
            fn get_global_messenger(&self) -> &nrg_messenger::MessengerRw {
                self.data.get_global_messenger()
            }
            #[inline]
            fn get_global_dispatcher(&self) -> nrg_messenger::MessageBox {
                self.data.get_global_dispatcher()
            }
            #[inline]
            fn get_listener(&self) -> nrg_messenger::Listener {
                self.data.get_listener()
            }
            #[inline]
            fn get_messagebox(&self) -> nrg_messenger::MessageBox {
                self.data.get_messagebox()
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
            pub fn register_to_listen_event<Msg>(&mut self) -> &mut Self
            where
                Msg: Message,
            {
                self.data.register_to_listen_event::<Msg>();
                self
            }
            #[inline]
            pub fn unregister_to_listen_event<Msg>(&mut self) -> &mut Self
            where
                Msg: Message,
            {
                self.data.unregister_to_listen_event::<Msg>();
                self
            }
            #[inline]
            pub fn stroke(&mut self, stroke: u32) -> &mut Self {
                let stroke = Screen::convert_size_from_pixels([stroke as _, stroke as _].into()).x;
                self.graphics_mut().set_stroke(stroke);
                self
            }
            #[inline]
            pub fn selectable(&mut self, selectable: bool) -> &mut Self {
                if selectable {
                    self.register_to_listen_event::<nrg_platform::MouseEvent>();
                } else {
                    self.unregister_to_listen_event::<nrg_platform::MouseEvent>();
                }
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
                self.invalidate_layout();
                self
            }
            #[inline]
            pub fn vertical_alignment(&mut self, alignment: VerticalAlignment) -> &mut Self {
                self.state_mut().set_vertical_alignment(alignment);
                self.invalidate_layout();
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
                self.invalidate_layout();
                self
            }
            #[inline]
            pub fn keep_fixed_height(&mut self, keep_fixed_height: bool) -> &mut Self {
                self.state_mut().keep_fixed_height(keep_fixed_height);
                self.invalidate_layout();
                self
            }
            #[inline]
            pub fn keep_fixed_width(&mut self, keep_fixed_width: bool) -> &mut Self {
                self.state_mut().keep_fixed_width(keep_fixed_width);
                self.invalidate_layout();
                self
            }
            #[inline]
            pub fn space_between_elements(&mut self, space_in_px: u32) -> &mut Self {
                self.state_mut().space_between_elements(space_in_px);
                self.invalidate_layout();
                self
            }
            #[inline]
            pub fn use_space_before_and_after(
                &mut self,
                use_space_before_after: bool,
            ) -> &mut Self {
                self.state_mut()
                    .use_space_before_and_after(use_space_before_after);
                self.invalidate_layout();
                self
            }
        }
    };
}

#[macro_export]
macro_rules! implement_widget_with_data {
    ($Type:ident) => {
        $crate::implement_widget!($Type);

        impl $Type {
            #[inline]
            pub fn new(
                shared_data: &nrg_resources::SharedDataRw,
                global_messenger: &nrg_messenger::MessengerRw,
            ) -> $Type {
                let mut w = $Type {
                    data: WidgetData::new(shared_data.clone(), global_messenger.clone()),
                };
                w.init();
                w
            }

            #[inline]
            pub fn load(
                shared_data: &nrg_resources::SharedDataRw,
                global_messenger: &nrg_messenger::MessengerRw,
                filepath: std::path::PathBuf,
            ) -> $Type {
                let mut w = $Type {
                    data: WidgetData::new(shared_data.clone(), global_messenger.clone()),
                };
                nrg_serialize::deserialize_from_file(&mut w, filepath);
                w.data
                    .load_override(shared_data.clone(), global_messenger.clone());
                w.init();
                w
            }
        }
    };
}

#[macro_export]
macro_rules! implement_widget_with_custom_members {
    ($Type:ident { $($field:ident : $value:expr),+ }) => {
        $crate::implement_widget!($Type);

        impl $Type {

            #[inline]
            pub fn new(shared_data: &nrg_resources::SharedDataRw, global_messenger: &nrg_messenger::MessengerRw) -> $Type {
                let mut w = $Type {
                    data: WidgetData::new(shared_data.clone(), global_messenger.clone()),
                    $($field: $value),+
                };
                w.init();
                w
            }


            #[inline]
            pub fn load(
                shared_data: &nrg_resources::SharedDataRw,
                global_messenger: &nrg_messenger::MessengerRw,
                filepath: std::path::PathBuf,
            ) -> $Type {
                let mut w = $Type {
                    data: WidgetData::new(shared_data.clone(), global_messenger.clone()),
                    $($field: $value),+
                };
                nrg_serialize::deserialize_from_file(&mut w, filepath);
                w.data.load_override(shared_data.clone(), global_messenger.clone());
                w.init();
                w
            }
        }
    };
}
