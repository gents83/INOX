use sabi_math::Vector4;
use sabi_serialize::{generate_random_uid, Uid, INVALID_UID};

use crate::TextureId;

pub const DEFAULT_AREA_SIZE: u32 = 4096;

#[derive(Clone, Copy)]
pub struct Area {
    pub id: TextureId,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Default for Area {
    fn default() -> Self {
        Self {
            id: INVALID_UID,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }
}
impl From<&Area> for [f32; 4] {
    fn from(area: &Area) -> Self {
        [
            area.x as f32,
            area.y as f32,
            area.width as f32,
            area.height as f32,
        ]
    }
}
impl From<&Area> for Vector4 {
    fn from(area: &Area) -> Self {
        Vector4::new(
            area.x as f32,
            area.y as f32,
            area.width as f32,
            area.height as f32,
        )
    }
}

impl Area {
    pub fn new(id: &TextureId, x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            id: *id,
            x,
            y,
            width,
            height,
        }
    }
    #[inline]
    pub fn is_adjacent(&self, area: &Area) -> bool {
        if self.x == area.x && self.width == area.width && self.y + self.height == area.y {
            return true;
        }
        if self.x == area.x && self.width == area.width && area.y + area.height == self.y {
            return true;
        }
        if self.y == area.y && self.height == area.height && self.x + self.width == area.x {
            return true;
        }
        if self.y == area.y && self.height == area.height && area.x + area.width == self.x {
            return true;
        }
        false
    }
    #[inline]
    pub fn combine(&mut self, area: &Area) -> bool {
        if self.x == area.x && self.width == area.width && self.y + self.height == area.y {
            self.height += area.height;
            return true;
        }
        if self.x == area.x && self.width == area.width && area.y + area.height == self.y {
            self.y -= area.height;
            self.height += area.height;
            return true;
        }
        if self.y == area.y && self.height == area.height && self.x + self.width == area.x {
            self.width += area.width;
            return true;
        }
        if self.y == area.y && self.height == area.height && area.x + area.width == self.x {
            self.x -= area.width;
            self.width += area.width;
            return true;
        }
        false
    }
}

#[derive(Default, Clone)]
struct AreaList {
    list: Vec<Area>,
}

impl AreaList {
    pub fn new(list: &[Area]) -> Self {
        Self {
            list: list.to_vec(),
        }
    }

    pub fn insert(&mut self, area: Area) {
        self.list.push(area);
    }

    pub fn last(&self) -> Option<&Area> {
        self.list.last()
    }

    pub fn get_area(&self, id: &TextureId) -> Option<&Area> {
        self.list.iter().find(|&area| area.id == *id)
    }

    pub fn find(&self, width: u32, height: u32) -> Option<usize> {
        self.list
            .iter()
            .position(|a| a.width >= width && a.height >= height)
    }
    pub fn remove(&mut self, area: &Area) {
        if let Some(index) = self.list.iter().position(|a| {
            a.x == area.x && a.width == area.width && a.y == area.y && a.height == area.height
        }) {
            self.list.remove(index);
        }
    }
    pub fn pop(&mut self, index: usize) -> Area {
        self.list.remove(index)
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn collapse(&mut self) {
        if self.list.len() <= 1 {
            return;
        }
        loop {
            let mut collapsed = 0;
            let mut collapsed_list = Vec::new();
            while !self.list.is_empty() {
                let mut first_area: Area = Area::default();
                let mut rest_areas: Vec<Area> = Vec::new();
                if let Some((first, rest)) = self.list.split_first_mut() {
                    first_area = *first;
                    rest_areas = rest.to_vec();
                }
                while !rest_areas.is_empty() {
                    if let Some(index) = rest_areas.iter().position(|a| a.is_adjacent(&first_area))
                    {
                        first_area.combine(&rest_areas[index]);
                        rest_areas.remove(index);
                        self.list.remove(index + 1);
                        collapsed += 1;
                    } else {
                        break;
                    }
                }
                collapsed_list.push(first_area);
                self.list.remove(0);
            }
            self.list = collapsed_list;
            if collapsed == 0 {
                break;
            }
        }
    }
}

#[derive(Clone)]
pub struct AreaAllocator {
    id: Uid,
    free: AreaList,
    occupied: AreaList,
}

impl AreaAllocator {
    pub fn new(width: u32, height: u32) -> Self {
        let id = generate_random_uid();
        Self {
            free: AreaList::new(&[Area::new(&id, 0, 0, width as _, height as _)]),
            occupied: AreaList::default(),
            id,
        }
    }
    pub fn allocate(&mut self, id: &TextureId, width: u32, height: u32) -> Option<&Area> {
        self.free.collapse();
        if let Some(index) = self.free.find(width, height) {
            let old_area = self.free.pop(index);
            let new_area = Area::new(id, old_area.x, old_area.y, width, height);
            self.occupied.insert(new_area);

            if old_area.width > width {
                self.free.insert(Area::new(
                    &self.id,
                    old_area.x + width,
                    old_area.y,
                    old_area.width - width,
                    height,
                ));
            }
            if old_area.height > height {
                self.free.insert(Area::new(
                    &self.id,
                    old_area.x,
                    old_area.y + height,
                    width,
                    old_area.height - height,
                ));
            }
            if old_area.width > width && old_area.height > height {
                self.free.insert(Area::new(
                    &self.id,
                    old_area.x + width,
                    old_area.y + height,
                    old_area.width - width,
                    old_area.height - height,
                ));
            }

            return self.occupied.last();
        }
        None
    }

    pub fn get_area(&self, id: &TextureId) -> Option<Area> {
        if let Some(area) = self.occupied.get_area(id) {
            return Some(*area);
        }
        None
    }

    pub fn remove_texture(&mut self, id: &TextureId) -> bool {
        if let Some(area) = self.get_area(id) {
            self.remove(area);
            return true;
        }
        false
    }

    pub fn remove(&mut self, mut area: Area) {
        self.occupied.remove(&area);
        area.id = self.id;
        self.free.insert(area);
        self.free.collapse();
    }

    pub fn is_empty(&self) -> bool {
        self.occupied.is_empty()
    }
}
