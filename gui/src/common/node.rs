use super::*;
use nrg_serialize::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct WidgetNode {
    id: UID,
    #[serde(skip)]
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
    pub fn add_child<W: 'static>(&mut self, widget: Widget<W>) -> &mut Self
    where
        W: WidgetTrait,
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
        W: WidgetTrait,
    {
        let mut result: Option<&mut Widget<W>> = None;
        self.children.iter_mut().for_each(|w| {
            if w.id() == uid {
                unsafe {
                    let boxed = Box::from_raw(w.as_mut());
                    let ptr = Box::into_raw(boxed);
                    let widget = ptr as *mut Widget<W>;
                    result = Some(&mut *widget);
                }
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
