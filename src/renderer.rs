use crate::{
    camera::Camera,
    environment::{Environment, Image},
};
use image::{ImageBuffer, Pixel, Rgb};
use nalgebra::Vector2;
use rayon::prelude::{ParallelBridge, ParallelIterator};

pub fn render<C: Camera, E: Environment>(
    camera: &C,
    environment: &E,
    radius: f64,
    gr: bool,
) -> Image {
    let res = camera.resolution();

    // Create the image buffer
    let mut buf: Image = ImageBuffer::new(res.x, res.y);

    // Calculate pixels in parallel
    if gr {
        buf.enumerate_pixels_mut()
            .par_bridge()
            .for_each(|(x, y, pixel)| {
                let rain_angle = camera.pixel_to_rain_angle(Vector2::new(x, y));

                if let Some(map_angle) = rain_angle.to_map_angle(radius) {
                    // Successful map angle
                    *pixel = environment.get_pixel(map_angle)
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
                let rain_angle = camera.pixel_to_rain_angle(Vector2::new(x, y));

                if let Some(map_angle) = rain_angle.try_to_map_angle_no_gr(radius) {
                    // Successful map angle
                    *pixel = environment.get_pixel(map_angle)
                } else {
                    // Ray went into black hole
                    *pixel = *Rgb::from_slice(&[0, 0, 0])
                }
            });
    }

    buf
}
