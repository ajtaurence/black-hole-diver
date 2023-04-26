use crate::{
    animation::{Animation, Frame},
    scene::Scene,
    traits::Interpolate,
};
use std::collections::BTreeMap;

pub struct Timeline {
    pub start_frame: i32,
    pub end_frame: i32,
    pub fps: f32,
    pub current_frame: i32,
    keyframes: BTreeMap<i32, Scene>,
}

impl Timeline {
    pub fn new(start_frame: i32, end_frame: i32, fps: f32) -> Self {
        let mut keyframes = BTreeMap::new();

        // add default scene as current keyframe
        keyframes.insert(start_frame, Scene::default());

        Self {
            start_frame,
            end_frame,
            fps,
            current_frame: start_frame,
            keyframes,
        }
    }

    pub fn get_scene(&self, frame: i32) -> Scene {
        if let Some(scene) = self.keyframes.get(&frame) {
            return Clone::clone(scene);
        }

        let left = self.keyframes.range(..frame).last();
        let right = self.keyframes.range(frame..).next();

        match (left, right) {
            (Some(left), Some(right)) => left
                .1
                .interpolate(right.1, (frame - left.0) as f32 / (right.0 - left.0) as f32),
            (None, Some(right)) => Clone::clone(right.1),
            (Some(left), None) => Clone::clone(left.1),
            (None, None) => unreachable!(),
        }
    }

    pub fn get_current_scene(&self) -> Scene {
        self.get_scene(self.current_frame)
    }

    pub fn with_current_scene(&mut self, edit_scene: impl FnOnce(&mut Scene)) {
        let mut scene = self.get_current_scene();

        edit_scene(&mut scene);

        self.set_scene_if_different(self.current_frame, scene)
    }

    pub fn set_scene(&mut self, frame: i32, scene: Scene) {
        self.keyframes.insert(frame, scene);
    }

    pub fn set_scene_if_different(&mut self, frame: i32, scene: Scene) {
        if scene != self.get_scene(frame) {
            self.set_scene(frame, scene);
        }
    }

    pub fn to_animation(&self) -> Animation {
        Animation::new(
            (self.start_frame..=self.end_frame)
                .into_iter()
                .map(|i| Frame(i, self.get_scene(i)))
                .collect(),
        )
    }

    pub fn delete_keyframe(&mut self, frame: i32) {
        // don't delete the last keyframe
        if self.keyframes.len() == 0 {
            return;
        }

        self.keyframes.remove(&frame);
    }
}

impl Default for Timeline {
    fn default() -> Self {
        let mut keyframes = BTreeMap::new();
        keyframes.insert(0, Scene::default());

        Self {
            start_frame: 0,
            end_frame: 120,
            fps: 30_f32,
            current_frame: 0,
            keyframes,
        }
    }
}
