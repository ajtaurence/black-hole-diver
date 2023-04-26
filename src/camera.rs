use nalgebra::{Rotation3, Vector2, Vector3};
use std::f64::consts::PI;

use crate::{
    spherical_angle::{RainAngle, SphericalAngle},
    traits::Interpolate,
};

// Camera is assumed to be at the origin
pub trait Camera: Interpolate + Default + Clone + Sync + 'static {
    // returns the ray direction through this pixel
    fn pixel_to_rain_angle(&self, pixel: Vector2<u32>, resolution: Vector2<u32>) -> RainAngle;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Cameras {
    #[default]
    Perspective,
    Equirectangular,
}

impl ToString for Cameras {
    fn to_string(&self) -> String {
        match self {
            Cameras::Perspective => "Perspective".to_owned(),
            Cameras::Equirectangular => "360°".to_owned(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct EquirectangularCamera {
    // view matrix for transforming from local space to world space
    // column vectors are right, up, facing in global space
    pub inverse_view_matrix: Rotation3<f64>,
}

impl From<PerspectiveCamera> for EquirectangularCamera {
    fn from(camera: PerspectiveCamera) -> Self {
        EquirectangularCamera {
            // rotate the perspective view matrix so that it makes the forward vector point toward the center in equirectangular projection
            inverse_view_matrix: camera.inverse_view_matrix
                * Rotation3::from_scaled_axis(Vector3::new(0_f64, 0_f64, PI / 2_f64))
                * Rotation3::from_scaled_axis(Vector3::new(0_f64, -PI / 2_f64, 0_f64)),
        }
    }
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
    pub fn new(vertical_resolution: u32, inverse_view_matrix: Rotation3<f64>) -> Self {
        EquirectangularCamera {
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

impl Camera for EquirectangularCamera {
    fn pixel_to_rain_angle(&self, pixel: Vector2<u32>, resolution: Vector2<u32>) -> RainAngle {
        // local coordinates
        let theta = PI * (1_f64 - pixel.y as f64 / resolution.y as f64);
        let phi = PI * pixel.x as f64 / resolution.y as f64;
        let local_dir = RainAngle::new(theta, phi).to_vector();

        // transform to global
        let dir = self.inverse_view_matrix.transform_vector(&local_dir);

        RainAngle::from_vector(dir)
    }
}

impl Interpolate for EquirectangularCamera {
    fn interpolate(&self, other: &Self, factor: f32) -> Self {
        EquirectangularCamera {
            inverse_view_matrix: self
                .inverse_view_matrix
                .slerp(&other.inverse_view_matrix, factor as f64),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct PerspectiveCamera {
    // vertical field of view in radians
    pub fov: f64,
    // view matrix for transforming from local space to world space
    // column vectors are right, up, facing in global space
    pub inverse_view_matrix: Rotation3<f64>,
}

impl Interpolate for PerspectiveCamera {
    fn interpolate(&self, other: &Self, factor: f32) -> Self {
        // todo: avoid panic when angles are 180 degrees apart
        let inverse_view_matrix = self
            .inverse_view_matrix
            .slerp(&other.inverse_view_matrix, factor as f64);

        Self {
            fov: self.fov.interpolate(&other.fov, factor),
            inverse_view_matrix,
        }
    }
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
            fov,
            inverse_view_matrix,
        }
    }

    // adjusts the pitch of the camera by the angle
    pub fn pitch(&mut self, angle: f64) {
        self.inverse_view_matrix =
            Rotation3::from_scaled_axis(self.right() * angle) * self.inverse_view_matrix;
    }

    // adjusts the yaw of the camera by the angle
    pub fn yaw(&mut self, angle: f64) {
        self.inverse_view_matrix = Rotation3::from_scaled_axis(
            Vector3::new(0_f64, self.up()[1], 0_f64).normalize() * angle,
        ) * self.inverse_view_matrix;
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

    pub fn drag_delta(&mut self, delta: egui::Vec2, sensitivity: f64) {
        self.pitch(-delta.y as f64 * self.fov * 0.0005 * sensitivity);
        self.yaw(-delta.x as f64 * self.fov * 0.0005 * sensitivity)
    }

    pub fn zoom(&mut self, scroll: f32, sensitivity: f64) {
        self.fov = (self.fov * 2_f64.powf(-scroll as f64 * 0.0005 * sensitivity)).clamp(0_f64, PI);
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
    fn pixel_to_rain_angle(&self, pixel: Vector2<u32>, resolution: Vector2<u32>) -> RainAngle {
        // local coordinates
        let x = pixel.x as f64 - resolution.x as f64 / 2_f64;
        let y = resolution.y as f64 / 2_f64 - pixel.y as f64;
        let z = -(resolution.y as f64) / (2_f64 * (self.fov / 2_f64).tan());

        // transform to global
        let dir = self
            .inverse_view_matrix
            .transform_vector(&Vector3::new(x, y, z));

        RainAngle::from_vector(dir)
    }
}
