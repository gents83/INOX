use super::widget::*;
use nrg_serialize::*;

pub struct WidgetNode {
    id: UID,
    children: Vec<Box<dyn WidgetBase>>,
}

impl Default for WidgetNode {
    fn default() -> Self {
        Self {
            id: generate_random_uid(),
            children: Vec::new(),
        }
    }
}

impl WidgetNode {
    pub fn get_id(&self) -> UID {
        self.id
    }
    pub fn add_child<W: 'static + WidgetTrait>(&mut self, widget: Widget<W>) -> &mut Self {
        self.children.push(Box::new(widget));
        self
    }
    pub fn propagate_on_children<F>(&mut self, mut f: F) -> &mut Self
    where
        F: FnMut(&mut dyn WidgetBase),
    {
        self.children.iter_mut().for_each(|w| f(w.as_mut()));
        self
    }

    pub fn propagate_on_child<F>(&mut self, uid: UID, mut f: F)
    where
        F: FnMut(&mut dyn WidgetBase),
    {
        if let Some(index) = self.children.iter().position(|child| child.id() == uid) {
            let w = &mut self.children[index as usize];
            return f(w.as_mut());
        }
    }
}
