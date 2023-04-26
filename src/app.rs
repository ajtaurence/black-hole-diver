use crate::{
    camera::PerspectiveCamera,
    render_manager::RenderManager,
    settings::Settings,
    timeline::Timeline,
    windows::{ALL_WINDOWS, INFO_WINDOW, RENDER_WINDOW, SETTINGS_WINDOW},
};
use eframe::egui;
use egui::{ColorImage, DragValue, Sense, Vec2};
use image::GenericImageView;
use nalgebra::Vector2;
use std::time::Instant;

pub struct BHDiver {
    pub timeline: Timeline<PerspectiveCamera>,
    pub settings: Settings,
    pub time_of_last_frame: Instant,
    pub animating: bool,
    pub render_manager: RenderManager<PerspectiveCamera>,
}

impl Default for BHDiver {
    fn default() -> Self {
        Self {
            timeline: Default::default(),
            settings: Default::default(),
            time_of_last_frame: Instant::now(),
            animating: false,
            render_manager: Default::default(),
        }
    }
}

impl BHDiver {
    pub fn new(__cc: &eframe::CreationContext) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        let mut app = Self::default();

        app.render_manager
            .new_render(app.timeline.get_current_scene().clone());

        app
    }
}

impl eframe::App for BHDiver {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Show all the windows
        ALL_WINDOWS.iter().for_each(|window| {
            window.show(ctx, self);
        });

        // Menu bar
        egui::TopBottomPanel::top("top_pannel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                INFO_WINDOW.menu_button(ui);
                SETTINGS_WINDOW.menu_button(ui);
                RENDER_WINDOW.menu_button(ui);
            });
        });

        egui::SidePanel::new(egui::panel::Side::Left, "left pannel").show(ctx, |ui| {
            egui::Grid::new("main_grid").num_columns(2).show(ui, |ui| {
                ui.label("Frame");
                ui.add(
                    DragValue::new(&mut self.timeline.current_frame)
                        .clamp_range(self.timeline.start_frame..=self.timeline.end_frame),
                );
                ui.end_row();

                ui.label("Initial radius");
                self.timeline.with_current_scene(|current_scene| {
                    ui.add(
                        DragValue::new(current_scene.diver.initial_radius_ref())
                            .clamp_range(0_f64..=f64::MAX)
                            .speed(0.1)
                            .suffix(" M"),
                    );
                });

                ui.end_row();

                ui.label("Time");

                // update diver time
                self.timeline.with_current_scene(|current_scene| {
                    if self.animating {
                        current_scene.diver.time_step(
                            ctx.input(|r| r.stable_dt) as f64 * self.settings.animation_speed,
                        );

                        if current_scene.diver.time() == current_scene.diver.final_time() {
                            self.animating = false;
                        }
                    }
                    self.time_of_last_frame = Instant::now();

                    let final_time = current_scene.diver.final_time();
                    ui.add(
                        DragValue::new(current_scene.diver.time_ref())
                            .clamp_range(f64::MIN..=final_time)
                            .speed(0.1)
                            .suffix(" M"),
                    );
                });
                ui.end_row();

                ui.label("Animate");
                if !self.animating {
                    if ui.button("▶").clicked() {
                        self.animating = true;
                    }
                } else {
                    if ui.button("⏸").clicked() {
                        self.animating = false;
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // get pixels per egui point
            let pixelsperpoint = frame.info().native_pixels_per_point.unwrap();

            let mut render_scene = self.timeline.get_current_scene();

            self.render_manager.with_render(|render, _time| {
                // update the camera resolution
                let space = ui.available_size();
                let res = space * pixelsperpoint * self.settings.resolution_scale;
                render_scene.camera.resolution = Vector2::new(res.x as u32, res.y as u32);

                // get the aspect ratio of the image
                let aspect_ratio_img = render.width() as f32 / render.height() as f32;

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
                let img_pixel_width = render
                    .width()
                    .min((render.height() as f32 * aspect_ratio_space) as u32);

                // trim the image
                let img = render.view(
                    (render.width() - img_pixel_width) / 2,
                    0,
                    img_pixel_width,
                    render.height(),
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

                let img_ui = ui
                    .vertical_centered(|ui| {
                        ui.add(egui::Image::new(texture, space).sense(Sense::click_and_drag()))
                    })
                    .inner;

                // Don't show the cursor when holding down on the image
                if img_ui.is_pointer_button_down_on() {
                    ctx.set_cursor_icon(egui::CursorIcon::None)
                }

                // handle input
                // scrolling while hovered
                self.timeline.with_current_scene(|current_scene| {
                    if img_ui.hovered() {
                        let scroll = ctx.input(|i| i.scroll_delta.y);
                        if scroll != 0_f32 {
                            current_scene
                                .camera
                                .zoom(scroll, self.settings.zoom_sensitivity)
                        }
                    }
                    // mouse drag
                    let drag_delta = img_ui.drag_delta();
                    if drag_delta.length() != 0_f32 {
                        current_scene
                            .camera
                            .drag_delta(drag_delta, self.settings.mouse_sensitivity)
                    }
                });
            });

            // Start a new render
            self.render_manager.new_render(render_scene);

            if self.render_manager.is_working() {
                ctx.request_repaint();
            }
        });

        // handle spacebar input
        if ctx.input(|r| r.key_pressed(egui::Key::Space)) {
            self.animating = !self.animating;
        }
    }
}
