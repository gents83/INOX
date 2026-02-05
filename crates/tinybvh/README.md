# tinybvh-rs

Opinionated rust wrapper for [tinybvh](https://github.com/jbikker/tinybvh).

## Features

* `BVH`, `MBVH` `BVH8_CPU`, `CWBVH`
* Intersection

Unimplemented features:
* Optimize via `Verbose`
* Loading/saving from/to disk

For more information about each layout, have a look at the original [tinybvh](https://github.com/jbikker/tinybvh) library.

## Examples

### BVH Wald

```rust
use tinybvh_rs::{bvh, Intersector, Ray};

let primitives = vec![
    [-2.0, 1.0, -1.0, 0.0],    //
    [-1.0, 1.0, -1.0, 0.0],    // Left triangle
    [-2.0, 0.0, -1.0, 0.0],    //

    [2.0, 1.0, -1.0, 0.0],     //
    [2.0, 0.0, -1.0, 0.0],     // Right triangle
    [1.0, 0.0, -1.0, 0.0],     //
];

let bvh = bvh::BVH::new(primitives.as_slice().into()).unwrap();

// No intersection, ray pass between the primitives
let mut ray = Ray::new([0.0, 0.0, 0.0], [0.0, 0.0, -1.0]);
bvh.intersect(&mut ray);
println!("Hit distance: {}", ray.hit.t); // 1e30

// Intersects left primitive
let mut ray = Ray::new([-1.5, 0.5, 0.0], [0.0, 0.0, -1.0]);
bvh.intersect(&mut ray);
println!("Hit distance & primitive: {} / {}", ray.hit.t, ray.hit.prim); // 1.0 / 0

// Intersects right primitive
let mut ray = Ray::new([1.5, 0.45, 0.0], [0.0, 0.0, -1.0]);
bvh.intersect(&mut ray);
println!("Hit distance & primitive: {} / {}", ray.hit.t, ray.hit.prim); // 1.0 / 1
```

### BVH8_CPU

```rust
let mut bvh = bvh::BVH::new(primitives.as_slice().into()).unwrap();
bvh.split_leaves(4); // Required or BVH8_CPU will return an error
let mbvh = mbvh::BVH::new(&bvh);
let bvh8 = bvh8_cpu::BVH::new(&mbvh).unwrap();
bvh8.intersect(&mut ray);
```

### CWBVH

```rust
let mut bvh = bvh::BVH::new(primitives.as_slice().into()).unwrap();
bvh.split_leaves(3); // Required or the CWBVH will return an error

let mbvh = mbvh::BVH::new(&bvh);
let cwbvh = cwbvh::BVH::new(&mbvh).unwrap();

println!("{}", cwbvh.nodes());
println!("{}", cwbvh.primitives());
```

### Strided

If the vertices position are strided (located in a `Vertex` struct for instance),
you can enable the `strided` feature and use:

```rust
use tinybvh_rs::{Intersector, Ray};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 4],
    normal: [f32; 3],
}

let vertices = [
    Vertex {
        position: [-1.0, 1.0, -1.0, 0.0],
        normal: [0.0, 0.0, 1.0]
    },
    Vertex {
        position: [-0.5, 1.0, -1.0, 0.0],
        normal: [0.0, 0.0, 1.0]
    },
    Vertex {
        position: [-1.0, 0.0, -1.0, 0.0],
        normal: [0.0, 0.0, 1.0]
    },
];
let positions = pas::slice_attr!(vertices, [0].position);
let bvh = bvh::BVH::new(positions);
```
