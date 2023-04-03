use crate::{
    environment::{Environment, Image},
    spherical_angle::{RainAngle, SphericalAngle},
};
use cgmath::{vec3, Quaternion, Rotation, Vector2};
use image::{ImageBuffer, Pixel, Rgb};
use rayon::prelude::{ParallelBridge, ParallelIterator};
use std::f64::consts::PI;

pub trait Camera: Sync {
    fn resolution(&self) -> Vector2<u32>;

    fn pixel_to_rain_angle(&self, pixel: Vector2<u32>) -> RainAngle;

    fn position(&self) -> f64;

    /// Renders an image with GR
    fn render(&self, env: &impl Environment) -> Image {
        let res = self.resolution();
        let pos = self.position();

        // Create the image buffer
        let mut buf: Image = ImageBuffer::new(res.x, res.y);

        // Calculate pixels in parallel
        buf.enumerate_pixels_mut()
            .par_bridge()
            .for_each(|(x, y, pixel)| {
                let rain_angle = self.pixel_to_rain_angle(Vector2::new(x, y));

                if let Some(map_angle) = rain_angle.try_to_map_angle(pos) {
                    // Successful map angle
                    *pixel = env.get_pixel(map_angle)
                } else {
                    // Ray went into black hole
                    *pixel = *Rgb::from_slice(&[0, 0, 0])
                }
            });

        buf
    }

    /// Render the image without GR but still shows the black hole shadow
    fn render_no_gr(&self, env: &impl Environment) -> Image {
        let res = self.resolution();
        let pos = self.position();

        // Create the image buffer
        let mut buf: Image = ImageBuffer::new(res.x, res.y);

        // Calculate pixels in parallel
        buf.enumerate_pixels_mut()
            .par_bridge()
            .for_each(|(x, y, pixel)| {
                let rain_angle = self.pixel_to_rain_angle(Vector2::new(x, y));

                if let Some(map_angle) = rain_angle.try_to_map_angle_no_gr(pos) {
                    // Successful map angle
                    *pixel = env.get_pixel(map_angle)
                } else {
                    // Ray went into black hole
                    *pixel = *Rgb::from_slice(&[0, 0, 0])
                }
            });

        buf
    }
}

pub struct EquirectangularCamera {
    position: f64,
    resolution: Vector2<u32>,
    facing: RainAngle,
}

impl EquirectangularCamera {
    pub fn new(position: f64, resolution: Vector2<u32>) -> Self {
        EquirectangularCamera {
            position,
            resolution,
            facing: RainAngle::new(PI / 2_f64, PI),
        }
    }

    pub fn update_position(&mut self, position: f64) {
        self.position = position
    }

    pub fn face(&mut self, angle: RainAngle) {
        self.facing = angle
    }

    fn pixel_to_local_angle(&self, pixel: Vector2<u32>) -> RainAngle {
        let theta = PI * pixel.y as f64 / self.resolution.y as f64;
        let phi = PI * pixel.x as f64 / self.resolution.y as f64;

        RainAngle::new(theta, phi)
    }
}

impl Camera for EquirectangularCamera {
    fn position(&self) -> f64 {
        self.position
    }

    fn resolution(&self) -> Vector2<u32> {
        self.resolution
    }

    fn pixel_to_rain_angle(&self, pixel: Vector2<u32>) -> RainAngle {
        let mat = Quaternion::look_at(self.facing.to_vector(), vec3(0_f64, 0_f64, 1_f64));
        let local_angle = self.pixel_to_local_angle(pixel);
        let new_angle = local_angle.rotate(mat);

        if new_angle.theta.is_nan() || new_angle.phi.is_nan() {
            return local_angle;
        } else {
            return new_angle;
        }
    }
}

#[derive(Clone, Copy)]
pub struct PerspectiveCamera {
    pub position: f64,
    pub resolution: Vector2<u32>,
    pub fov: f64,
    pub facing: RainAngle,
}

impl Default for PerspectiveCamera {
    fn default() -> Self {
        PerspectiveCamera::new(10_f64, Vector2 { x: 1024, y: 1024 }, 60_f64 * PI / 180_f64)
    }
}

impl PerspectiveCamera {
    pub fn new(position: f64, resolution: Vector2<u32>, fov: f64) -> Self {
        PerspectiveCamera {
            position,
            resolution,
            fov,
            facing: Default::default(),
        }
    }

    pub fn update_position(&mut self, position: f64) {
        self.position = position
    }

    pub fn face(&mut self, angle: RainAngle) {
        self.facing = angle
    }

    // TODO: Make projection not look so weird?
    fn pixel_to_local_angle(&self, pixel: Vector2<u32>) -> RainAngle {
        let theta = ((self.fov / 2_f64).tan()
            * ((pixel.x as f64 - self.resolution.x as f64 / 2_f64).powi(2)
                + (pixel.y as f64 - self.resolution.y as f64 / 2_f64).powi(2))
            .sqrt())
        .atan2(self.resolution.y as f64 / 2_f64);

        let mut phi = f64::atan2(
            self.resolution.y as f64 / 2_f64 - pixel.y as f64,
            pixel.x as f64 - self.resolution.x as f64 / 2_f64,
        );

        // Map phi back to 0 <= phi < 2pi because this can cause problems later
        if phi < 0_f64 {
            phi += 2_f64 * PI;
        }

        RainAngle::new(theta, phi)
    }
}

impl Camera for PerspectiveCamera {
    fn position(&self) -> f64 {
        self.position
    }

    fn resolution(&self) -> Vector2<u32> {
        self.resolution
    }

    fn pixel_to_rain_angle(&self, pixel: Vector2<u32>) -> RainAngle {
        let mat = Quaternion::look_at(self.facing.to_vector(), vec3(0_f64, 0_f64, 1_f64));
        let local_angle = self.pixel_to_local_angle(pixel);
        let new_angle = local_angle.rotate(mat);

        if new_angle.theta.is_nan() || new_angle.phi.is_nan() {
            return local_angle;
        } else {
            return new_angle;
        }
    }
}
