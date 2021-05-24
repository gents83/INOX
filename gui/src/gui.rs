use std::{
    borrow::Borrow,
    cell::{Ref, RefCell, RefMut},
    sync::Arc,
    sync::Once,
};

use nrg_messenger::MessengerRw;
use nrg_resources::SharedDataRw;

use crate::WidgetNode;

pub struct GuiInternal {
    widgets_root: WidgetNode,
    shared_data: SharedDataRw,
    global_messenger: MessengerRw,
}

impl GuiInternal {
    #[inline]
    fn new(shared_data: SharedDataRw, global_messenger: MessengerRw) -> Self {
        Self {
            widgets_root: WidgetNode::default(),
            shared_data,
            global_messenger,
        }
    }

    pub fn get_root(&self) -> &WidgetNode {
        &self.widgets_root
    }

    pub fn get_root_mut(&mut self) -> &mut WidgetNode {
        &mut self.widgets_root
    }
}

static mut GUI: Option<Arc<RefCell<GuiInternal>>> = None;
static mut INIT: Once = Once::new();

pub struct Gui {}

impl Gui {
    pub fn create(shared_data: SharedDataRw, global_messenger: MessengerRw) {
        unsafe {
            INIT.call_once(|| {
                GUI = Some(Arc::new(RefCell::new(GuiInternal::new(
                    shared_data,
                    global_messenger,
                ))));
            });
        }
    }
    #[inline]
    fn get_and_init_once() -> &'static Option<Arc<RefCell<GuiInternal>>> {
        unsafe {
            debug_assert!(GUI.is_some(), "Gui has never been created");
            &GUI
        }
    }
    #[inline]
    fn get_internal() -> &'static RefCell<GuiInternal> {
        let gui = Gui::get_and_init_once();
        gui.as_ref().unwrap().borrow()
    }
    #[inline]
    pub fn get() -> Ref<'static, GuiInternal> {
        Gui::get_internal().borrow()
    }
    #[inline]
    pub fn get_mut() -> RefMut<'static, GuiInternal> {
        Gui::get_internal().borrow_mut()
    }
}
