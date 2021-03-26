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
    pub fn add_child<W>(&mut self, widget: Widget<W>) -> &mut Self
    where
        W: WidgetTrait + Default + 'static,
    {
        self.children.push(Box::new(widget));
        self
    }

    pub fn remove_children(&mut self) -> &mut Self {
        self.children.clear();
        self
    }

    pub fn get_child<W>(&mut self, uid: UID) -> Option<&mut Widget<W>>
    where
        W: WidgetTrait + Default + 'static,
    {
        let mut result: Option<&mut Widget<W>> = None;
        self.children.iter_mut().for_each(|w| {
            if w.id() == uid {
                result = w.as_any_mut().downcast_mut::<Widget<W>>();
            } else if result.is_none() {
                result = w.get_data_mut().node.get_child(uid);
            }
        });
        result
    }

    pub fn get_num_children(&self) -> usize {
        self.children.len()
    }

    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
    pub fn propagate_on_children<F>(&self, mut f: F)
    where
        F: FnMut(&dyn WidgetBase),
    {
        self.children.iter().for_each(|w| f(w.as_ref()));
    }
    pub fn propagate_on_children_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut dyn WidgetBase),
    {
        self.children.iter_mut().for_each(|w| f(w.as_mut()));
    }
    pub fn propagate_on_child<F>(&self, uid: UID, mut f: F)
    where
        F: FnMut(&dyn WidgetBase),
    {
        if let Some(index) = self.children.iter().position(|child| child.id() == uid) {
            let w = &self.children[index as usize];
            return f(w.as_ref());
        }
    }
    pub fn propagate_on_child_mut<F>(&mut self, uid: UID, mut f: F)
    where
        F: FnMut(&mut dyn WidgetBase),
    {
        if let Some(index) = self.children.iter().position(|child| child.id() == uid) {
            let w = &mut self.children[index as usize];
            return f(w.as_mut());
        }
    }
}
