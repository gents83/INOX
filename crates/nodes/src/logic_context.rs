use std::{any::Any, collections::HashMap};

use inox_resources::{Resource, ResourceTrait};
use inox_serialize::{generate_uid_from_string, Uid};

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
impl<T> LogicContextData for Resource<T>
where
    T: ResourceTrait,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn duplicate(&self) -> Box<dyn LogicContextData> {
        Box::new(self.clone())
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
    pub fn get_with_name<T>(&self, name: &str) -> Option<&T>
    where
        T: LogicContextData + 'static,
    {
        let id = generate_uid_from_string(name);
        self.get::<T>(id)
    }
}
