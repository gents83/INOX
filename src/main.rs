use nrg_graphics::*;
use nrg_core::*;
use nrg_math::*;
use nrg_platform::*;

fn main() {    
    let _entity = Entity::new();
    let _transf = Matrix4f::identity();
    println!("{:?}", _transf);

    let _pos = Vector2u::new(10, 10);
    let size = Vector2u::new(1024, 768);

    let window = 
    Window::create( String::from("NRGWindow"),
                   String::from("NRG - Window"),
                   _pos.x, _pos.y,
                   size.x, size.y );
                   
    let mut model_transform:Matrix4f = Matrix4f::identity();
    let cam_pos:Vector3f = [0.0, 16.0, -64.0].into();
    
    let mut renderer = Renderer::new(&window.handle, false);
    renderer.set_viewport_size(size);
    let mut default_render_pass = renderer.create_default_render_pass();
    let mut material = Material::create(&mut renderer.device, "C:\\PROJECTS\\NRG\\data\\vert.spv", "C:\\PROJECTS\\NRG\\data\\frag.spv");
    material.add_texture(&renderer.device, "C:\\PROJECTS\\NRG\\data\\Test.bmp");

    let vertices: [VertexData; 4] = [
        VertexData { pos: [-20., -20., 0.].into(), normal: [0., 0., 1.].into(), color: [1., 0., 0.].into(), tex_coord: [1., 0.].into()},
        VertexData { pos: [ 20., -20., 0.].into(), normal: [0., 0., 1.].into(), color: [0., 1., 0.].into(), tex_coord: [0., 0.].into()},
        VertexData { pos: [ 20.,  20., 0.].into(), normal: [0., 0., 1.].into(), color: [0., 0., 1.].into(), tex_coord: [0., 1.].into()},
        VertexData { pos: [-20.,  20., 0.].into(), normal: [0., 0., 1.].into(), color: [1., 1., 1.].into(), tex_coord: [1., 1.].into()},
    ]; 
    let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];

    let mut mesh = Mesh::create();
    mesh.set_vertices(&renderer.device, &vertices)
        .set_indices(&renderer.device, &indices);

    let mut frame_count = 0;
    
    loop 
    {                
        let is_ended = window.update();
        if is_ended
        {
            break;
        }

        let rotation = Matrix4::from_axis_angle([0.0, 0.0, 0.0].into(), Degree(0.1).into());
        model_transform.set_translation([0.0, 0.0, 0.0].into());
        model_transform = rotation * model_transform;

        let time = std::time::Instant::now();
        let mut result = renderer.begin_frame();
        if result {
            default_render_pass.begin(&renderer.device);

            material.update_uniform_buffer(&renderer.device, &model_transform, cam_pos);
            material.prepare_pipeline(&renderer.device, &default_render_pass);
            mesh.draw(&renderer.device);
            
            default_render_pass.end(&renderer.device);
            result = renderer.end_frame();
        }

        if !result {
            material.destroy(&renderer.device);
            default_render_pass.destroy(&renderer.device);

            renderer.device.recreate_swap_chain();

            default_render_pass = renderer.create_default_render_pass();
   
            material = Material::create(&mut renderer.device, "C:\\PROJECTS\\NRG\\data\\vert.spv", "C:\\PROJECTS\\NRG\\data\\frag.spv");
            material.add_texture(&renderer.device, "C:\\PROJECTS\\NRG\\data\\Test.bmp");
        } 

        println!("Frame {} rendered in {} ", frame_count, time.elapsed().as_secs_f32());
        frame_count += 1;
    }

    material.destroy(&renderer.device);
    default_render_pass.destroy(&renderer.device);
}
