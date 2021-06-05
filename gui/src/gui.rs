use std::{
    sync::Once,
    sync::{Arc, RwLock},
};

use nrg_core::JobHandlerRw;
use nrg_messenger::MessengerRw;
use nrg_resources::SharedDataRw;

use crate::{Screen, WidgetNode};

pub struct GuiInternal {
    widgets_root: WidgetNode,
    shared_data: SharedDataRw,
    global_messenger: MessengerRw,
    job_handler: JobHandlerRw,
}

impl GuiInternal {
    #[inline]
    fn new(
        shared_data: SharedDataRw,
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
        shared_data: SharedDataRw,
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
        Gui::get()
            .read()
            .unwrap()
            .job_handler
            .write()
            .unwrap()
            .add_job(name, func);
    }

    #[inline]
    pub fn update_widgets(job_handler: &JobHandlerRw) {
        Gui::get()
            .write()
            .unwrap()
            .get_root_mut()
            .get_children()
            .iter()
            .for_each(|w| {
                let widget = w.clone();
                let job_name = format!("widget[{}]", widget.read().unwrap().node().get_name());
                job_handler
                    .write()
                    .unwrap()
                    .add_job(job_name.as_str(), move || {
                        widget
                            .write()
                            .unwrap()
                            .update(Screen::get_draw_area(), Screen::get_draw_area());
                    })
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
