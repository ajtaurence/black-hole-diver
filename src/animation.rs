use crate::{camera::Projection, scene::Scene};
use image::{ImageError, RgbImage};
use nalgebra::Vector2;
use std::{ffi::OsStr, path::Path};

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

    pub fn render_frames(
        self,
        projection: Projection,
        resolution: Vector2<u32>,
    ) -> impl Iterator<Item = (i32, RgbImage)> {
        self.frames
            .into_iter()
            .map(move |frame| (frame.0, frame.1.render(projection, resolution)))
    }

    pub fn save_frames<Q: AsRef<Path>>(
        self,
        base_path: Q,
        frames: impl Iterator<Item = (i32, RgbImage)>,
    ) -> Result<(), ImageError> {
        // todo: handle unwrap error
        let base_path_name = base_path.as_ref().file_stem().unwrap().to_str().unwrap();

        let mut result = Result::Ok(());
        frames.for_each(|(i, frame)| {
            let mut frame_name = base_path_name.to_owned();
            frame_name.push_str(&format!(".{:0>4}", i + 1));
            frame_name.push_str(&format!(
                ".{}",
                base_path.as_ref().extension().unwrap().to_str().unwrap()
            ));

            result = frame.save(base_path.as_ref().with_file_name(OsStr::new(&frame_name)));
            if result.is_err() {
                return;
            }
        });

        result
    }
}
