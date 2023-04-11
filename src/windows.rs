use rfd::FileDialog;

use crate::{app::BHDiver, settings::Settings};

pub const ALL_WINDOWS: &[Window] = &[SETTINGS_WINDOW, INFO_WINDOW, RENDER_WINDOW];

pub const SETTINGS_WINDOW: Window = Window {
    name: "Settings",
    build: Settings::build,
};

pub const INFO_WINDOW: Window = Window {
    name: "Info",
    build: |ui, app| {
        egui::Grid::new("performance_grid")
            .striped(true)
            .show(ui, |ui| {
                ui.label("Radius");
                ui.label(format!("{:.3} M", app.scene.diver.position()));
                ui.end_row();

                ui.label("Remaining time");
                ui.label(format!("{:.3} M", app.scene.diver.remaining_time()));
                ui.end_row();

                ui.label("Shell speed");
                ui.label(format!("{:.3}", app.scene.diver.speed()));
                ui.end_row();

                ui.label("Vertical field of view");
                ui.label(format!("{:.3}°", app.scene.camera.fov.to_degrees()));
                ui.end_row();

                ui.label("Focal length");
                ui.label(format!("{:.0} px", app.scene.camera.get_focal_length()));
            });
    },
};

pub const RENDER_WINDOW: Window = Window {
    name: "Render",
    build: |ui, app| {
        if ui
            .add_enabled(
                app.render_manager.is_render_available(),
                egui::Button::new("Save Image"),
            )
            .clicked()
        {
            if let Some(file_path) = FileDialog::new()
                .add_filter("Image", &["png", "jpg", "tif"])
                .save_file()
            {
                app.render_manager.with_render(|render, _| {
                    let _ = render.save(file_path);
                });
            }
        }
    },
};

macro_rules! unique_id {
    ($($args:tt)*) => {
        egui::Id::new((file!(), line!(), column!(), $($args)*))
    };
}

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
