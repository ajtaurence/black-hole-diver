use std::f64::consts::PI;

use image::{io::Reader as ImageReader, DynamicImage, ImageError};

pub fn pixel_index_to_coord(index: u32, height: u32) -> (u32, u32) {
    (index % (2 * height as u32), index / height as u32)
}

pub fn spherical_to_equirectangular(coord: (f64, f64), height: u32) -> (u32, u32) {
    let (theta, phi) = coord;
    let x = (height as f64 * phi / PI).floor() as u32;
    let y = (height as f64 * theta / PI).floor() as u32;
    (x, y)
}

pub fn equirectangular_to_spherical(coord: (u32, u32), height: u32) -> (f64, f64) {
    let (x, y) = coord;
    let theta = PI * y as f64 / height as f64;
    let phi = PI * x as f64 / height as f64;
    (theta, phi)
}

pub fn get_star_map() -> Result<DynamicImage, ImageError> {
    ImageReader::open("sky.tif")?.decode()
}

#[test]
fn test() {
    println!("{:?}", equirectangular_to_spherical((100, 511), 512))
}
