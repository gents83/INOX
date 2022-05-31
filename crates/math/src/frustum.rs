use cgmath::InnerSpace;

use crate::{unproject, Mat4Ops, Matrix4, VecBaseFloat, Vector2, Vector3, Vector4};

pub enum Faces {
    Top = 0,
    Bottom = 1,
    Right = 2,
    Left = 3,
    Far = 4,
    Near = 5,
    Count = 6,
}

impl From<Faces> for u32 {
    fn from(val: Faces) -> Self {
        let i: usize = val.into();
        i as _
    }
}

impl From<Faces> for usize {
    fn from(val: Faces) -> Self {
        match val {
            Faces::Top => 0,
            Faces::Bottom => 1,
            Faces::Right => 2,
            Faces::Left => 3,
            Faces::Far => 4,
            Faces::Near => 5,
            Faces::Count => 6,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Plane {
    pub normal: Vector3,
    pub distance: f32, //from origin
}

impl Default for Plane {
    fn default() -> Self {
        Plane {
            normal: Vector3::new(0., 0., 1.),
            distance: 0.,
        }
    }
}
impl From<Plane> for Vector4 {
    fn from(p: Plane) -> Self {
        [p.normal.x, p.normal.y, p.normal.z, p.distance].into()
    }
}
impl From<Plane> for [f32; 4] {
    fn from(p: Plane) -> Self {
        [p.normal.x, p.normal.y, p.normal.z, p.distance]
    }
}

pub struct Frustum {
    pub faces: [Plane; Faces::Count as usize],
    pub ntr: Vector3,
    pub ntl: Vector3,
    pub nbr: Vector3,
    pub nbl: Vector3,
    pub ftr: Vector3,
    pub ftl: Vector3,
    pub fbr: Vector3,
    pub fbl: Vector3,
}

impl Default for Frustum {
    fn default() -> Self {
        Frustum {
            faces: [Plane::default(); Faces::Count as usize],
            ntr: Vector3::new(1., 1., 0.),
            ntl: Vector3::new(-1., 1., 0.),
            nbr: Vector3::new(1., -1., 0.),
            nbl: Vector3::new(-1., -1., 0.),
            ftr: Vector3::new(1., 1., 1.),
            ftl: Vector3::new(-1., 1., 1.),
            fbr: Vector3::new(1., -1., 1.),
            fbl: Vector3::new(-1., -1., 1.),
        }
    }
}

pub fn convert_in_3d(
    normalized_pos: Vector2,
    view: &Matrix4,
    proj: &Matrix4,
) -> (Vector3, Vector3) {
    let ray_start = Vector3::new(normalized_pos.x * 2. - 1., normalized_pos.y * 2. - 1., 0.);
    let ray_end = Vector3::new(ray_start.x, ray_start.y, 1.);
    let ray_start_world = unproject(ray_start, *view, *proj);
    let ray_end_world = unproject(ray_end, *view, *proj);

    (ray_start_world.xyz(), ray_end_world.xyz())
}

pub fn compute_frustum(view: &Matrix4, proj: &Matrix4) -> Frustum {
    let mut frustum = Frustum::default();

    (frustum.ntl, frustum.ftl) = convert_in_3d(Vector2::new(0., 1.), view, proj);
    (frustum.ntr, frustum.ftr) = convert_in_3d(Vector2::new(1., 1.), view, proj);

    (frustum.nbl, frustum.fbl) = convert_in_3d(Vector2::new(0., 0.), view, proj);
    (frustum.nbr, frustum.fbr) = convert_in_3d(Vector2::new(1., 0.), view, proj);

    ////////////////////////////////
    // ftl                     ftr
    //  \                      /
    //   \                    /
    //   ntl----------------ntr
    //    |                  |
    //    |                  |
    //    |                  |
    //    |                  |
    //   nbl----------------nbr
    //   /                    \
    //  /                      \
    // fbl                     fbr
    ////////////////////////////////
    frustum.faces[Faces::Near as usize].normal = -view.direction().normalize();
    frustum.faces[Faces::Near as usize].distance =
        -frustum.faces[Faces::Near as usize].normal.dot(frustum.ntr);

    frustum.faces[Faces::Far as usize].normal = view.direction().normalize();
    frustum.faces[Faces::Far as usize].distance =
        -frustum.faces[Faces::Far as usize].normal.dot(frustum.ftl);

    frustum.faces[Faces::Top as usize].normal = (frustum.ntr - frustum.ntl)
        .normalized()
        .cross((frustum.ftl - frustum.ntl).normalized())
        .normalize();
    frustum.faces[Faces::Top as usize].distance =
        -frustum.faces[Faces::Top as usize].normal.dot(frustum.ntl);

    frustum.faces[Faces::Bottom as usize].normal = (frustum.nbl - frustum.nbr)
        .normalized()
        .cross((frustum.fbr - frustum.nbr).normalized())
        .normalize();
    frustum.faces[Faces::Bottom as usize].distance = -frustum.faces[Faces::Bottom as usize]
        .normal
        .dot(frustum.nbr);

    frustum.faces[Faces::Left as usize].normal = (frustum.ntl - frustum.nbl)
        .normalized()
        .cross((frustum.fbl - frustum.nbl).normalized())
        .normalize();
    frustum.faces[Faces::Left as usize].distance =
        -frustum.faces[Faces::Left as usize].normal.dot(frustum.nbl);

    frustum.faces[Faces::Right as usize].normal = (frustum.nbr - frustum.ntr)
        .normalized()
        .cross((frustum.fbr - frustum.ntr).normalized())
        .normalize();
    frustum.faces[Faces::Right as usize].distance =
        -frustum.faces[Faces::Right as usize].normal.dot(frustum.ntr);

    frustum
}
