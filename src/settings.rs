use crate::app::BHDiver;

pub struct Settings {
    pub resolution_scale: f32,
    pub mouse_sensitivity: f64,
    pub zoom_sensitivity: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            resolution_scale: 0.5,
            mouse_sensitivity: 10_f64,
            zoom_sensitivity: 10_f64,
        }
    }
}

impl Settings {
    pub fn build(ui: &mut egui::Ui, app: &mut BHDiver) {
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

        ui.heading("Timeline");

        egui::Grid::new("timeline_settings_grid").show(ui, |ui| {
            ui.label("Playback frame rate");
            ui.add(egui::DragValue::new(&mut app.timeline.fps).clamp_range(0_f32..=f32::INFINITY));
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
    }
}
