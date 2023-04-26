use crate::{
    camera::{Camera, Projection},
    diver::Diver,
    environment::Environment,
    traits::Interpolate,
};
use image::{ImageBuffer, Pixel, Rgb, RgbImage};
use nalgebra::Vector2;
use rayon::prelude::{ParallelBridge, ParallelIterator};
use std::sync::Arc;

#[derive(PartialEq)]
pub struct Scene {
    pub camera: Camera,
    pub env: Arc<Environment>,
    pub diver: Diver,
    pub gr: bool,
}

impl Clone for Scene {
    fn clone(&self) -> Self {
        Self::new(self.camera, self.env.clone(), self.diver, self.gr)
    }
}

impl Scene {
    pub fn new(camera: Camera, env: Arc<Environment>, diver: Diver, gr: bool) -> Scene {
        Self {
            camera,
            env,
            diver,
            gr,
        }
    }

    pub fn render(&self, projection: Projection, resolution: Vector2<u32>) -> RgbImage {
        // Create the image buffer
        let mut buf: RgbImage = ImageBuffer::new(resolution.x, resolution.y);

        // Calculate pixels in parallel
        if self.gr {
            buf.enumerate_pixels_mut()
                .par_bridge()
                .for_each(|(x, y, pixel)| {
                    let rain_angle =
                        self.camera
                            .pixel_to_rain_angle(projection, Vector2::new(x, y), resolution);

                    if let Some(map_angle) = rain_angle.to_map_angle(self.diver.position()) {
                        // Successful map angle
                        *pixel = self.env.get_pixel(map_angle)
                    } else {
                        // Ray went into black hole
                        *pixel = *Rgb::from_slice(&[0, 0, 0])
                    }
                });
        } else {
            // Calculate pixels in parallel
            buf.enumerate_pixels_mut()
                .par_bridge()
                .for_each(|(x, y, pixel)| {
                    let rain_angle =
                        self.camera
                            .pixel_to_rain_angle(projection, Vector2::new(x, y), resolution);

                    if let Some(map_angle) =
                        rain_angle.try_to_map_angle_no_gr(self.diver.position())
                    {
                        // Successful map angle
                        *pixel = self.env.get_pixel(map_angle)
                    } else {
                        // Ray went into black hole
                        *pixel = *Rgb::from_slice(&[0, 0, 0])
                    }
                });
        }

        buf
    }
}

impl Interpolate for Scene {
    fn interpolate(&self, other: &Scene, factor: f32) -> Scene {
        let camera = self.camera.interpolate(&other.camera, factor);
        Scene::new(
            camera,
            self.env.clone(),
            self.diver.interpolate(&other.diver, factor),
            self.gr,
        )
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            camera: Default::default(),
            env: Arc::new(Default::default()),
            diver: Default::default(),
            gr: true,
        }
    }
}
