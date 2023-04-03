use bh_diver::{
    camera::PerspectiveCamera,
    environment::{Image, ImageEnvironment},
    render_worker::{RenderRequest, RenderWorker},
};
use cgmath::Vector2;
use eframe::egui;
use egui::{Checkbox, ColorImage, DragValue, Slider, Vec2};
use std::sync::Arc;

struct MyApp {
    environment: Arc<ImageEnvironment>,
    res_scale: f32,
    camera: PerspectiveCamera,
    gr: bool,
    render_worker: RenderWorker<PerspectiveCamera, ImageEnvironment>,
    previous_render: Option<Image>,
}

impl Default for MyApp {
    fn default() -> Self {
        let environment = Arc::new(
            ImageEnvironment::new(
                image::load_from_memory(include_bytes!("../sky.tif"))
                    .unwrap()
                    .into_rgb8(),
            )
            .unwrap(),
        );

        Self {
            // Initialize environments
            environment: environment.clone(),
            res_scale: 0.5,
            camera: Default::default(),
            gr: true,
            render_worker: Default::default(),
            previous_render: None,
        }
    }
}

impl MyApp {
    fn new(__cc: &eframe::CreationContext) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        let app = Self::default();

        // initiate the first render request
        let request = RenderRequest::new(app.camera, app.environment.clone(), app.gr);
        app.render_worker.render(request);

        app
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::SidePanel::new(egui::panel::Side::Left, "left pannel").show(ctx, |ui| {
            ui.heading("Settings");

            ui.horizontal(|ui| {
                ui.label("r/M");
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
                let mut angle_degrees = self.camera.fov.to_degrees();
                ui.add(Slider::new(&mut angle_degrees, 0_f64..=180_f64).suffix("°"));
                self.camera.fov = angle_degrees.to_radians();
            });

            ui.horizontal(|ui| {
                ui.label("Theta");
                let mut angle_degrees = self.camera.facing.theta.to_degrees();
                ui.add(Slider::new(&mut angle_degrees, 0_f64..=180_f64).suffix("°"));
                self.camera.facing.theta = angle_degrees.to_radians();
            });
            ui.horizontal(|ui| {
                ui.label("Phi");
                let mut angle_degrees = self.camera.facing.phi.to_degrees();
                ui.add(Slider::new(&mut angle_degrees, 0_f64..=360_f64).suffix("°"));
                self.camera.facing.phi = angle_degrees.to_radians();
            });

            ui.add(Checkbox::new(&mut self.gr, "GR"));
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // get pixels per egui point
            let pixelsperpoint = frame.info().native_pixels_per_point.unwrap();

            // update the camera resolution
            let space = ui.available_size();
            let res = space * pixelsperpoint * self.res_scale;
            self.camera.resolution = Vector2::new(res.x as u32, res.y as u32);

            // Try to get a new render from the worker
            if let Ok(img) = self.render_worker.receiver.try_recv() {
                // get a new egui texture handle
                let texture: &egui::TextureHandle = &ctx.load_texture(
                    "render texture",
                    ColorImage::from_rgb(
                        [img.dimensions().0 as _, img.dimensions().1 as _],
                        img.as_flat_samples().as_slice(),
                    ),
                    Default::default(),
                );

                // show the image
                ui.image(texture, space);

                // Send a new request
                let request = RenderRequest::new(self.camera, self.environment.clone(), self.gr);
                self.render_worker.render(request);

                // update latest render
                self.previous_render = Some(img);
            // Fall back to displaying the previous render
            } else if let Some(img) = self.previous_render.as_ref() {
                // frame in progress so we need to repaint
                ctx.request_repaint();

                let aspect_ratio = img.dimensions().0 as f32 / img.dimensions().1 as f32;

                let space = if aspect_ratio > 1_f32 {
                    // wide image
                    Vec2::new(ui.available_width(), ui.available_width() / aspect_ratio)
                } else {
                    // narrow image
                    Vec2::new(ui.available_height() * aspect_ratio, ui.available_height())
                };

                // get a new egui texture handle
                let texture: &egui::TextureHandle = &ctx.load_texture(
                    "render texture",
                    ColorImage::from_rgb(
                        [img.dimensions().0 as _, img.dimensions().1 as _],
                        img.as_flat_samples().as_slice(),
                    ),
                    Default::default(),
                );

                // show the image
                ui.image(texture, space);
            }
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
