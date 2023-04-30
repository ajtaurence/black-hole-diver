use crate::{
    preview_manager::PreviewManager,
    render::Renderer,
    settings::Settings,
    timeline::Timeline,
    windows::{ALL_WINDOWS, SETTINGS_WINDOW},
};
use eframe::egui;
use egui::{ColorImage, Sense, Vec2};
use image::GenericImageView;
use nalgebra::Vector2;

#[derive(Default)]
pub struct BHDiver {
    pub timeline: Timeline,
    pub settings: Settings,
    pub preview_manager: PreviewManager,
    pub renderer: Renderer,
}

impl BHDiver {
    pub fn new(__cc: &eframe::CreationContext) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        let mut app = Self::default();

        // initialize first preview
        app.preview_manager
            .new_render(app.timeline.get_current_scene().clone(), Vector2::new(1, 1));

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
        egui::TopBottomPanel::top("menu_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                SETTINGS_WINDOW.menu_button(ui);
            });
        });

        egui::TopBottomPanel::bottom("timeline panel")
            .default_height(100_f32)
            .resizable(true)
            .show(ctx, |ui| {
                self.timeline.show(ui);
            });

        egui::SidePanel::new(egui::panel::Side::Left, "scene panel")
            .resizable(true)
            .show(ctx, |ui| {
                self.timeline.with_current_scene(|scene| {
                    ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                        ui.heading("Scene");
                        scene.show(ui);
                    });
                });
            });

        egui::SidePanel::right("render panel").show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                self.renderer.show(&self.timeline, ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // get pixels per egui point
            let pixelsperpoint = frame.info().native_pixels_per_point.unwrap();

            // update the preview resolution
            let space = ui.available_size();
            let res = space * pixelsperpoint * self.settings.resolution_scale;
            let preview_res = Vector2::new(res.x as u32, res.y as u32);

            self.preview_manager.with_render(|render, _time| {
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
            self.preview_manager
                .new_render(self.timeline.get_current_scene(), preview_res);
        });

        ctx.request_repaint();
    }
}
