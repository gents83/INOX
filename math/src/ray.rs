use std::mem::swap;

use cgmath::InnerSpace;

use crate::{Mat4Ops, Matrix4, Vector3};

pub fn compute_distance_between_ray_and_oob(
    ray_origin: Vector3,    // Ray origin, in world space
    ray_direction: Vector3, // Ray direction (NOT target position!), in world space. Must be normalize()'d.
    aabb_min: Vector3,      // Minimum X,Y,Z coords of the mesh when not transformed at all.
    aabb_max: Vector3, // Maximum X,Y,Z coords. Often aabb_min*-1 if your mesh is centered, but it's not always the case.
    model_matrix: Matrix4, // Transformation applied to the mesh (which will thus be also applied to its bounding box)
) -> bool {
    // Intersection method from Real-Time Rendering and Essential Mathematics for Games
    let mut t_min = 0.;
    let mut t_max = 10000.0;

    let model_position = model_matrix.translation();
    let delta = model_position - ray_origin;

    // Test intersection with the 2 planes perpendicular to the OBB's X axis
    {
        let x_axis = model_matrix.x.xyz();
        let e = x_axis.dot(delta);
        let f = ray_direction.dot(x_axis);

        if f.abs() > 0.001 {
            // Standard case
            let mut t1 = (e + aabb_min.x) / f; // Intersection with the "left" plane
            let mut t2 = (e + aabb_max.x) / f; // Intersection with the "right" plane

            // t1 and t2 now contain distances betwen ray origin and ray-plane intersections

            // We want t1 to represent the nearest intersection,
            // so if it's not the case, invert t1 and t2
            if t1 > t2 {
                swap(&mut t1, &mut t2); // swap t1 and t2
            }

            // tMax is the nearest "far" intersection (amongst the X,Y and Z planes pairs)
            if t2 < t_max {
                t_max = t2;
            }
            // tMin is the farthest "near" intersection (amongst the X,Y and Z planes pairs)
            if t1 > t_min {
                t_min = t1;
            }
            // If "far" is closer than "near", then there is NO intersection.
            if t_max < t_min {
                return false;
            }
        }
        // Rare case : the ray is almost parallel to the planes, so they don't have any "intersection"
        else if -e + aabb_min.x > 0. || -e + aabb_max.x < 0. {
            return false;
        }
    }

    // Test intersection with the 2 planes perpendicular to the OBB's Y axis
    // Exactly the same thing than above.
    {
        let y_axis = model_matrix.y.xyz();
        let e = y_axis.dot(delta);
        let f = ray_direction.dot(y_axis);

        if f.abs() > 0.001 {
            let mut t1 = (e + aabb_min.y) / f;
            let mut t2 = (e + aabb_max.y) / f;

            if t1 > t2 {
                swap(&mut t1, &mut t2);
            }

            if t2 < t_max {
                t_max = t2;
            }
            if t1 > t_min {
                t_min = t1;
            }
            if t_min > t_max {
                return false;
            }
        } else if -e + aabb_min.y > 0. || -e + aabb_max.y < 0. {
            return false;
        }
    }

    // Test intersection with the 2 planes perpendicular to the OBB's Z axis
    // Exactly the same thing than above.
    {
        let z_axis = model_matrix.z.xyz();
        let e = z_axis.dot(delta);
        let f = ray_direction.dot(z_axis);

        if f.abs() > 0.001 {
            let mut t1 = (e + aabb_min.z) / f;
            let mut t2 = (e + aabb_max.z) / f;

            if t1 > t2 {
                swap(&mut t1, &mut t2);
            }

            if t2 < t_max {
                t_max = t2;
            }
            if t1 > t_min {
                t_min = t1;
            }
            if t_min > t_max {
                return false;
            }
        } else if -e + aabb_min.z > 0. || -e + aabb_max.z < 0. {
            return false;
        }
    }
    true
}
