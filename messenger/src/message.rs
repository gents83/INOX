use std::any::{type_name, Any};

use crate::MessageBox;

pub trait Message: Send + Sync + Any {
    fn as_any(&self) -> &dyn Any;
    fn redo(&self, events_rw: &MessageBox);
    fn undo(&self, events_rw: &MessageBox);

    #[inline]
    fn get_type_name(&self) -> String {
        let mut str = type_name::<Self>()
            .split(':')
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
            .to_string();
        str.push_str(" - ");
        str.push_str(self.get_debug_info().as_str());
        str
    }
    fn get_debug_info(&self) -> String;
    fn as_boxed(&self) -> Box<dyn Message>;
}
