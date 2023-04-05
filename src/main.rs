use bh_diver::{
    camera::PerspectiveCamera,
    environment::{Image, ImageEnvironment},
    render_worker::{RenderRequest, RenderWorker},
};
use eframe::egui;
use egui::{Checkbox, ColorImage, DragValue, Vec2};
use image::GenericImageView;
use nalgebra::{Vector2, Vector3};
use std::sync::Arc;

struct MyApp {
    environment: Arc<ImageEnvironment>,
    res_scale: f64,
    camera: PerspectiveCamera,
    radius: f64,
    look_at: Vector3<f64>,
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
            radius: 10_f64,
            look_at: Vector3::new(0_f64, 0_f64, 1_f64),
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
        let request = RenderRequest::new(app.camera, app.environment.clone(), app.radius, app.gr);
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
                    DragValue::new(&mut self.radius)
                        .clamp_range(0_f64..=f64::MAX)
                        .speed(0.1),
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
                ui.label("Vertical fov");
                let mut fov = self.camera.fov.to_degrees();
                let fov_widget = ui.add(
                    DragValue::new(&mut fov)
                        .suffix("°")
                        .clamp_range(0_f64..=180_f64),
                );
                if fov_widget.changed() {
                    self.camera.fov = fov.to_radians();
                }
            });

            ui.horizontal(|ui| {
                ui.label("Look at");

                ui.add(
                    DragValue::new(&mut self.look_at.x)
                        .clamp_range(-1_f64..=1_f64)
                        .speed(0.01),
                );
                ui.add(
                    DragValue::new(&mut self.look_at.y)
                        .clamp_range(-1_f64..=1_f64)
                        .speed(0.01),
                );
                ui.add(
                    DragValue::new(&mut self.look_at.z)
                        .clamp_range(-1_f64..=1_f64)
                        .speed(0.01),
                );
            });

            self.camera.look_at(
                &Vector3::new(self.look_at.x, self.look_at.y, self.look_at.z),
                &Vector3::new(0_f64, 1_f64, 0_f64),
            );

            ui.add(Checkbox::new(&mut self.gr, "GR"));
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // get pixels per egui point
            let pixelsperpoint = frame.info().native_pixels_per_point.unwrap();

            // update the camera resolution
            let space = ui.available_size();
            let res = space * pixelsperpoint * self.res_scale as f32;
            self.camera.resolution = Vector2::new(res.x as u32, res.y as u32);

            // Try to update the image from the render worker
            if let Ok(img) = self.render_worker.receiver.try_recv() {
                // update latest render
                self.previous_render = Some(img);

                // Send a new request
                let request =
                    RenderRequest::new(self.camera, self.environment.clone(), self.radius, self.gr);
                self.render_worker.render(request);
            }

            // Display an image if we have one
            if let Some(img) = self.previous_render.as_ref() {
                // get the aspect ratio of the image
                let aspect_ratio_img = img.width() as f32 / img.height() as f32;

                // get the aspect ratio of the space to fill
                let aspect_ratio_space = ui.available_width() / ui.available_height();

                // get the space we want to fill with the image
                let space = Vec2::new(
                    ui.available_width()
                        .min(ui.available_height() * aspect_ratio_img),
                    ui.available_height(),
                );

                // get the pixel width of the image to fit in the aspect ratio of the space
                // keeping the height the same
                let img_pixel_width = img
                    .width()
                    .min((img.height() as f32 * aspect_ratio_space) as u32);

                // trim the image
                let img = img.view(
                    (img.width() - img_pixel_width) / 2,
                    0,
                    img_pixel_width,
                    img.height(),
                );

                // get a new egui texture handle
                let texture: &egui::TextureHandle = &ctx.load_texture(
                    "render texture",
                    ColorImage::from_rgb(
                        [img.dimensions().0 as _, img.dimensions().1 as _],
                        img.to_image().as_flat_samples().as_slice(),
                    ),
                    Default::default(),
                );
                // show the image

                let img_ui = ui.vertical_centered(|ui| ui.image(texture, space)).inner;

                if img_ui.hovered() {
                    // allow scrolling when hovered
                    let scroll = ctx.input(|i| i.scroll_delta.y);
                    if scroll != 0_f32 {
                        self.camera.fov =
                            (self.camera.fov * 2_f64.powf(scroll as f64 / 1000_f64)).max(0_f64);
                    }
                }
            }
            ctx.request_repaint();
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
