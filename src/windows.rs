use rfd::FileDialog;
use std::{
    sync::{Arc, Mutex},
    thread,
};

use crate::{animation::Animation, app::BHDiver, scene::Scene, settings::Settings};

macro_rules! unique_id {
    ($($args:tt)*) => {
        egui::Id::new((file!(), line!(), column!(), $($args)*))
    };
}

pub const ALL_WINDOWS: &[Window] = &[SETTINGS_WINDOW, INFO_WINDOW, RENDER_WINDOW];

pub const SETTINGS_WINDOW: Window = Window {
    name: "Settings",
    build: Settings::build,
};

pub const INFO_WINDOW: Window = Window {
    name: "Info",
    build: |ui, app| {
        let current_scene = app.timeline.get_current_scene();

        egui::Grid::new("performance_grid")
            .striped(true)
            .show(ui, |ui| {
                ui.label("Radius");
                ui.label(format!("{:.3} M", current_scene.diver.position()));
                ui.end_row();

                ui.label("Remaining time");
                ui.label(format!("{:.3} M", current_scene.diver.remaining_time()));
                ui.end_row();

                ui.label("Shell speed");
                ui.label(format!("{:.3}", current_scene.diver.speed()));
                ui.end_row();

                ui.label("Vertical field of view");
                ui.label(format!("{:.3}°", current_scene.camera.fov.to_degrees()));
                ui.end_row();
            });
    },
};

pub const RENDER_WINDOW: Window = Window {
    name: "Render",
    build: |ui, app| {
        let current_scene = app.timeline.get_current_scene();

        if ui
            .add_enabled(
                app.preview_manager.is_render_available(),
                egui::Button::new("Save Image"),
            )
            .clicked()
        {
            if let Some(file_path) = FileDialog::new()
                .add_filter("Image", &["png", "jpg", "tif"])
                .save_file()
            {
                app.preview_manager.with_render(|render, _| {
                    let _ = render.save(file_path);
                });
            }
        }

        // get rendering mutex from temp data or insert false
        let rendering_id = unique_id!();
        let rendering: Arc<Mutex<bool>> = ui
            .data_mut(|w| w.get_temp(rendering_id))
            .unwrap_or_default();
        ui.data_mut(|w| w.insert_temp(rendering_id, rendering.clone()));

        // 360 rendering
        ui.horizontal(|ui| {
            // add render 360 button if not currently rendering
            /*
            if ui
                .add_enabled(
                    !*rendering.lock().unwrap(),
                    egui::Button::new("Render 360°"),
                )
                .clicked()
            {
                if let Some(file_path) = FileDialog::new()
                    .add_filter("Image", &["png", "jpg", "tif"])
                    .save_file()
                {
                    let rendering_clone = rendering.clone();
                    let scene = current_scene.clone();
                    // we are now rendering
                    *rendering_clone.lock().unwrap() = true;
                    // start a new thread to render and save the image
                    thread::spawn(move || {
                        let image = Scene::<EquirectangularCamera>::from(scene).render();
                        let _ = image.save(file_path);
                        if let Ok(mut rendering_clone) = rendering_clone.lock() {
                            // set rendering to false
                            *rendering_clone = false;
                        }
                    });
                }
            }
            // show spinner when rendering
            if *rendering.lock().unwrap() {
                ui.add(egui::Spinner::new());
            }
            */
        });

        // animation rendering
        ui.horizontal(|ui| {
            let frames_id = unique_id!();
            let duration_id = unique_id!();

            let mut frames: usize = ui.data_mut(|w| w.get_temp(frames_id)).unwrap_or(30);
            let mut duration: f64 = ui.data_mut(|w| w.get_temp(duration_id)).unwrap_or(1_f64);

            ui.label("Frames");
            ui.add(egui::DragValue::new(&mut frames));
            ui.label("Duration");
            ui.add(
                egui::DragValue::new(&mut duration)
                    .clamp_range(0_f64..=current_scene.diver.final_time()),
            );

            // add render 360 button if not currently rendering
            /*
            if ui
                .add_enabled(
                    !*rendering.lock().unwrap(),
                    egui::Button::new("Render Animation"),
                )
                .clicked()
            {
                if let Some(file_path) = FileDialog::new()
                    .add_filter("Image", &["png", "jpg", "tif"])
                    .save_file()
                {
                    let rendering_clone = rendering.clone();
                    let animation =
                        Animation::from_scene_duration(current_scene.clone(), duration, frames);
                    // we are now rendering
                    *rendering_clone.lock().unwrap() = true;
                    // start a new thread to render and save the image
                    thread::spawn(move || {
                        let _ = animation.save_frames(file_path);

                        if let Ok(mut rendering_clone) = rendering_clone.lock() {
                            // set rendering to false
                            *rendering_clone = false;
                        }
                    });
                }
            }
            // show spinner when rendering
            if *rendering.lock().unwrap() {
                ui.add(egui::Spinner::new());
            }
            */

            ui.data_mut(|w| w.insert_temp(frames_id, frames));
            ui.data_mut(|w| w.insert_temp(duration_id, duration));
        });
    },
};

#[allow(unused)]
fn with_temp_data<F, T>(ui: &mut egui::Ui, f: F)
where
    F: FnOnce(&mut egui::Ui, &mut T),
    T: Default + Clone + Send + Sync + 'static,
{
    let id = unique_id!();
    let mut values: T = ui.data_mut(|reader| reader.get_temp(id).unwrap_or_default());

    f(ui, &mut values);

    ui.data_mut(|reader| reader.insert_temp(id, values))
}

pub struct Window {
    pub name: &'static str,
    build: fn(&mut egui::Ui, &mut BHDiver),
}

impl Window {
    pub fn id(&self) -> egui::Id {
        egui::Id::new(unique_id!(self.name))
    }

    pub fn is_open(&self, ctx: &egui::Context) -> bool {
        ctx.data_mut(|reader| reader.get_temp(self.id()).unwrap_or(false))
    }

    pub fn set_open(&self, ctx: &egui::Context, is_open: bool) {
        ctx.data_mut(|reader| reader.insert_temp(self.id(), is_open))
    }

    pub fn show(&self, ctx: &egui::Context, app: &mut BHDiver) {
        let mut is_open = self.is_open(ctx);
        egui::Window::new(self.name)
            .open(&mut is_open)
            .show(ctx, |ui| (self.build)(ui, app));
        self.set_open(ctx, is_open);
    }

    pub fn menu_button(&self, ui: &mut egui::Ui) {
        if ui.button(self.name).clicked() {
            self.set_open(ui.ctx(), true)
        }
    }
}
