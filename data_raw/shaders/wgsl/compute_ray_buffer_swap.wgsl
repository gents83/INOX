// Ray Buffer Swap Compute Shader
// Copies rays_next buffer into rays buffer for next bounce iteration

#import "ray_data.inc"

@group(0) @binding(0) var<storage, read> rays_next: Rays;
@group(0) @binding(1) var<storage, read_write> rays: Rays;

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let ray_index = global_id.x;
    
    // Bounds check
    if (ray_index >= arrayLength(&rays.data)) {
        return;
    }
    
    // Copy ray from rays_next to rays
    rays.data[ray_index] = rays_next.data[ray_index];
}
