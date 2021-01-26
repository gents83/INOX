
#[repr(C)]
pub struct Entity {
    pub transform: u32,
}

impl Default for Entity {
    fn default() -> Self {
        Self { 
            transform: 0, 
        }
    }
}

impl Entity {
    pub fn set(&mut self, integer:u32) -> &mut Self {
        self.transform = integer;
        self
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn create_entity() -> Entity {
    Entity::default()
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn create_entity_with_param(integer:u32) -> Entity {
    let mut e = Entity::default();
    e.set(integer);
    e
}