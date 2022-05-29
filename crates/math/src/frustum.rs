use cgmath::InnerSpace;

use crate::{Mat4Ops, Matrix4, VecBaseFloat, Vector2, Vector3, Vector4};

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

#[derive(Default)]
pub struct Frustum {
    pub faces: [Plane; Faces::Count as usize],
}

pub fn convert_in_3d(normalized_pos: Vector2, view: Matrix4, proj: Matrix4) -> (Vector3, Vector3) {
    let ray_end = Vector4::new(
        normalized_pos.x * 2. - 1.,
        normalized_pos.y * 2. - 1.,
        1.,
        1.,
    );

    let inv_proj = proj.inverse();
    let inv_view = view.inverse();

    let ray_start_world = view.translation();

    let mut ray_end_camera = inv_proj * ray_end;
    ray_end_camera /= ray_end_camera.w;
    let mut ray_end_world = inv_view * ray_end_camera;
    ray_end_world /= ray_end_world.w;

    (ray_start_world.xyz(), ray_end_world.xyz())
}

//From LearnOpenGL Frustum Culling
pub fn compute_frustum_planes(view: Matrix4, proj: Matrix4) -> Frustum {
    let mut frustum = Frustum::default();

    let (ntl, ftl) = convert_in_3d(Vector2::new(-1., 1.), view, proj);
    let (ntr, _ftr) = convert_in_3d(Vector2::new(1., 1.), view, proj);

    let (nbl, fbl) = convert_in_3d(Vector2::new(-1., -1.), view, proj);
    let (nbr, fbr) = convert_in_3d(Vector2::new(1., -1.), view, proj);

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
        -frustum.faces[Faces::Near as usize].normal.dot(ntr);

    frustum.faces[Faces::Far as usize].normal = view.direction().normalize();
    frustum.faces[Faces::Far as usize].distance =
        -frustum.faces[Faces::Far as usize].normal.dot(ftl);

    frustum.faces[Faces::Top as usize].normal = (ntr - ntl)
        .normalized()
        .cross((ftl - ntl).normalized())
        .normalize();
    frustum.faces[Faces::Top as usize].distance =
        -frustum.faces[Faces::Top as usize].normal.dot(ntl);

    frustum.faces[Faces::Bottom as usize].normal = (nbl - nbr)
        .normalized()
        .cross((fbr - nbr).normalized())
        .normalize();
    frustum.faces[Faces::Bottom as usize].distance =
        -frustum.faces[Faces::Bottom as usize].normal.dot(nbr);

    frustum.faces[Faces::Left as usize].normal = (ntl - nbl)
        .normalized()
        .cross((fbl - nbl).normalized())
        .normalize();
    frustum.faces[Faces::Left as usize].distance =
        -frustum.faces[Faces::Left as usize].normal.dot(nbl);

    frustum.faces[Faces::Right as usize].normal = (nbr - ntr)
        .normalized()
        .cross((fbr - ntr).normalized())
        .normalize();
    frustum.faces[Faces::Right as usize].distance =
        -frustum.faces[Faces::Right as usize].normal.dot(ntr);

    frustum
}
