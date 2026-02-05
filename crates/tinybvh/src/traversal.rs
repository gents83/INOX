use crate::Ray;

/// Intersector for BVH and nodes intersection.
pub trait Intersector {
    /// Intersect this instance with a ray.
    ///
    /// [`Ray::hit`] is mutated with the intersection data.
    ///
    /// Returns the number of steps (A.K.A intersections) performed.
    fn intersect(&self, ray: &mut Ray) -> u32;
}
