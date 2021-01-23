use core::*;
use nrg_graphics::*;
use nrg_math::*;
use nrg_platform::*;

const VS_PATH: &str = "C:\\PROJECTS\\NRG\\data\\shaders\\compiled\\shader_vert.spv";
const FRAG_PATH: &str = "C:\\PROJECTS\\NRG\\data\\shaders\\compiled\\shader_frag.spv";
const IMAGE_PATH: &str = "C:\\PROJECTS\\NRG\\data\\textures\\Test.jpg";

const FONT_VS_PATH: &str = "C:\\PROJECTS\\NRG\\data\\shaders\\compiled\\text_shader_vert.spv";
const FONT_FRAG_PATH: &str = "C:\\PROJECTS\\NRG\\data\\shaders\\compiled\\text_shader_frag.spv";
const FONT_PATH: & str = "C:\\PROJECTS\\NRG\\data\\fonts\\BasicFont.ttf";

fn main() {        
/*
    let NRG_CORE = nrg::CoreLib::load();
    unsafe {
        let _entity = nrg::CreateEntity.unwrap()();
        let _myentity = nrg::CoreLib::CreateEntity();
        let _entity2 = nrg::CreateEntityWithParam.unwrap()(3);
        println!("{:?}", _entity.transform);
        println!("{:?}", _myentity.transform);
        println!("{:?}", _entity2.transform);
    }
*/
/*
    let _entity = Entity::default();
    let _transf = Matrix4f::identity();
    println!("{:?}", _transf);
*/
    let _pos = Vector2u::new(10, 10);
    let size = Vector2u::new(1024, 768);

    let window = 
    Window::create( String::from("NRGWindow"),
                   String::from("NRG - Window"),
                   _pos.x, _pos.y,
                   size.x, size.y );
                   
    let mut model_transform_left:Matrix4f = Matrix4f::identity();
    let mut model_transform_right:Matrix4f = Matrix4f::identity();
    let rotation_left = Matrix4::from_axis_angle([0.0, 0.0, 0.0].into(), Degree(0.1).into());
    let rotation_right = Matrix4::from_axis_angle([0.0, 0.0, 0.0].into(), Degree(0.1).into());
    
    model_transform_left.set_translation([-30.0, 0.0, 0.0].into());
    model_transform_right.set_translation([30.0, 0.0, 0.0].into());

    let cam_pos:Vector3f = [0.0, 40.0, -100.0].into();
    
    let mut renderer = Renderer::new(&window.handle, false);
    renderer.set_viewport_size(size);
    let mut default_render_pass = renderer.create_default_render_pass();
    let mut pipeline = Pipeline::create(&mut renderer.device, VS_PATH, FRAG_PATH);
    let mut ui_pipeline = Pipeline::create(&mut renderer.device, FONT_VS_PATH, FONT_FRAG_PATH);
    
    let mut vertices: [VertexData; 4] = [
        VertexData { pos: [-20., -20., 0.].into(), normal: [0., 0., 1.].into(), color: [1., 0., 0.].into(), tex_coord: [1., 0.].into()},
        VertexData { pos: [ 20., -20., 0.].into(), normal: [0., 0., 1.].into(), color: [0., 1., 0.].into(), tex_coord: [0., 0.].into()},
        VertexData { pos: [ 20.,  20., 0.].into(), normal: [0., 0., 1.].into(), color: [0., 0., 1.].into(), tex_coord: [0., 1.].into()},
        VertexData { pos: [-20.,  20., 0.].into(), normal: [0., 0., 1.].into(), color: [1., 1., 1.].into(), tex_coord: [1., 1.].into()},
    ]; 
    let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];

    let font = fonts::font::Font::new(&renderer.device, &ui_pipeline, FONT_PATH);
    let img = font.get_bitmap();

    let mut mesh_left = Mesh::create(&renderer.device);
    for vertex in vertices.iter_mut() {
        vertex.color = [1., 0., 0.].into();
    }
    mesh_left.set_vertices( &vertices)
             .set_indices(&indices)
             .finalize();
             
    let mut material_left = Material::create(&renderer.device, &pipeline);
    material_left.add_texture_from_image(&img);
        
    let mut mesh_right = Mesh::create(&renderer.device);
    for vertex in vertices.iter_mut() {
        vertex.color = [1., 1., 1.].into();
    }
    mesh_right.set_vertices(&vertices)
              .set_indices(&indices)
              .finalize();

    let mut material_right = Material::create(&renderer.device, &pipeline);
    material_right.add_texture_from_path(IMAGE_PATH);

    let mut time_per_frame:f32 = 1.0;
    loop 
    {                
        let is_ended = window.update();
        if is_ended
        {
            break;
        }
            
        let fps = (1.0 / time_per_frame) as u32;
        let time = std::time::Instant::now();
        let mut result = renderer.begin_frame();
        if result {
            default_render_pass.begin();
            pipeline.prepare(&default_render_pass);

            model_transform_left = rotation_left * model_transform_left;
            material_left.update_uniform_buffer(&model_transform_left, cam_pos);
            mesh_left.draw();
            
            model_transform_right = rotation_right * model_transform_right;
            material_right.update_uniform_buffer(&model_transform_right, cam_pos);
            mesh_right.draw();
            
            ui_pipeline.prepare(&default_render_pass);
            font.get_material().update_simple();

            let mut str:String = String::from("FPS = ");
            str += fps.to_string().as_str();
            let mut text_mesh = font.create_text(str.as_str(), [-0.9, -0.9].into(), 1.0);
            text_mesh.finalize()
                     .draw();

            str = String::from("Mauro Gentile aka gents");
            let mut signature_mesh = font.create_text( str.as_str(), [-0.9, 0.9].into(), 0.8);
            signature_mesh.set_vertex_color([0.2, 0.6, 1.0].into())
                          .finalize()
                          .draw();

            default_render_pass.end();
            result = renderer.end_frame();
        }

        if !result {
            default_render_pass.destroy();
            renderer.device.recreate_swap_chain();
            default_render_pass = renderer.create_default_render_pass();
        } 

        time_per_frame = time.elapsed().as_secs_f32();
    }

    material_right.destroy();
    material_left.destroy();
    default_render_pass.destroy();
}
