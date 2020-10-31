use nrg_core::*;
use nrg_math::*;
use nrg_platform::*;

fn main() {
    let _entity = Entity::new();
    let _transf = Matrix4::identity();
    _transf.print();    

    let window = 
    Window::create( String::from("NRGWindow"),
                   String::from("NRG - Window"),
                   100, 100,
                   1024, 768 );

    loop 
    {
        let is_ended = window.update() ;
        if is_ended
        {
            break;
        }
    }
}