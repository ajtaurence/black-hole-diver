use crate::spherical_angle::{MapAngle, SphericalAngle};
use image::{Rgb, RgbImage};
use std::f64::consts::PI;

#[derive(Debug)]
pub enum EnvironmentError {
    NotEquirectangularImage,
}

pub trait Environment: Default + Send + Sync {
    fn get_pixel(&self, angle: MapAngle) -> Rgb<u8>;
}

#[derive(PartialEq)]
pub struct ImageEnvironment {
    image: RgbImage,
}

impl Default for ImageEnvironment {
    fn default() -> Self {
        ImageEnvironment::new(
            image::load_from_memory(include_bytes!("../sky.tif"))
                .unwrap()
                .into_rgb8(),
        )
        .unwrap()
    }
}

impl ImageEnvironment {
    pub fn new(image: impl Into<RgbImage>) -> Result<Self, EnvironmentError> {
        let image = image.into();

        if image.width() == 2 * image.height() {
            return Ok(ImageEnvironment { image });
        } else {
            return Err(EnvironmentError::NotEquirectangularImage);
        }
    }
}

impl Environment for ImageEnvironment {
    fn get_pixel(&self, angle: MapAngle) -> Rgb<u8> {
        let x = (self.image.height() as f64 * angle.phi() / PI).floor() as u32;
        let y = (self.image.height() as f64 * angle.theta() / PI).floor() as u32;
        *self.image.get_pixel(
            x.min(self.image.width() - 1),
            y.min(self.image.height() - 1),
        )
    }
}
