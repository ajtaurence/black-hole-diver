use image::{GenericImageView, Pixel, Primitive};
use noise::NoiseFn;
use num_traits::AsPrimitive;
use std::f64::consts::PI;

use crate::spherical_angle::{MapAngle, SphericalAngle};

#[derive(Debug)]
pub enum EnvironmentError {
    NotEquirectangularImage,
}

pub trait Environment<P>: Send + Sync
where
    P: Pixel,
{
    fn get_pixel(&self, angle: MapAngle) -> P;
}

pub struct ImageEnvironment<I>
where
    I: GenericImageView,
{
    image: I,
}

impl<I> ImageEnvironment<I>
where
    I: GenericImageView,
{
    pub fn new(image: I) -> Result<Self, EnvironmentError> {
        if image.width() == 2 * image.height() {
            return Ok(ImageEnvironment { image });
        } else {
            return Err(EnvironmentError::NotEquirectangularImage);
        }
    }
}

impl<P, I> Environment<P> for ImageEnvironment<I>
where
    P: Pixel,
    I: GenericImageView<Pixel = P> + Send + Sync,
{
    fn get_pixel(&self, angle: MapAngle) -> I::Pixel {
        let x = (self.image.height() as f64 * angle.phi() / PI).floor() as u32;
        let y = (self.image.height() as f64 * angle.theta() / PI).floor() as u32;
        self.image.get_pixel(
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

impl<P, N> Environment<P> for ProceeduralEnvironment<N>
where
    P: Pixel,
    f64: AsPrimitive<P::Subpixel>,
    P::Subpixel: AsPrimitive<f64>,
    N: NoiseFn<f64, 3> + Send + Sync,
{
    fn get_pixel(&self, angle: MapAngle) -> P {
        let r = 1_f64 / self.scale;

        let point = [
            r * angle.theta().sin() * angle.phi().cos(),
            r * angle.theta().sin() * angle.phi().sin(),
            r * angle.theta().cos(),
        ];

        let max = <P::Subpixel as Primitive>::DEFAULT_MAX_VALUE;

        let subpixel_value = (self.noise.get(point) * max.as_()).as_();
        *P::from_slice(&vec![subpixel_value; P::CHANNEL_COUNT as usize].into_boxed_slice())
    }
}
