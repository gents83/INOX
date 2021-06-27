use nrg_math::{
    direction_to_euler_angles, Angle, InnerSpace, MatBase, Matrix4, NewAngle, Radians, Vector3,
    Zero,
};

pub struct Camera {
    position: Vector3,
    rotation: Vector3, //pitch, yaw, roll
    direction: Vector3,
    proj_matrix: Matrix4,
    is_flipped: bool,
}

pub struct CameraInput {
    pub movement: Vector3,
    pub rotation: Vector3,
    pub speed: f32,
}

impl Camera {
    pub fn new(position: Vector3, target: Vector3, is_flipped: bool) -> Self {
        let direction = (target - position).normalize();
        let rotation = direction_to_euler_angles(direction);
        let mut camera = Self {
            position,
            rotation,
            direction,
            proj_matrix: Matrix4::default_identity(),
            is_flipped,
        };
        camera.update();
        camera
    }

    #[inline]
    pub fn set_projection(
        &mut self,
        fov: f32,
        screen_width: f32,
        screen_height: f32,
        near: f32,
        far: f32,
    ) -> &mut Self {
        let proj =
            nrg_math::perspective(nrg_math::Deg(fov), screen_width / screen_height, near, far);

        #[rustfmt::skip]
        const OPENGL_TO_VULKAN_MATRIX: Matrix4 = Matrix4::new(
            -1.0, 0.0, 0.0, 0.0,
            0.0, -1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.0,
            0.0, 0.0, 0.5, 1.0,
        );

        if self.is_flipped {
            self.proj_matrix = OPENGL_TO_VULKAN_MATRIX * proj;
        } else {
            self.proj_matrix = proj;
        }

        self
    }

    #[inline]
    pub fn translate(&mut self, movement: Vector3) -> &mut Self {
        self.position += self.direction * movement.z;
        let up: Vector3 = [0., 1., 0.].into();
        let right = self.direction.cross(up).normalize();
        let up = right.cross(self.direction).normalize();
        self.position += right * movement.x;
        self.position += up * movement.y;
        self.update();
        self
    }

    #[inline]
    pub fn rotate(&mut self, rotation_angle: Vector3) -> &mut Self {
        self.rotation += rotation_angle;
        self.update();
        self
    }

    #[inline]
    fn update(&mut self) -> &mut Self {
        let mut forward = Vector3::zero();
        forward.x = Radians::new(self.rotation.y).cos() * Radians::new(self.rotation.x).cos();
        forward.y = Radians::new(self.rotation.x).sin();
        forward.z = Radians::new(self.rotation.y).sin() * Radians::new(self.rotation.x).cos();

        if self.is_flipped {
            forward.y *= -1.;
        }

        self.direction = forward.normalize();
        self
    }

    #[inline]
    pub fn get_view_matrix(&self) -> Matrix4 {
        let up: Vector3 = [0., 1., 0.].into();
        let right = self.direction.cross(up).normalize();
        let up = right.cross(self.direction).normalize();

        nrg_math::create_look_at(self.position, self.position + self.direction, up)
    }

    #[inline]
    pub fn get_proj_matrix(&self) -> Matrix4 {
        self.proj_matrix
    }
}
