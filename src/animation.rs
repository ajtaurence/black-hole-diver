use crate::{camera::Camera, environment::Environment, scene::Scene};
use image::{ImageError, RgbImage};
use std::{ffi::OsStr, path::Path};

#[derive(Default)]
pub struct Animation<C: Camera, E: Environment> {
    scenes: Vec<Scene<C, E>>,
}

impl<C: Camera + 'static, E: Environment + 'static> Animation<C, E> {
    pub fn new(scenes: Vec<Scene<C, E>>) -> Self {
        Animation { scenes }
    }

    pub fn from_scene_duration(initial_scene: Scene<C, E>, duration: f64, frames: usize) -> Self {
        Self::new(
            (0..frames)
                .into_iter()
                .map(|i| {
                    let mut new_scene = initial_scene.clone();
                    new_scene.diver.time = (new_scene.diver.time
                        + duration * (i as f64 / (frames - 1) as f64))
                        .min(new_scene.diver.final_time());
                    new_scene
                })
                .collect(),
        )
    }

    pub fn into_frames(self) -> impl Iterator<Item = RgbImage> {
        self.scenes.into_iter().map(|scene| scene.render())
    }

    pub fn save_frames<Q: AsRef<Path>>(self, base_path: Q) -> Result<(), ImageError> {
        // todo: handle unwrap error
        let base_path_name = base_path.as_ref().file_stem().unwrap().to_str().unwrap();

        let mut result = Result::Ok(());
        self.into_frames().enumerate().for_each(|(i, frame)| {
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
