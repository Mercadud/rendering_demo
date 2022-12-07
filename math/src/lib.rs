pub use glam::*;

#[derive(Clone, Copy, Debug)]
pub struct Location {
    pub rotation: Vec3,
    pub position: Vec3,
    pub scale: Vec3
}

impl Location {
    pub fn new(rot: [f32; 3], pos: [f32; 3], scale: [f32; 3]) -> Self {
        Self {
            rotation: rot.into(),
            position: pos.into(),
            scale: scale.into()
        }
    }

    pub fn calculate_matrix(&self) -> glam::Mat4 {
        Mat4::from_scale(self.scale) *
        Mat4::from_translation(self.position)
            * Mat4::from_euler(
                glam::EulerRot::XYZ,
                self.rotation.x,
                self.rotation.y,
                self.rotation.z,
            )
    }

    pub fn translation_matrix(&self) -> Mat4 {
        Mat4::from_translation(self.position)
    }

    pub fn rotation_matrix(&self) -> Mat4 {
        Mat4::from_euler(
            glam::EulerRot::XYZ,
            self.rotation.x,
            self.rotation.y,
            self.rotation.z,
        )
    }

    pub fn ez_camera_matrix(&self) -> Mat4 {
        Mat4::from_euler(glam::EulerRot::XYZ, self.rotation.x, 0.0, 0.0)
            * Mat4::from_euler(glam::EulerRot::XYZ, 0.0, self.rotation.y, 0.0)
            * Mat4::from_translation(self.position)
    }

    pub fn move_from_look(&mut self, distance: f32) {
        self.position.y += self.rotation.x.sin() * distance;
        self.position.z += self.rotation.y.cos() * distance;
        self.position.x += (self.rotation.y + deg_2_rad(90.0)).cos() * distance;
    }

    pub fn straffe_from_look(&mut self, distance: f32) {
        let dir_y = self.rotation.y + deg_2_rad(90.0);
        self.position.z += dir_y.cos() * distance;
        self.position.x += (dir_y + deg_2_rad(90.0)).cos() * distance;
    }
}

#[inline]
pub fn deg_2_rad(degrees: f32) -> f32 {
    degrees * std::f32::consts::PI / 180.0
}

#[inline]
pub fn rad_2_deg(rad: f32) -> f32 {
    rad * 180.0 / std::f32::consts::PI
}

#[inline]
pub fn perspective_rh(aspect_ratio: f32) -> [[f32; 4]; 4] {
    glam::Mat4::perspective_rh(std::f32::consts::FRAC_PI_2, aspect_ratio, 0.1, 1000.0)
        .to_cols_array_2d()
}

#[inline]
pub fn translation_from_matrix(mat4: Mat4) -> Vec3 {
    Vec3::new(mat4.w_axis[0], mat4.w_axis[1], mat4.w_axis[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deg_2_rad_test() {
        assert_eq!((1000.0 * deg_2_rad(57.2958)).round() / 1000.0, 1.0);
    }

    #[test]
    fn rad_2_deg_test() {
        assert_eq!(114.5916, (10000.0 * rad_2_deg(2.0)).round() / 10000.0);
    }

    #[test]
    fn translation_from_matrix_test() {
        let location = Location::new([0.0, 0.0, 0.0], [1.0, 20.0, 30.0], [1.0, 1.0, 1.0]);
        let matrix = location.calculate_matrix();

        assert_eq!(translation_from_matrix(matrix), location.position);
    }
}
