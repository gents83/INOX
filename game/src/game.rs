use nrg_app::*;

use super::system::*;

const UPDATE_PHASE:&str = "UPDATE_PHASE";

#[repr(C)]
pub struct Game {
    pub game_name: String,
}

impl Default for Game {
    fn default() -> Self {
        Self{
            game_name: String::from("Game"),
        }
    }
}

impl Plugin for Game {
    fn build(&mut self, main_app: &mut App) {             
        let mut update_phase = PhaseWithSystems::new(UPDATE_PHASE); 
        let my_system = MySystem::new();
        
        update_phase.add_system(my_system);
        main_app.create_phase(update_phase); 
    }
}