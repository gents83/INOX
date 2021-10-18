use std::{
    sync::Once,
    sync::{Arc, RwLock},
};

use nrg_core::JobHandlerRw;
use nrg_messenger::MessengerRw;
use nrg_resources::SharedDataRc;
use nrg_serialize::generate_uid_from_string;

use crate::{Screen, WidgetNode, DEFAULT_WIDGET_HEIGHT};

pub struct GuiInternal {
    widgets_root: WidgetNode,
    shared_data: SharedDataRc,
    global_messenger: MessengerRw,
    job_handler: JobHandlerRw,
}

impl GuiInternal {
    #[inline]
    fn new(
        shared_data: SharedDataRc,
        global_messenger: MessengerRw,
        job_handler: JobHandlerRw,
    ) -> Self {
        Self {
            widgets_root: WidgetNode::default(),
            shared_data,
            global_messenger,
            job_handler,
        }
    }

    #[inline]
    pub fn get_root(&self) -> &WidgetNode {
        &self.widgets_root
    }

    #[inline]
    pub fn get_root_mut(&mut self) -> &mut WidgetNode {
        &mut self.widgets_root
    }
}

static mut GUI: Option<Arc<RwLock<GuiInternal>>> = None;
static mut INIT: Once = Once::new();

pub struct Gui {}

impl Gui {
    pub fn create(
        shared_data: SharedDataRc,
        global_messenger: MessengerRw,
        job_handler: JobHandlerRw,
    ) {
        unsafe {
            INIT.call_once(|| {
                GUI = Some(Arc::new(RwLock::new(GuiInternal::new(
                    shared_data,
                    global_messenger,
                    job_handler,
                ))));
            });
        }
    }
    #[inline]
    fn get_and_init_once() -> &'static Option<Arc<RwLock<GuiInternal>>> {
        unsafe {
            debug_assert!(GUI.is_some(), "Gui has never been created");
            &GUI
        }
    }
    #[inline]
    pub fn get() -> Arc<RwLock<GuiInternal>> {
        let gui = Gui::get_and_init_once();
        gui.as_ref().unwrap().clone()
    }

    #[inline]
    pub fn add_additional_job<F>(name: &str, func: F)
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        let gui_category = generate_uid_from_string("GUI");
        Gui::get()
            .read()
            .unwrap()
            .job_handler
            .write()
            .unwrap()
            .add_job(&gui_category, name, func);
    }

    #[inline]
    pub fn update_widgets(job_handler: &JobHandlerRw, first_reduce_draw_area: bool) {
        let mut widget_area = Screen::get_draw_area();
        let mut next_area = widget_area;
        let gui_category = generate_uid_from_string("GUI");
        Gui::get()
            .write()
            .unwrap()
            .get_root_mut()
            .get_children()
            .iter()
            .enumerate()
            .for_each(|(i, w)| {
                let widget = w.clone();
                if first_reduce_draw_area && i == 0 {
                    let size = widget.read().unwrap().state().get_size();
                    next_area.x = 0.;
                    next_area.y = size.y * Screen::get_scale_factor() + DEFAULT_WIDGET_HEIGHT;
                    next_area.z = Screen::get_size().x;
                    next_area.w = Screen::get_size().y
                        - size.y * Screen::get_scale_factor()
                        - DEFAULT_WIDGET_HEIGHT;
                }
                let job_name = String::from(widget.read().unwrap().node().get_name());
                job_handler
                    .write()
                    .unwrap()
                    .add_job(&gui_category, job_name.as_str(), move || {
                        widget.write().unwrap().update(widget_area, widget_area);
                    });
                widget_area = next_area;
            });
    }

    #[inline]
    pub fn invalidate_all_widgets() {
        Gui::get()
            .write()
            .unwrap()
            .get_root_mut()
            .get_children()
            .iter()
            .for_each(|w| {
                w.write().unwrap().mark_as_dirty();
            });
    }
}
