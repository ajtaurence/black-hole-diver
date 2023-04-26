use crate::camera::Projection;
use crate::scene::Scene;
use image::RgbImage;
use nalgebra::Vector2;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub struct PreviewManager {
    working: Arc<Mutex<bool>>,
    previous_render: Arc<Mutex<Option<(RgbImage, Duration)>>>,
    previous_scene_resolution: Option<(Scene, Vector2<u32>)>,
}

impl Default for PreviewManager {
    fn default() -> Self {
        Self {
            working: Arc::new(Mutex::new(false)),
            previous_render: Arc::new(Mutex::new(None)),
            previous_scene_resolution: None,
        }
    }
}

impl PreviewManager {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn is_render_available(&self) -> bool {
        self.previous_render
            .as_ref()
            .lock()
            .unwrap()
            .deref()
            .is_some()
    }

    pub fn with_render(&self, func: impl FnOnce(&RgbImage, Duration) -> ()) {
        if let Some((render, duration)) = self
            .previous_render
            .as_ref()
            .lock()
            .unwrap()
            .deref()
            .as_ref()
        {
            func(render, *duration)
        }
    }

    pub fn is_working(&self) -> bool {
        *self.working.lock().unwrap()
    }

    pub fn new_render(&mut self, scene: Scene, resolution: Vector2<u32>) {
        // if the scene is the same as the last scene then don't re-render it
        if let Some((previous_scene, previous_resolution)) = &self.previous_scene_resolution {
            if scene == *previous_scene && resolution == *previous_resolution {
                return;
            }
        }

        let mut working = self.working.lock().unwrap();

        if !*working {
            *working = true;
            drop(working);

            self.previous_scene_resolution = Some((scene.clone(), resolution));

            let working = self.working.clone();
            let previous_render = self.previous_render.clone();

            // render on a new thread
            thread::spawn(move || {
                // rendering logic
                let start = Instant::now();
                let render = scene.render(Projection::Perspective, resolution);

                // save render
                *previous_render.lock().unwrap() = Some((render, Instant::now() - start));
                // update working to false
                *working.lock().unwrap() = false;
            });
        }
    }
}
