#![allow(dead_code)]
use std::any::{type_name, TypeId};

use nrg_gui::{Widget, WidgetCreator};
use nrg_messenger::MessengerRw;
use nrg_resources::SharedDataRw;

pub type CreationCallback =
    dyn Fn(&nrg_resources::SharedDataRw, &nrg_messenger::MessengerRw) -> Box<dyn Widget>;

struct NodesData {
    pub typeid: TypeId,
    pub func: Box<CreationCallback>,
    pub name: String,
}

pub struct NodesRegistry {
    registry: Vec<NodesData>,
    shared_data: SharedDataRw,
    global_messenger: MessengerRw,
}

unsafe impl Send for NodesRegistry {}
unsafe impl Sync for NodesRegistry {}

impl NodesRegistry {
    pub fn new(shared_data: &SharedDataRw, global_messenger: &MessengerRw) -> Self {
        Self {
            registry: Vec::new(),
            shared_data: shared_data.clone(),
            global_messenger: global_messenger.clone(),
        }
    }
    pub fn register<W>(&mut self) -> &mut Self
    where
        W: WidgetCreator,
    {
        let widget_name = type_name::<W>()
            .split(':')
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
            .to_string();
        self.registry.push(NodesData {
            typeid: TypeId::of::<W>(),
            func: Box::new(W::create_widget),
            name: widget_name,
        });
        self
    }

    pub fn count(&self) -> usize {
        self.registry.len()
    }

    fn get_index_from_type(&self, typeid: &TypeId) -> Option<usize> {
        self.registry.iter().position(|n| n.typeid == *typeid)
    }

    fn get_index_from_name(&self, name: &str) -> Option<usize> {
        self.registry.iter().position(|n| n.name == name)
    }

    pub fn get_name_from_index(&self, index: usize) -> &str {
        debug_assert!(index < self.registry.len());
        self.registry[index].name.as_str()
    }

    pub fn create_from_index(&self, index: usize) -> Box<dyn Widget> {
        debug_assert!(index < self.registry.len());
        let call_fn = self.registry[index].func.as_ref();
        call_fn(&self.shared_data, &self.global_messenger)
    }

    pub fn create_from_type(&self, typeid: TypeId) -> Box<dyn Widget> {
        if let Some(index) = self.get_index_from_type(&typeid) {
            self.create_from_index(index)
        } else {
            panic!("Trying to create an type not registered {:?}", typeid);
        }
    }

    pub fn create_from_name(&self, name: String) -> Box<dyn Widget> {
        if let Some(index) = self.get_index_from_name(&name) {
            self.create_from_index(index)
        } else {
            panic!("Trying to create an type not registered {:?}", name);
        }
    }
}
