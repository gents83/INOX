use nrg_app::*;

fn main() {
    let mut app = App::new();

    app.create_phase(phases::UPDATE);
    
    app.run_once();
}