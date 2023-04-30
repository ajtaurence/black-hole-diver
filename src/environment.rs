use crate::spherical_angle::{MapAngle, SphericalAngle};
use image::{Rgb, RgbImage};
use std::{f64::consts::PI, sync::Arc};

#[derive(Debug, Clone, Copy)]
pub enum EnvironmentError {
    NotEquirectangularImage,
}

#[derive(Clone, PartialEq)]
pub struct Environment {
    image: Arc<RgbImage>,
}

impl Default for Environment {
    fn default() -> Self {
        Environment::new(
            image::load_from_memory(include_bytes!("../sky.tif"))
                .unwrap()
                .into_rgb8(),
        )
        .unwrap()
    }
}

impl Environment {
    pub fn new(image: impl Into<RgbImage>) -> Result<Self, EnvironmentError> {
        let image = image.into();

        if image.width() == 2 * image.height() {
            return Ok(Environment {
                image: Arc::new(image),
            });
        } else {
            return Err(EnvironmentError::NotEquirectangularImage);
        }
    }

    pub fn get_pixel(&self, angle: MapAngle) -> Rgb<u8> {
        let x = (self.image.height() as f64 * angle.phi() / PI).floor() as u32;
        let y = (self.image.height() as f64 * angle.theta() / PI).floor() as u32;
        *self.image.get_pixel(
            x.min(self.image.width() - 1),
            y.min(self.image.height() - 1),
        )
    }
}
