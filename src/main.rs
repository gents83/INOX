use nrg_core::*;
use nrg_math::*;

fn main() {
    let _entity = Entity::new();
    let _transf = Matrix4::identity();
    _transf.print();    

    let mut window = 
    Window::create( String::from("NRGWindow"),
                   String::from("NRG - Window"),
                   100, 100,
                   800, 600 );

    loop {
        if !handle_message( &mut window ) {
            break;
        }
    }
}