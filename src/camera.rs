use std::f64::consts::PI;

use nalgebra::{Rotation3, Vector2, Vector3};

use crate::spherical_angle::{RainAngle, SphericalAngle};

// Camera is assumed to be at the origin
pub trait Camera: Sync {
    // returns the resolution of the camera
    fn resolution(&self) -> Vector2<u32>;

    // returns the ray direction through this pixel
    fn pixel_to_rain_angle(&self, pixel: Vector2<u32>) -> RainAngle;
}

pub struct EquirectangularCamera {
    // resolution of the camera
    resolution: Vector2<u32>,
    // view matrix for transforming from local space to world space
    // column vectors are right, up, facing in global space
    pub inverse_view_matrix: Rotation3<f64>,
}

impl Default for EquirectangularCamera {
    fn default() -> Self {
        let mut cam = EquirectangularCamera::new(1024, Default::default());
        cam.look_at(
            &Vector3::new(0_f64, 0_f64, 1_f64),
            &Vector3::new(0_f64, 1_f64, 0_f64),
        );
        cam
    }
}

impl EquirectangularCamera {
    pub fn new(height: u32, inverse_view_matrix: Rotation3<f64>) -> Self {
        EquirectangularCamera {
            resolution: Vector2::new(height, 2 * height),
            inverse_view_matrix,
        }
    }

    // makes the camera look at dir
    pub fn look_at(&mut self, dir: &Vector3<f64>, up: &Vector3<f64>) {
        // direction of z axis is -view direction
        let z = -dir.normalize();

        // direction of x axis is perpendicular to z and up
        let x = up.cross(&z).normalize();
        debug_assert!(
            !x[0].is_nan() && !x[1].is_nan() && !x[2].is_nan(),
            "direction and up vectors are parallel"
        );

        // direction of y is then perpendicular to x and z
        let y = z.cross(&x);

        self.inverse_view_matrix = Rotation3::from_basis_unchecked(&[x, y, z]);
    }
}

impl Camera for EquirectangularCamera {
    fn resolution(&self) -> Vector2<u32> {
        self.resolution
    }

    fn pixel_to_rain_angle(&self, pixel: Vector2<u32>) -> RainAngle {
        // local coordinates
        let theta = PI * pixel.y as f64 / self.resolution.y as f64;
        let phi = PI * pixel.x as f64 / self.resolution.y as f64;
        let local_dir = RainAngle::new(theta, phi).to_vector();

        // transform to global
        let dir = self.inverse_view_matrix.transform_vector(&local_dir);

        RainAngle::from_vector(dir)
    }
}

#[derive(Clone, Copy)]
pub struct PerspectiveCamera {
    // resolution of the camera
    pub resolution: Vector2<u32>,
    // vertical field of view in radians
    pub fov: f64,
    // view matrix for transforming from local space to world space
    // column vectors are right, up, facing in global space
    pub inverse_view_matrix: Rotation3<f64>,
}

impl Default for PerspectiveCamera {
    fn default() -> Self {
        let mut cam = PerspectiveCamera::new(
            Vector2::new(128, 128),
            60_f64.to_radians(),
            Rotation3::default(),
        );
        cam.look_at(
            &Vector3::new(0_f64, 0_f64, 1_f64),
            &Vector3::new(0_f64, 1_f64, 0_f64),
        );
        cam
    }
}

impl PerspectiveCamera {
    pub fn new(resolution: Vector2<u32>, fov: f64, inverse_view_matrix: Rotation3<f64>) -> Self {
        PerspectiveCamera {
            resolution,
            fov,
            inverse_view_matrix,
        }
    }

    // sets the focal length in pixels
    pub fn set_focal_length(&mut self, focal_length: f64) {
        self.fov = 2_f64 * (self.resolution.y as f64 / (2_f64 * focal_length)).atan()
    }

    // gets the vertical fov in pixels
    pub fn get_focal_length(&self) -> f64 {
        self.resolution.y as f64 / (2_f64 * (self.fov / 2_f64).tan())
    }

    // makes the camera look at dir
    pub fn look_at(&mut self, dir: &Vector3<f64>, up: &Vector3<f64>) {
        // direction of z axis is -view direction
        let z = -dir.normalize();

        // direction of x axis is perpendicular to z and up
        let x = up.cross(&z).normalize();
        debug_assert!(
            !x[0].is_nan() && !x[1].is_nan() && !x[2].is_nan(),
            "direction and up vectors are parallel"
        );

        // direction of y is then perpendicular to x and z
        let y = z.cross(&x);

        self.inverse_view_matrix = Rotation3::from_basis_unchecked(&[x, y, z]);
    }

    pub fn right(&self) -> Vector3<f64> {
        self.inverse_view_matrix.matrix().column(0).into()
    }

    pub fn up(&self) -> Vector3<f64> {
        self.inverse_view_matrix.matrix().column(1).into()
    }

    pub fn facing(&self) -> Vector3<f64> {
        self.inverse_view_matrix.matrix().column(2).into()
    }
}

impl Camera for PerspectiveCamera {
    fn resolution(&self) -> Vector2<u32> {
        self.resolution
    }

    fn pixel_to_rain_angle(&self, pixel: Vector2<u32>) -> RainAngle {
        // local coordinates
        let x = pixel.x as f64 - self.resolution.x as f64 / 2_f64;
        let y = self.resolution.y as f64 / 2_f64 - pixel.y as f64;
        let z = -self.get_focal_length();

        // transform to global
        let dir = self
            .inverse_view_matrix
            .transform_vector(&Vector3::new(x, y, z));

        RainAngle::from_vector(dir)
    }
}
