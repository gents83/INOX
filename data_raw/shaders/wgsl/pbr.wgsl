#import "utils.wgsl"
#import "common.wgsl"


struct PbrData {
    dimensions: vec2<u32>,
    albedo_texture_index: u32,
    normals_texture_index: u32,
    material_params_texture_index: u32,
};


@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<uniform> pbr_data: PbrData;
@group(0) @binding(2)
var<storage, read> materials: Materials;
@group(0) @binding(3)
var<storage, read> textures: Textures;

@group(1) @binding(0)
var render_target: texture_storage_2d_array<rgba8unorm, read_write>;


#import "texture_utils.wgsl"

fn load(texture_index: u32, v: vec2<i32>) -> vec4<f32> {  
    return sample_texture(vec3<f32>(vec2<f32>(f32(v.x),f32(v.y)), f32(texture_index)));
}


//@id(0) override workgroup_x: f32 = 1920.;
//@id(1) override workgroup_y: f32 = 1080.;
//@id(2) override workgroup_z: f32 = 1.;
let workgroup_x: i32 = 16;
let workgroup_y: i32 = 16;
let workgroup_z: i32 = 1;

@compute
@workgroup_size(16, 16, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(local_invocation_index) local_invocation_index: u32, 
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let pixel = vec2<i32>(i32(global_invocation_id.x), i32(global_invocation_id.y));
    if (pixel.x >= workgroup_x || pixel.y >= workgroup_y)
    {
        return;
    }
    
    let albedo_params = load(pbr_data.albedo_texture_index, pixel);
    let normal_params = load(pbr_data.normals_texture_index, pixel);
    let instance_params = load(pbr_data.material_params_texture_index, pixel);

    let normal = unpack_normal(normal_params.xy);

    //textureStore(render_target, pixel.xy, 0, vec4<f32>(normal.x, normal.y, normal.z, normal_params.z));
    textureStore(render_target, pixel.xy, 0, vec4<f32>(1.,1.,1.,1.));
}