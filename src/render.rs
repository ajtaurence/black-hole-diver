use crate::{animation::Animation, camera::Projection, scene::Scene, timeline::Timeline};
use egui::mutex::Mutex;
use nalgebra::Vector2;
use std::{
    ffi::OsStr,
    path::PathBuf,
    sync::{
        mpsc::{channel, Sender},
        Arc,
    },
};

#[derive(Debug, Clone, Copy)]
pub struct RenderSettings {
    pub projection: Projection,
    pub resolution: Vector2<u32>,
    pub super_sampling: Option<usize>,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self::new(Projection::Perspective, Vector2::new(1024, 1024), None)
    }
}

impl RenderSettings {
    pub fn new(
        projection: Projection,
        resolution: Vector2<u32>,
        super_sampling: Option<usize>,
    ) -> Self {
        Self {
            projection,
            resolution,
            super_sampling,
        }
    }

    pub fn preview(resolution: Vector2<u32>) -> Self {
        Self::new(Projection::Perspective, resolution, None)
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Projection");
            egui::ComboBox::from_id_source("projection combo box")
                .selected_text(self.projection.to_string())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.projection,
                        Projection::Perspective,
                        Projection::Perspective.to_string(),
                    );
                    ui.selectable_value(
                        &mut self.projection,
                        Projection::Equirectangular,
                        Projection::Equirectangular.to_string(),
                    );
                });
        });

        if self.projection == Projection::Equirectangular {
            self.resolution.x = self.resolution.y * 2;
        }
        ui.horizontal(|ui| {
            ui.label("Resolution");
            if ui
                .add(egui::DragValue::new(&mut self.resolution.x).suffix(" px"))
                .changed()
            {
                if self.projection == Projection::Equirectangular {
                    self.resolution.y = self.resolution.x / 2;
                    self.resolution.x = self.resolution.y * 2;
                }
            }
            ui.label("×");
            if ui
                .add(egui::DragValue::new(&mut self.resolution.y).suffix(" px"))
                .changed()
            {
                if self.projection == Projection::Equirectangular {
                    self.resolution.x = self.resolution.y * 2;
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label("Super sampling");
            let mut super_sampling_bool = self.super_sampling.is_some();
            ui.checkbox(&mut super_sampling_bool, "");

            if super_sampling_bool {
                let mut super_sampling_value = self.super_sampling.unwrap_or(2);
                ui.add(
                    egui::DragValue::new(&mut super_sampling_value).clamp_range(1_u32..=u32::MAX),
                );
                self.super_sampling = Some(super_sampling_value)
            } else {
                self.super_sampling = None;
            }
        });
    }
}

pub struct Renderer {
    render_settings: RenderSettings,
    output_path: String,
    rendering: Arc<Mutex<bool>>,
    progress: Arc<Mutex<Option<f32>>>,
    cancel_sender: Option<Sender<()>>,
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            render_settings: Default::default(),
            output_path: Default::default(),
            rendering: Arc::new(Mutex::new(false)),
            progress: Arc::new(Mutex::new(None)),
            cancel_sender: None,
        }
    }
}

impl Renderer {
    pub fn cancel_render(&mut self) {
        if let Some(sender) = &self.cancel_sender {
            let result = sender.send(());

            // if there was an error then the rendering thread crashed so set rendering to false
            if result.is_err() {
                *self.rendering.lock() = false;
            }
        }
    }

    pub fn is_rendering(&self) -> bool {
        *self.rendering.lock()
    }

    pub fn get_output_path(&self) -> Option<PathBuf> {
        let path = self.output_path.parse::<PathBuf>().ok()?;

        if image::ImageFormat::from_extension(path.extension()?).is_some() {
            return Some(path);
        } else {
            return None;
        }
    }

    /// returns whether to allow rendering
    pub fn can_render(&self) -> bool {
        self.get_output_path().is_some() && !self.is_rendering()
    }

    pub fn render_frame(&mut self, scene: Scene) {
        // set rendering to true
        *self.rendering.lock() = true;

        let render_settings = self.render_settings.clone();
        let output_path = self.get_output_path();
        let rendering = self.rendering.clone();
        std::thread::spawn(move || {
            // render the image
            let image = scene.render(render_settings);

            // save the image and ignore the result for now
            let _ = image.save(output_path.unwrap());

            // set rendering to false
            *rendering.lock() = false;
        });
    }

    pub fn render_animation(&mut self, animation: Animation) {
        // set rendering to true
        *self.rendering.lock() = true;

        let (sender, receiver) = channel::<()>();

        self.cancel_sender = Some(sender);

        let render_settings = self.render_settings.clone();
        let output_path = self.get_output_path();
        let rendering = self.rendering.clone();
        let progress = self.progress.clone();
        std::thread::spawn(move || {
            let output_path = output_path.unwrap();
            let base_path_name = output_path.file_stem().unwrap().to_str().unwrap();
            let n_frames = animation.n_frames();

            // render the animation
            for (i, (_frame, image)) in animation.render_frames(render_settings).enumerate() {
                // if a cancel message was sent then stop rendering
                if receiver.try_recv().is_ok() {
                    break;
                }

                let mut frame_name = base_path_name.to_owned();
                frame_name.push_str(&format!(".{:0>5}", i + 1));
                frame_name.push_str(&format!(
                    ".{}",
                    output_path.extension().unwrap().to_str().unwrap()
                ));

                let result = image.save(output_path.with_file_name(OsStr::new(&frame_name)));

                // problem saving a frame so stop rendering
                if result.is_err() {
                    break;
                }
                *progress.lock() = Some(i as f32 / n_frames as f32)
            }

            // remove progress
            *progress.lock() = None;
            // set rendering to false
            *rendering.lock() = false;
        });
    }

    pub fn show(&mut self, timeline: &Timeline, ui: &mut egui::Ui) {
        self.render_settings.show(ui);

        // output path

        ui.horizontal(|ui| {
            ui.label("Output");
            if ui.button("🗀").clicked() {
                if let Some(new_path) = rfd::FileDialog::new().save_file() {
                    if let Some(path_string) = new_path.to_str() {
                        self.output_path = path_string.to_owned();
                    }
                }
            }
            ui.add(egui::TextEdit::singleline(&mut self.output_path).desired_width(f32::INFINITY))
        });

        ui.vertical_centered_justified(|ui| {
            if ui
                .add_enabled(self.can_render(), egui::Button::new("Render Current Frame"))
                .clicked()
            {
                self.render_frame(timeline.get_current_scene());
            }
            if ui
                .add_enabled(self.can_render(), egui::Button::new("Render Animation"))
                .clicked()
            {
                self.render_animation(timeline.to_animation());
            }
            if ui
                .add_enabled(self.is_rendering(), egui::Button::new("Cancel Render"))
                .clicked()
            {
                self.cancel_render();
            }

            if let Some(progress) = *self.progress.lock() {
                ui.add(egui::ProgressBar::new(progress).show_percentage());
            }
        });
    }
}
