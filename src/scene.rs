use crate::{
    camera::Camera, diver::Diver, environment::Environment, render::RenderSettings,
    traits::Interpolate,
};
use image::{ImageBuffer, Pixel, Rgb, RgbImage};
use nalgebra::Vector2;
use rayon::prelude::{ParallelBridge, ParallelIterator};

#[derive(Clone, PartialEq)]
pub struct Scene {
    pub camera: Camera,
    pub env: Environment,
    pub diver: Diver,
    pub gr: bool,
}

impl Scene {
    pub fn new(camera: Camera, env: Environment, diver: Diver, gr: bool) -> Scene {
        Self {
            camera,
            env,
            diver,
            gr,
        }
    }

    pub fn render(&self, render_settings: RenderSettings) -> RgbImage {
        let super_sampling_bool = render_settings.super_sampling.is_some();
        let super_sampling = render_settings.super_sampling.unwrap_or(1);

        let resolution = Vector2::new(
            render_settings.resolution.x * super_sampling as u32,
            render_settings.resolution.y * super_sampling as u32,
        );

        // Create the image buffer
        let mut buf: RgbImage = ImageBuffer::new(resolution.x, resolution.y);

        // Calculate pixels in parallel
        if self.gr {
            buf.enumerate_pixels_mut()
                .par_bridge()
                .for_each(|(x, y, pixel)| {
                    let rain_angle = self.camera.pixel_to_rain_angle(
                        render_settings.projection,
                        Vector2::new(x, y),
                        resolution,
                    );

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
                    let rain_angle = self.camera.pixel_to_rain_angle(
                        render_settings.projection,
                        Vector2::new(x, y),
                        resolution,
                    );

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

        // downscale the image if needed
        if super_sampling_bool {
            return image::imageops::resize(
                &buf,
                render_settings.resolution.x,
                render_settings.resolution.y,
                image::imageops::FilterType::Lanczos3,
            );
        } else {
            return buf;
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Camera", |ui| {
            self.camera.show(ui);
        });
        ui.collapsing("Diver", |ui| {
            self.diver.show(ui);
        });
        ui.checkbox(&mut self.gr, "General Relativity");
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
            env: Default::default(),
            diver: Default::default(),
            gr: true,
        }
    }
}
