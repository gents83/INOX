use nrg_graphics::Renderer;

use nrg_platform::EventsRw;
use nrg_serialize::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use crate::{implement_widget, InternalWidget, WidgetData, DEFAULT_WIDGET_SIZE};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Indicator {
    #[serde(skip)]
    is_blinking: bool,
    #[serde(skip)]
    refresh_time: Duration,
    #[serde(skip, default = "Instant::now")]
    elapsed_time: Instant,
    data: WidgetData,
}
implement_widget!(Indicator);

impl Default for Indicator {
    fn default() -> Self {
        Self {
            is_blinking: true,
            refresh_time: Duration::from_millis(500),
            elapsed_time: Instant::now(),
            data: WidgetData::default(),
        }
    }
}

impl Indicator {
    fn update_blinkng(&mut self) {
        if self.elapsed_time.elapsed() >= self.refresh_time {
            let blinking = self.is_blinking;
            self.elapsed_time = Instant::now();

            if !blinking {
                self.style(WidgetStyle::FullActive)
                    .border_style(WidgetStyle::FullActive);
            } else {
                self.style(WidgetStyle::Invisible)
                    .border_style(WidgetStyle::Invisible);
            }
            self.is_blinking = !blinking;
        }
    }
}

impl InternalWidget for Indicator {
    fn widget_init(&mut self, _renderer: &mut Renderer) {
        if self.is_initialized() {
            return;
        }
        self.draggable(false)
            .size([2, DEFAULT_WIDGET_SIZE.y - 2].into())
            .vertical_alignment(VerticalAlignment::Stretch)
            .selectable(false)
            .style(WidgetStyle::FullActive)
            .border_style(WidgetStyle::FullActive);
    }

    fn widget_update(&mut self, _renderer: &mut Renderer, _events: &mut EventsRw) {
        self.update_blinkng();
    }

    fn widget_uninit(&mut self, _renderer: &mut Renderer) {}
}
