pub const DEFAULT_AREA_SIZE: u32 = 4096;

#[derive(Clone)]
pub struct Area {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Default for Area {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }
}

impl Area {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
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

#[derive(Clone)]
struct AreaList {
    list: Vec<Area>,
}

impl Default for AreaList {
    fn default() -> Self {
        Self { list: Vec::new() }
    }
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

    pub fn find(&self, width: u32, height: u32) -> Option<usize> {
        self.list
            .iter()
            .position(|a| a.width >= width && a.height >= height)
    }
    pub fn remove(&mut self, area: Area) {
        if let Some(index) = self.list.iter().position(|a| {
            a.x == area.x && a.width == area.width && a.y == area.y && a.height == area.height
        }) {
            self.list.remove(index);
        }
    }
    pub fn pop(&mut self, index: usize) -> Area {
        self.list.remove(index)
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
                    first_area = first.clone();
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
    free: AreaList,
    occupied: AreaList,
}

impl Default for AreaAllocator {
    fn default() -> Self {
        Self {
            free: AreaList::new(&[Area::new(
                0,
                0,
                DEFAULT_AREA_SIZE as _,
                DEFAULT_AREA_SIZE as _,
            )]),
            occupied: AreaList::default(),
        }
    }
}

impl AreaAllocator {
    pub fn allocate(&mut self, width: u32, height: u32) -> Option<&Area> {
        self.free.collapse();
        if let Some(index) = self.free.find(width, height) {
            let old_area = self.free.pop(index);
            let new_area = Area::new(old_area.x, old_area.y, width, height);
            self.occupied.insert(new_area);

            if old_area.width > width {
                self.free.insert(Area::new(
                    old_area.x + width,
                    old_area.y,
                    old_area.width - width,
                    height,
                ));
            }
            if old_area.height > height {
                self.free.insert(Area::new(
                    old_area.x,
                    old_area.y + height,
                    width,
                    old_area.height - height,
                ));
            }
            if old_area.width > width && old_area.height > height {
                self.free.insert(Area::new(
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

    pub fn remove(&mut self, area: Area) {
        self.occupied.remove(area.clone());
        self.free.insert(area);
    }
}
