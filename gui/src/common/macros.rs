#[macro_export]
macro_rules! implement_widget {
    ($Type:ident) => {
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
        }
    };
}

#[macro_export]
macro_rules! implement_container {
    ($Type:ident) => {
        impl ContainerTrait for $Type {
            fn get_container_data(&self) -> &ContainerData {
                &self.container
            }
            fn get_container_data_mut(&mut self) -> &mut ContainerData {
                &mut self.container
            }
        }
    };
}
