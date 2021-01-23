use nrg_app::*;

fn main() {
    let mut app = App::new();

    let _game_plugin = load_plugin(&mut app, "C:\\PROJECTS\\NRG\\game\\target\\debug\\nrg_game.dll");
    
    app.run_once();
}