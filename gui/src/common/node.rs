use std::any::{type_name, Any};

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
    #[inline]
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
    #[inline]
    pub fn get_id(&self) -> Uid {
        self.id
    }

    #[inline]
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    #[inline]
    pub fn set_name(&mut self, name: &str) -> &mut Self {
        self.name = String::from(name);
        self
    }

    #[inline]
    pub fn add_child(&mut self, mut widget: Box<dyn Widget>) -> &mut Self {
        widget.node_mut().parent_id = self.id;
        self.children.push(widget);
        self
    }

    #[inline]
    pub fn remove_children(&mut self) -> &mut Self {
        self.children.iter_mut().for_each(|w| {
            w.node_mut().parent_id = INVALID_UID;
            w.uninit();
        });
        self.children.clear();
        self
    }

    #[inline]
    pub fn remove_child(&mut self, uid: Uid) -> &mut Self {
        self.children.iter_mut().for_each(|w| {
            if w.as_ref().id() == uid {
                w.node_mut().parent_id = INVALID_UID;
                w.uninit();
            }
        });
        self.children.retain(|w| w.as_ref().id() != uid);
        self
    }

    #[inline]
    pub fn get_children(&self) -> &Vec<Box<dyn Widget>> {
        &self.children
    }

    #[inline]
    pub fn get_child<W>(&mut self, uid: Uid) -> Option<&mut W>
    where
        W: Widget + 'static,
    {
        let mut result: Option<&mut W> = None;
        self.children.iter_mut().for_each(|w| {
            if w.id() == uid {
                let mut is_same_widget_type = <dyn Any>::is::<W>(w);
                if !is_same_widget_type {
                    is_same_widget_type |= type_name::<W>().contains(w.get_type());
                }
                if is_same_widget_type {
                    unsafe {
                        let boxed = Box::from_raw(w.as_mut());
                        let ptr = Box::into_raw(boxed);
                        let widget = ptr as *mut W;
                        result = Some(&mut *widget);
                    }
                }
            } else if result.is_none() {
                result = w.node_mut().get_child(uid);
            }
        });
        result
    }

    #[inline]
    pub fn get_parent(&self) -> Uid {
        self.parent_id
    }

    #[inline]
    pub fn has_parent(&self) -> bool {
        !self.parent_id.is_nil()
    }

    #[inline]
    pub fn get_num_children(&self) -> usize {
        self.children.len()
    }

    #[inline]
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    #[inline]
    pub fn has_child(&self, uid: Uid) -> bool {
        let mut found = false;
        self.children.iter().for_each(|w| {
            if w.id() == uid {
                found = true;
            }
        });
        found
    }

    #[inline]
    pub fn propagate_on_children<F>(&self, mut f: F)
    where
        F: FnMut(&dyn Widget),
    {
        self.children.iter().for_each(|w| f(w.as_ref()));
    }
    #[inline]
    pub fn get_children_mut(&mut self) -> &mut Vec<Box<dyn Widget>> {
        &mut self.children
    }

    #[inline]
    pub fn propagate_on_children_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut dyn Widget),
    {
        self.children.iter_mut().for_each(|w| f(w.as_mut()));
    }

    #[inline]
    pub fn propagate_on_child<F>(&self, uid: Uid, mut f: F)
    where
        F: FnMut(&dyn Widget),
    {
        if let Some(index) = self.children.iter().position(|child| child.id() == uid) {
            let w = &self.children[index as usize];
            return f(w.as_ref());
        }
    }

    #[inline]
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
