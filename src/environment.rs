use crate::spherical_angle::{MapAngle, SphericalAngle};
use image::{ImageBuffer, Pixel, Rgb};
use noise::NoiseFn;
use std::f64::consts::PI;

pub type Image = ImageBuffer<Rgb<u8>, Vec<u8>>;

#[derive(Debug)]
pub enum EnvironmentError {
    NotEquirectangularImage,
}

pub trait Environment: Send + Sync {
    fn get_pixel(&self, angle: MapAngle) -> Rgb<u8>;
}

pub struct ImageEnvironment {
    image: Image,
}

impl ImageEnvironment {
    pub fn new(image: impl Into<Image>) -> Result<Self, EnvironmentError> {
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

pub struct ProceeduralEnvironment<N>
where
    N: NoiseFn<f64, 3>,
{
    noise: N,
    pub scale: f64,
}

impl<N> ProceeduralEnvironment<N>
where
    N: NoiseFn<f64, 3>,
{
    pub fn new(noise: N, scale: f64) -> Self {
        Self { noise, scale }
    }
}

impl<N> Environment for ProceeduralEnvironment<N>
where
    N: NoiseFn<f64, 3> + Send + Sync,
{
    fn get_pixel(&self, angle: MapAngle) -> Rgb<u8> {
        let r = 1_f64 / self.scale;

        let point = [
            r * angle.theta().sin() * angle.phi().cos(),
            r * angle.theta().sin() * angle.phi().sin(),
            r * angle.theta().cos(),
        ];

        let subpixel_value = (self.noise.get(point) * u8::MAX as f64) as u8;
        *Pixel::from_slice(&[subpixel_value, subpixel_value, subpixel_value])
    }
}
