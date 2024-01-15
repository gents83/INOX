use crate::{
    unproject, Degrees, Mat4Ops, Matrix4, VecBase, VecBaseFloat, Vector2, Vector3, Vector4,
};

pub enum Faces {
    Near,
    Far,
    Top,
    Bottom,
    Left,
    Right,
    Count,
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
            Faces::Near => 0,
            Faces::Far => 1,
            Faces::Top => 2,
            Faces::Bottom => 3,
            Faces::Left => 4,
            Faces::Right => 5,
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

pub fn normalize_plane(plane: Vector4) -> Vector4 {
    plane / plane.xyz().length()
}

pub fn convert_in_3d(pos_2d: Vector2, view: &Matrix4, proj: &Matrix4) -> (Vector3, Vector3) {
    let ray_start = Vector3::new(pos_2d.x, pos_2d.y, 0.);
    let ray_end = Vector3::new(ray_start.x, ray_start.y, 1.);
    let ray_start_world = unproject(ray_start, *view, *proj);
    let ray_end_world = unproject(ray_end, *view, *proj);

    (ray_start_world.xyz(), ray_end_world.xyz())
}

pub fn compute_frustum(
    view: &Matrix4,
    near: f32,
    far: f32,
    fov: Degrees,
    aspect_ratio: f32,
) -> Frustum {
    let mut frustum = Frustum::default();
    /*
    (frustum.ntl, frustum.ftl) = convert_in_3d(Vector2::new(-1., -1.), view, proj);
    (frustum.ntr, frustum.ftr) = convert_in_3d(Vector2::new(1., 1.), view, proj);

    (frustum.nbl, frustum.fbl) = convert_in_3d(Vector2::new(-1., -1.), view, proj);
    (frustum.nbr, frustum.fbr) = convert_in_3d(Vector2::new(1., -1.), view, proj);
    */

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

    let position = view.translation();
    let facing = view.forward().normalized();
    let up = view.up().normalized();
    let right = view.right().normalized();
    let tang = (fov * 0.5).0.to_radians().tan();
    let nh = near * tang;
    let nw = nh * aspect_ratio;
    let fh = far * tang;
    let fw = fh * aspect_ratio;
    let nc = position - facing * near;
    let fc = position - facing * far;

    // compute the 4 corners of the frustum on the near plane
    frustum.ntl = nc + up * nh - right * nw;
    frustum.ntr = nc + up * nh + right * nw;
    frustum.nbl = nc - up * nh - right * nw;
    frustum.nbr = nc - up * nh + right * nw;

    // compute the 4 corners of the frustum on the far plane
    frustum.ftl = fc + up * fh - right * fw;
    frustum.ftr = fc + up * fh + right * fw;
    frustum.fbl = fc - up * fh - right * fw;
    frustum.fbr = fc - up * fh + right * fw;

    frustum.faces[Faces::Near as usize].normal = facing;
    frustum.faces[Faces::Near as usize].distance =
        frustum.faces[Faces::Near as usize].normal.dot_product(nc);

    frustum.faces[Faces::Far as usize].normal = -facing;
    frustum.faces[Faces::Far as usize].distance =
        frustum.faces[Faces::Far as usize].normal.dot_product(fc);

    let aux = nc + up * nh;
    frustum.faces[Faces::Top as usize].normal =
        ((aux - position).normalized()).cross(right).normalized();
    frustum.faces[Faces::Top as usize].distance =
        frustum.faces[Faces::Top as usize].normal.dot_product(aux);

    let aux = nc - up * nh;
    frustum.faces[Faces::Bottom as usize].normal =
        (right).cross((aux - position).normalized()).normalized();
    frustum.faces[Faces::Bottom as usize].distance = frustum.faces[Faces::Bottom as usize]
        .normal
        .dot_product(aux);

    let aux = nc - right * nw;
    frustum.faces[Faces::Left as usize].normal =
        ((aux - position).normalized()).cross(up).normalized();
    frustum.faces[Faces::Left as usize].distance =
        frustum.faces[Faces::Left as usize].normal.dot_product(aux);

    let aux = nc + right * nw;
    frustum.faces[Faces::Right as usize].normal =
        (up).cross((aux - position).normalized()).normalized();
    frustum.faces[Faces::Right as usize].distance =
        frustum.faces[Faces::Left as usize].normal.dot_product(aux);

    frustum
}
