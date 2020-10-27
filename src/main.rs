use nrg_core::*;
use nrg_math::*;

fn main() {
    let _entity = Entity::new();
    let _transf = Matrix4::identity();
    _transf.print();    

    let _handle = TrustedHandle::new();

    let mut window = create_window( "NRGWindow", "NRG - Window" );

    loop {
        if !handle_message( &mut window ) {
            break;
        }
    }
}