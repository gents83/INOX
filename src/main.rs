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
                   
    let mut model_transform_left:Matrix4f = Matrix4f::identity();
    let mut model_transform_right:Matrix4f = Matrix4f::identity();
    let rotation_left = Matrix4::from_axis_angle([0.0, 0.0, -1.0].into(), Degree(0.1).into());
    let rotation_right = Matrix4::from_axis_angle([0.0, 0.0, 1.0].into(), Degree(0.1).into());
    
    model_transform_left.set_translation([-30.0, 0.0, 0.0].into());
    model_transform_right.set_translation([30.0, 0.0, 0.0].into());

    let cam_pos:Vector3f = [0.0, 40.0, -100.0].into();
    
    let mut renderer = Renderer::new(&window.handle, false);
    renderer.set_viewport_size(size);
    let mut default_render_pass = renderer.create_default_render_pass();
    let mut pipeline = Pipeline::create(&mut renderer.device, "C:\\PROJECTS\\NRG\\data\\vert.spv", "C:\\PROJECTS\\NRG\\data\\frag.spv");
    
    let mut vertices: [VertexData; 4] = [
        VertexData { pos: [-20., -20., 0.].into(), normal: [0., 0., 1.].into(), color: [1., 0., 0.].into(), tex_coord: [1., 0.].into()},
        VertexData { pos: [ 20., -20., 0.].into(), normal: [0., 0., 1.].into(), color: [0., 1., 0.].into(), tex_coord: [0., 0.].into()},
        VertexData { pos: [ 20.,  20., 0.].into(), normal: [0., 0., 1.].into(), color: [0., 0., 1.].into(), tex_coord: [0., 1.].into()},
        VertexData { pos: [-20.,  20., 0.].into(), normal: [0., 0., 1.].into(), color: [1., 1., 1.].into(), tex_coord: [1., 1.].into()},
    ]; 
    let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];

    let mut mesh_left = Mesh::create();
    for vertex in vertices.iter_mut() {
        vertex.color = [1., 0., 0.].into();
    }
    mesh_left.set_vertices(&renderer.device, &vertices)
             .set_indices(&renderer.device, &indices);
             
    let mut material_left = Material::create(&mut renderer.device, &pipeline);
    material_left.add_texture(&renderer.device, "C:\\PROJECTS\\NRG\\data\\Test.bmp");
        
    let mut mesh_right = Mesh::create();
    for vertex in vertices.iter_mut() {
        vertex.color = [0., 1., 0.].into();
    }
    mesh_right.set_vertices(&renderer.device, &vertices)
              .set_indices(&renderer.device, &indices);

    let mut material_right = Material::create(&mut renderer.device, &pipeline);
    material_right.add_texture(&renderer.device, "C:\\PROJECTS\\NRG\\data\\Test.bmp");

    let mut frame_count = 0;
    
    loop 
    {                
        let is_ended = window.update();
        if is_ended
        {
            break;
        }

        let time = std::time::Instant::now();
        let mut result = renderer.begin_frame();
        if result {
            default_render_pass.begin(&renderer.device);
            pipeline.prepare(&renderer.device, &default_render_pass);

            model_transform_left = rotation_left * model_transform_left;
            material_left.update_uniform_buffer(&renderer.device, &model_transform_left, cam_pos);
            mesh_left.draw(&renderer.device);
            
            model_transform_right = rotation_right * model_transform_right;
            material_right.update_uniform_buffer(&renderer.device, &model_transform_right, cam_pos);
            mesh_right.draw(&renderer.device);

            default_render_pass.end(&renderer.device);
            result = renderer.end_frame();
        }

        if !result {
            material_right.destroy(&renderer.device);
            material_left.destroy(&renderer.device);
            default_render_pass.destroy(&renderer.device);

            renderer.device.recreate_swap_chain();

            default_render_pass = renderer.create_default_render_pass();
               
            pipeline = Pipeline::create(&mut renderer.device, "C:\\PROJECTS\\NRG\\data\\vert.spv", "C:\\PROJECTS\\NRG\\data\\frag.spv");
            
            material_left = Material::create(&mut renderer.device, &pipeline);
            material_left.add_texture(&renderer.device, "C:\\PROJECTS\\NRG\\data\\Test.bmp");
            
            material_right = Material::create(&mut renderer.device, &pipeline);
            material_right.add_texture(&renderer.device, "C:\\PROJECTS\\NRG\\data\\Test.bmp");
        } 

        println!("Frame {} rendered in {} ", frame_count, time.elapsed().as_secs_f32());
        frame_count += 1;
    }

    material_right.destroy(&renderer.device);
    material_left.destroy(&renderer.device);
    default_render_pass.destroy(&renderer.device);
}
