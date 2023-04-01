use crate::{
    environment::Environment,
    spherical_angle::{RainAngle, SphericalAngle},
};
use cgmath::{vec3, Quaternion, Rotation};
use image::{ImageBuffer, Pixel, Primitive};
use rayon::prelude::{ParallelBridge, ParallelIterator};
use std::f64::consts::PI;

pub struct EquirectangularCamera {
    position: f64,
    vertical_resolution: u32,
    facing: RainAngle,
}

impl EquirectangularCamera {
    pub fn new(position: f64, vertical_resolution: u32) -> Self {
        EquirectangularCamera {
            position,
            vertical_resolution,
            facing: RainAngle::new(PI / 2_f64, PI),
        }
    }

    pub fn update_position(&mut self, position: f64) {
        self.position = position
    }

    pub fn face(&mut self, angle: RainAngle) {
        self.facing = angle
    }

    pub fn render<P: Pixel + Send + Sync>(
        &self,
        env: &impl Environment<P>,
    ) -> ImageBuffer<P, Vec<P::Subpixel>>
    where
        P::Subpixel: Send + Sync,
    {
        // Create the image buffer
        let mut buf: ImageBuffer<P, Vec<P::Subpixel>> =
            ImageBuffer::new(2 * self.vertical_resolution, self.vertical_resolution);

        // Calculate pixels in parallel
        buf.enumerate_pixels_mut()
            .par_bridge()
            .for_each(|(x, y, pixel)| {
                let rain_angle = self.pixel_to_rain_angle(x, y);
                if let Some(map_angle) = rain_angle.try_to_map_angle(self.position) {
                    *pixel = env.get_pixel(map_angle)
                } else {
                    *pixel = *P::from_slice(
                        &vec![
                            <P::Subpixel as Primitive>::DEFAULT_MIN_VALUE;
                            P::CHANNEL_COUNT as usize
                        ]
                        .into_boxed_slice(),
                    )
                }
            });

        buf
    }

    fn pixel_to_local_angle(&self, x: u32, y: u32) -> RainAngle {
        let theta = PI * y as f64 / self.vertical_resolution as f64;
        let phi = PI * x as f64 / self.vertical_resolution as f64;

        RainAngle::new(theta, phi)
    }

    fn pixel_to_rain_angle(&self, x: u32, y: u32) -> RainAngle {
        let mat = Quaternion::look_at(self.facing.to_vector(), vec3(0_f64, 0_f64, 1_f64));
        let local_angle = self.pixel_to_local_angle(x, y);
        let new_angle = local_angle.rotate(mat);

        if new_angle.theta.is_nan() || new_angle.phi.is_nan() {
            return local_angle;
        } else {
            return new_angle;
        }
    }
}

pub struct Camera {
    position: f64,
    resolution: (u32, u32),
    fov: f64,
    facing: RainAngle,
}

impl Camera {
    pub fn new(position: f64, resolution: (u32, u32), fov: f64) -> Self {
        Camera {
            position,
            resolution,
            fov: fov * PI / 180_f64,
            facing: Default::default(),
        }
    }

    pub fn update_position(&mut self, position: f64) {
        self.position = position
    }

    pub fn face(&mut self, angle: RainAngle) {
        self.facing = angle
    }

    pub fn render<P: Pixel + Send + Sync>(
        &self,
        env: &impl Environment<P>,
    ) -> ImageBuffer<P, Vec<P::Subpixel>>
    where
        P::Subpixel: Send + Sync,
    {
        // Create the image buffer
        let mut buf: ImageBuffer<P, Vec<P::Subpixel>> =
            ImageBuffer::new(self.resolution.0, self.resolution.1);

        // Calculate pixels in parallel
        buf.enumerate_pixels_mut()
            .par_bridge()
            .for_each(|(x, y, pixel)| {
                let rain_angle = self.pixel_to_rain_angle(x, y);
                if let Some(map_angle) = rain_angle.try_to_map_angle(self.position) {
                    *pixel = env.get_pixel(map_angle)
                } else {
                    *pixel = *P::from_slice(
                        &vec![
                            <P::Subpixel as Primitive>::DEFAULT_MIN_VALUE;
                            P::CHANNEL_COUNT as usize
                        ]
                        .into_boxed_slice(),
                    )
                }
            });

        buf
    }

    // TODO: Make projection not look so weird
    fn pixel_to_local_angle(&self, x: u32, y: u32) -> RainAngle {
        let theta = (self.fov
            * ((x as f64 - self.resolution.0 as f64 / 2_f64).powi(2)
                + (y as f64 - self.resolution.1 as f64 / 2_f64).powi(2))
            .sqrt())
        .atan2(self.resolution.1 as f64 / 2_f64);

        let mut phi = f64::atan2(
            self.resolution.1 as f64 / 2_f64 - y as f64,
            x as f64 - self.resolution.0 as f64 / 2_f64,
        );

        // Map phi back to 0 <= phi < 2pi because this can cause problems later
        if phi < 0_f64 {
            phi += 2_f64 * PI;
        }

        RainAngle::new(theta, phi)
    }

    fn pixel_to_rain_angle(&self, x: u32, y: u32) -> RainAngle {
        let mat = Quaternion::look_at(self.facing.to_vector(), vec3(0_f64, 0_f64, 1_f64));
        let local_angle = self.pixel_to_local_angle(x, y);
        let new_angle = local_angle.rotate(mat);

        if new_angle.theta.is_nan() || new_angle.phi.is_nan() {
            return local_angle;
        } else {
            return new_angle;
        }
    }
}
