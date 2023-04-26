use crate::app::BHDiver;

pub struct Settings {
    pub resolution_scale: f32,
    pub mouse_sensitivity: f64,
    pub zoom_sensitivity: f64,
    pub animation_speed: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            resolution_scale: 0.5,
            mouse_sensitivity: 10_f64,
            zoom_sensitivity: 10_f64,
            animation_speed: 1_f64,
        }
    }
}

impl Settings {
    pub fn build(ui: &mut egui::Ui, app: &mut BHDiver) {
        let mut current_scene = app.timeline.get_current_scene();

        ui.heading("Visual");

        egui::Grid::new("visual_grid").show(ui, |ui| {
            ui.label("Enable relativity");
            ui.checkbox(&mut current_scene.gr, "");
        });

        app.timeline.get_current_scene();

        ui.separator();

        ui.heading("Performance");

        egui::Grid::new("performance_grid").show(ui, |ui| {
            ui.label("Resolution scale");
            ui.add(
                egui::DragValue::new(&mut app.settings.resolution_scale)
                    .clamp_range(0_f32..=2_f32)
                    .speed(0.1),
            );
        });

        ui.separator();

        ui.heading("Sensitivity");

        egui::Grid::new("sensitivity_grid").show(ui, |ui| {
            ui.label("Mouse sensitivity");
            ui.add(
                egui::DragValue::new(&mut app.settings.mouse_sensitivity)
                    .clamp_range(0_f64..=f64::MAX),
            );
            ui.end_row();

            ui.label("Zoom sensitivity");
            ui.add(
                egui::DragValue::new(&mut app.settings.zoom_sensitivity)
                    .clamp_range(0_f64..=f64::MAX),
            );
            ui.end_row();
        });

        ui.separator();

        ui.heading("Animation");

        egui::Grid::new("animation_grid").show(ui, |ui| {
            ui.label("Animation speed");
            ui.add(
                egui::DragValue::new(&mut app.settings.animation_speed)
                    .clamp_range(0_f64..=f64::MAX),
            );
        });
    }
}
