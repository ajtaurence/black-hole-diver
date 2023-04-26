use crate::{
    camera::{Camera, EquirectangularCamera, PerspectiveCamera},
    scene::Scene,
    traits::Interpolate,
};
use image::{ImageError, RgbImage};
use std::{ffi::OsStr, path::Path};

#[derive(Clone)]
pub struct Frame<C: Camera>(pub i32, pub Scene<C>);

impl<C: Camera> Default for Frame<C> {
    fn default() -> Self {
        Frame(0, Default::default())
    }
}

impl<C: Camera> Interpolate for Frame<C> {
    fn interpolate(&self, other: &Self, factor: f32) -> Self {
        Self(
            self.0.interpolate(&other.0, factor),
            self.1.interpolate(&other.1, factor),
        )
    }
}

impl<C: Camera> PartialEq for Frame<C> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl From<Frame<PerspectiveCamera>> for Frame<EquirectangularCamera> {
    fn from(value: Frame<PerspectiveCamera>) -> Self {
        Self(value.0, value.1.into())
    }
}

impl<C: Camera> PartialOrd for Frame<C> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

#[derive(Default)]
pub struct Animation<C: Camera> {
    frames: Vec<Frame<C>>,
}

impl<C: Camera> Animation<C> {
    pub fn new(frames: Vec<Frame<C>>) -> Self {
        Animation { frames }
    }

    pub fn from_scene_duration(initial_scene: Scene<C>, duration: f64, n_frames: usize) -> Self {
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

    pub fn into_frame_images(self) -> impl Iterator<Item = (i32, RgbImage)> {
        self.frames
            .into_iter()
            .map(|frame| (frame.0, frame.1.render()))
    }

    pub fn save_frames<Q: AsRef<Path>>(self, base_path: Q) -> Result<(), ImageError> {
        // todo: handle unwrap error
        let base_path_name = base_path.as_ref().file_stem().unwrap().to_str().unwrap();

        let mut result = Result::Ok(());
        self.into_frame_images().for_each(|(i, frame)| {
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
