use std::f64::consts::PI;

use bh_diver::{
    camera::{Camera, PerspectiveCamera},
    environment::ImageEnvironment,
};
use cgmath::Vector2;
use eframe::egui;
use egui::{ColorImage, DragValue, Slider};

struct MyApp {
    environment: ImageEnvironment,
    res_scale: f32,
    camera: PerspectiveCamera,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            // Initialize environments
            environment: ImageEnvironment::new(
                image::load_from_memory(include_bytes!("../sky.tif"))
                    .unwrap()
                    .into_rgb8(),
            )
            .unwrap(),
            res_scale: 0.5,
            camera: Default::default(),
        }
    }
}

impl MyApp {
    fn new(__cc: &eframe::CreationContext) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        Self::default()
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::SidePanel::new(egui::panel::Side::Left, "left pannel").show(ctx, |ui| {
            ui.heading("Settings");

            ui.horizontal(|ui| {
                ui.label("Distance");
                ui.add(
                    DragValue::new(&mut self.camera.position)
                        .clamp_range(0_f64..=f64::MAX)
                        .speed(0.5),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Resolution scale");
                ui.add(
                    DragValue::new(&mut self.res_scale)
                        .clamp_range(0_f32..=2_f32)
                        .speed(0.1),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Field of view");
                ui.add(
                    Slider::new(&mut self.camera.fov, 0_f64..=PI)
                        .fixed_decimals(2)
                        .suffix(" rad"),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Theta");
                ui.add(
                    Slider::new(&mut self.camera.facing.theta, 0_f64..=PI)
                        .fixed_decimals(2)
                        .suffix(" rad"),
                );
            });
            ui.horizontal(|ui| {
                ui.label("phi");
                ui.add(
                    Slider::new(&mut self.camera.facing.phi, 0_f64..=2_f64 * PI)
                        .fixed_decimals(2)
                        .suffix(" rad"),
                );
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // get pixels per egui point
            let pixelsperpoint = frame.info().native_pixels_per_point.unwrap();

            // update the camera resolution
            let space = ui.available_size();
            let res = space * pixelsperpoint * self.res_scale;
            self.camera.resolution = Vector2::new(res.x as u32, res.y as u32);

            // render the new image
            let img = self.camera.render(&self.environment);

            // get a new egui texture handle
            let texture: &egui::TextureHandle = &ctx.load_texture(
                "render texture",
                ColorImage::from_rgb([res.x as _, res.y as _], img.as_flat_samples().as_slice()),
                Default::default(),
            );

            // show the image
            ui.image(texture, space);
        });
    }
}

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Black Hole Raytracer",
        options,
        Box::new(|cc| Box::new(MyApp::new(cc))),
    )
    .unwrap();
}
