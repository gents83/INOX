use nrg_math::Vector4;
use nrg_messenger::Message;
use nrg_serialize::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use crate::{
    implement_widget_with_custom_members, InternalWidget, WidgetData, DEFAULT_WIDGET_SIZE,
};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Indicator {
    data: WidgetData,
    #[serde(skip)]
    is_blinking: bool,
    refresh_time: Duration,
    #[serde(skip, default = "Instant::now")]
    elapsed_time: Instant,
}
implement_widget_with_custom_members!(Indicator {
    is_blinking: true,
    refresh_time: Duration::from_millis(500),
    elapsed_time: Instant::now()
});

impl Indicator {
    fn update_blinkng(&mut self) {
        if self.elapsed_time.elapsed() >= self.refresh_time {
            let blinking = self.is_blinking;
            self.elapsed_time = Instant::now();

            if !blinking {
                self.style(WidgetStyle::DefaultText)
                    .border_style(WidgetStyle::DefaultText)
                    .border_width(1.);
            } else {
                self.style(WidgetStyle::Invisible)
                    .border_style(WidgetStyle::Invisible)
                    .border_width(0.);
            }
            self.is_blinking = !blinking;
        }
    }
}

impl InternalWidget for Indicator {
    fn widget_init(&mut self) {
        if self.is_initialized() {
            return;
        }
        self.draggable(false)
            .size([2., DEFAULT_WIDGET_SIZE[1] - 2.].into())
            .vertical_alignment(VerticalAlignment::Stretch)
            .horizontal_alignment(HorizontalAlignment::None)
            .selectable(false)
            .style(WidgetStyle::DefaultText)
            .border_style(WidgetStyle::DefaultText)
            .border_width(1.);
    }

    fn widget_update(&mut self, _drawing_area_in_px: Vector4) {
        self.update_blinkng();
    }

    fn widget_uninit(&mut self) {}
    fn widget_process_message(&mut self, _msg: &dyn Message) {}
}
