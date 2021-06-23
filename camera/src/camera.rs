use nrg_math::{create_look_at, InnerSpace, Matrix4, NewAngle, Radians, Vector3};

pub struct Camera {
    eye_matrix: Matrix4,
    target_matrix: Matrix4,
}

pub struct CameraInput {
    pub movement: Vector3,
    pub rotation: Vector3,
    pub speed: f32,
}

impl Camera {
    pub fn new(position: Vector3, target: Vector3) -> Self {
        let target_matrix = Matrix4::from_translation(target);
        let eye_dir = (target - position).normalize();
        let right = eye_dir.cross([0., 1., 0.].into());
        let up = right.cross(eye_dir);
        let eye_matrix = create_look_at(position, target, up);
        Self {
            eye_matrix,
            target_matrix,
        }
    }

    #[inline]
    pub fn translate(&mut self, movement: Vector3) -> &mut Self {
        let translation = Matrix4::from_translation(movement);
        self.eye_matrix = translation * self.eye_matrix;
        self.target_matrix = translation * self.target_matrix;
        self
    }

    #[inline]
    pub fn rotate(&mut self, rotation_angle: Vector3) -> &mut Self {
        let rot_x = Matrix4::from_angle_x(Radians::new(rotation_angle.x));
        let rot_y = Matrix4::from_angle_y(Radians::new(rotation_angle.y));
        self.eye_matrix = rot_x * self.eye_matrix * rot_y;
        self
    }

    #[inline]
    pub fn get_view_matrix(&self) -> Matrix4 {
        self.eye_matrix
    }
}
