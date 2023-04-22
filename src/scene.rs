use std::sync::Arc;

use crate::{
    camera::{Camera, EquirectangularCamera, PerspectiveCamera},
    diver::Diver,
    environment::Environment,
};
use image::{ImageBuffer, Pixel, Rgb, RgbImage};
use nalgebra::Vector2;
use rayon::prelude::{ParallelBridge, ParallelIterator};

#[derive(PartialEq)]
pub struct Scene<C: Camera, E: Environment> {
    pub camera: C,
    pub env: Arc<E>,
    pub diver: Diver,
    pub gr: bool,
}

impl<E: Environment> From<Scene<PerspectiveCamera, E>> for Scene<EquirectangularCamera, E> {
    fn from(scene: Scene<PerspectiveCamera, E>) -> Self {
        Self {
            camera: scene.camera.into(),
            env: scene.env,
            diver: scene.diver,
            gr: scene.gr,
        }
    }
}

impl<C: Camera, E: Environment> Clone for Scene<C, E> {
    fn clone(&self) -> Self {
        Self::new(self.camera.clone(), self.env.clone(), self.diver, self.gr)
    }
}

impl<C: Camera, E: Environment> Scene<C, E> {
    pub fn new(camera: C, env: Arc<E>, diver: Diver, gr: bool) -> Scene<C, E> {
        Self {
            camera,
            env,
            diver,
            gr,
        }
    }

    pub fn render(&self) -> RgbImage {
        let res = self.camera.resolution();

        // Create the image buffer
        let mut buf: RgbImage = ImageBuffer::new(res.x, res.y);

        // Calculate pixels in parallel
        if self.gr {
            buf.enumerate_pixels_mut()
                .par_bridge()
                .for_each(|(x, y, pixel)| {
                    let rain_angle = self.camera.pixel_to_rain_angle(Vector2::new(x, y));

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
                    let rain_angle = self.camera.pixel_to_rain_angle(Vector2::new(x, y));

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

impl<C: Camera, E: Environment> Default for Scene<C, E> {
    fn default() -> Self {
        Self {
            camera: Default::default(),
            env: Arc::new(Default::default()),
            diver: Default::default(),
            gr: true,
        }
    }
}
