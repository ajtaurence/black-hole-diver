use bh_diver::{
    diver::map_coords_from_rain_coords,
    image::{equirectangular_to_spherical, get_star_map, spherical_to_equirectangular},
};
use image::{ImageBuffer, Rgb};

fn main() {
    let img = get_star_map("sky2.jpg").unwrap().to_rgb8();

    let r = 10.;
    let m = 1.;

    let new_img = ImageBuffer::from_fn(img.width(), img.height(), |x, y| {
        let rain_coords = equirectangular_to_spherical((x, y), img.height());

        let map_coords = map_coords_from_rain_coords(rain_coords, r, m);

        match map_coords {
            None => Rgb::<u8>([0, 0, 0]),
            Some(map_coords) => img[spherical_to_equirectangular(map_coords, img.height())],
        }
    });

    new_img.save("new_img.jpg").unwrap();
}
