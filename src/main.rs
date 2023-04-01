use bh_diver::{camera::Camera, environment::ImageEnvironment};

fn main() {
    // TODO: Fix the direction that the camera rotates to make it easier to use
    // Add rotation control to the environment

    let cam = Camera::new(2_f64, (1920, 1080), 45.);

    let map = image::io::Reader::open("sky.tif")
        .unwrap()
        .decode()
        .unwrap();

    let env = ImageEnvironment::new(map).unwrap();

    let img = cam.render(&env);

    img.save("img.jpg").unwrap();
}
