use crate::{render::RenderSettings, scene::Scene};
use image::RgbImage;

#[derive(Clone)]
pub struct Frame(pub i32, pub Scene);

impl Default for Frame {
    fn default() -> Self {
        Frame(0, Default::default())
    }
}

#[derive(Default)]
pub struct Animation {
    frames: Vec<Frame>,
}

impl Animation {
    pub fn new(frames: Vec<Frame>) -> Self {
        Animation { frames }
    }

    pub fn from_scene_duration(initial_scene: Scene, duration: f64, n_frames: usize) -> Self {
        Self::new(
            (0..n_frames)
                .into_iter()
                .map(|i| {
                    let mut new_scene = initial_scene.clone();
                    new_scene.diver.set_time(
                        new_scene.diver.time() + duration * (i as f64 / (n_frames - 1) as f64),
                    );
                    Frame(i as i32, new_scene)
                })
                .collect(),
        )
    }

    pub fn n_frames(&self) -> usize {
        self.frames.len()
    }

    pub fn render_frames(
        self,
        render_settings: RenderSettings,
    ) -> impl Iterator<Item = (i32, RgbImage)> {
        self.frames
            .into_iter()
            .map(move |frame| (frame.0, frame.1.render(render_settings)))
    }
}
