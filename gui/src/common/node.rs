use nrg_serialize::{generate_random_uid, Deserialize, Serialize, Uid, INVALID_UID};

use crate::Widget;

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct WidgetNode {
    id: Uid,
    name: String,
    parent_id: Uid,
    children: Vec<Box<dyn Widget>>,
}

impl Default for WidgetNode {
    fn default() -> Self {
        Self {
            id: generate_random_uid(),
            name: String::from("no-name"),
            parent_id: INVALID_UID,
            children: Vec::new(),
        }
    }
}

impl WidgetNode {
    pub fn get_id(&self) -> Uid {
        self.id
    }
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }
    pub fn set_name(&mut self, name: &str) -> &mut Self {
        self.name = String::from(name);
        self
    }
    pub fn add_child(&mut self, mut widget: Box<dyn Widget>) -> &mut Self {
        widget.node_mut().parent_id = self.id;
        self.children.push(widget);
        self
    }

    pub fn remove_children(&mut self) -> &mut Self {
        self.children.iter_mut().for_each(|w| {
            w.node_mut().parent_id = INVALID_UID;
        });
        self.children.clear();
        self
    }

    pub fn remove_child(&mut self, uid: Uid) -> &mut Self {
        self.children.iter_mut().for_each(|w| {
            if w.as_ref().id() == uid {
                w.node_mut().parent_id = INVALID_UID;
            }
        });
        self.children.retain(|w| w.as_ref().id() != uid);
        self
    }

    pub fn get_children(&self) -> &Vec<Box<dyn Widget>> {
        &self.children
    }
    pub fn get_child<W>(&mut self, uid: Uid) -> Option<&mut W>
    where
        W: Widget,
    {
        let mut result: Option<&mut W> = None;
        self.children.iter_mut().for_each(|w| {
            if w.id() == uid {
                unsafe {
                    let boxed = Box::from_raw(w.as_mut());
                    let ptr = Box::into_raw(boxed);
                    let widget = ptr as *mut W;
                    result = Some(&mut *widget);
                }
            } else if result.is_none() {
                result = w.node_mut().get_child(uid);
            }
        });
        result
    }
    pub fn get_parent(&self) -> Uid {
        self.parent_id
    }
    pub fn has_parent(&self) -> bool {
        !self.parent_id.is_nil()
    }
    pub fn get_num_children(&self) -> usize {
        self.children.len()
    }
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
    pub fn has_child(&self, uid: Uid) -> bool {
        let mut found = false;
        self.children.iter().for_each(|w| {
            if w.id() == uid {
                found = true;
            }
        });
        found
    }
    pub fn propagate_on_children<F>(&self, mut f: F)
    where
        F: FnMut(&dyn Widget),
    {
        self.children.iter().for_each(|w| f(w.as_ref()));
    }
    pub fn propagate_on_children_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut dyn Widget),
    {
        self.children.iter_mut().for_each(|w| f(w.as_mut()));
    }
    pub fn propagate_on_child<F>(&self, uid: Uid, mut f: F)
    where
        F: FnMut(&dyn Widget),
    {
        if let Some(index) = self.children.iter().position(|child| child.id() == uid) {
            let w = &self.children[index as usize];
            return f(w.as_ref());
        }
    }
    pub fn propagate_on_child_mut<F>(&mut self, uid: Uid, mut f: F)
    where
        F: FnMut(&mut dyn Widget),
    {
        if let Some(index) = self.children.iter().position(|child| child.id() == uid) {
            let w = &mut self.children[index as usize];
            return f(w.as_mut());
        }
    }
}
