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

#[no_mangle]
pub unsafe extern "C" fn create_entity() -> Entity {
    Entity::default()
}

#[no_mangle]
pub unsafe extern "C" fn create_entity_with_param(_integer:u32) -> Entity {
    let mut e = Entity::default();
    e.transform = _integer;
    e
}