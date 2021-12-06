use std::{any::Any, collections::HashMap};

use sabi_serialize::{generate_uid_from_string, Uid};

pub type DataId = Uid;
pub trait LogicContextData: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn duplicate(&self) -> Box<dyn LogicContextData>;
}
impl Clone for Box<dyn LogicContextData> {
    fn clone(&self) -> Box<dyn LogicContextData> {
        self.duplicate()
    }
}

#[derive(Default, Clone)]
pub struct LogicContext {
    data: HashMap<DataId, Box<dyn LogicContextData>>,
}

impl LogicContext {
    pub fn set<T>(&mut self, name: &str, data: T) -> DataId
    where
        T: LogicContextData + 'static,
    {
        let id = generate_uid_from_string(name);
        self.data.insert(id, Box::new(data));
        id
    }
    pub fn get<T>(&self, id: DataId) -> Option<&T>
    where
        T: LogicContextData + 'static,
    {
        self.data
            .get(&id)
            .and_then(|data| data.as_any().downcast_ref::<T>())
    }
}
