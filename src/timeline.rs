use crate::{
    animation::{Animation, Frame},
    camera::{Camera, EquirectangularCamera, PerspectiveCamera},
    scene::Scene,
    traits::Interpolate,
};
use std::collections::BTreeMap;

pub struct Timeline<C: Camera> {
    pub start_frame: i32,
    pub end_frame: i32,
    pub fps: f32,
    pub current_frame: i32,
    keyframes: BTreeMap<i32, Scene<C>>,
}

impl<C: Camera + PartialEq> Timeline<C> {
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

    pub fn get_scene(&self, frame: i32) -> Scene<C> {
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

    pub fn get_current_scene(&self) -> Scene<C> {
        self.get_scene(self.current_frame)
    }

    pub fn with_current_scene(&mut self, edit_scene: impl FnOnce(&mut Scene<C>)) {
        let mut scene = self.get_current_scene();

        edit_scene(&mut scene);

        self.set_scene_if_different(self.current_frame, scene)
    }

    pub fn set_scene(&mut self, frame: i32, scene: Scene<C>) {
        self.keyframes.insert(frame, scene);
    }

    pub fn set_scene_if_different(&mut self, frame: i32, scene: Scene<C>) {
        if scene != self.get_scene(frame) {
            self.set_scene(frame, scene);
        }
    }

    pub fn to_animation(&self) -> Animation<C> {
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

impl<C: Camera> Default for Timeline<C> {
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

impl Into<Timeline<EquirectangularCamera>> for Timeline<PerspectiveCamera> {
    fn into(self) -> Timeline<EquirectangularCamera> {
        Timeline {
            start_frame: self.start_frame,
            end_frame: self.end_frame,
            fps: self.fps,
            current_frame: self.current_frame,
            keyframes: self
                .keyframes
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}
