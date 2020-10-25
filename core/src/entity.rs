pub struct Entity {
    transform: u32,
}

impl Entity {
    pub fn new() -> Self {
        Entity { transform: 0, }
    }
}

pub trait EntityInterface {
    fn get_transform(&self) -> u32;
}

impl EntityInterface for Entity
{
    fn get_transform(&self) -> u32
    {
        self.transform
    }
}