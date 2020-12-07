use nrg_core::*;
use nrg_math::*;
use nrg_platform::*;

fn main() {    
    let _entity = Entity::new();
    let _transf = Matrix4f::identity();
    println!("{:?}", _transf);

    let _pos = Vector2u::new(100, 100);
    let _size = Vector2u::new(1024, 768);

    let window = 
    Window::create( String::from("NRGWindow"),
                   String::from("NRG - Window"),
                   _pos.x, _pos.y,
                   _size.x, _size.y );
    loop 
    {
        let is_ended = window.update() ;
        if is_ended
        {
            break;
        }
    }

}
