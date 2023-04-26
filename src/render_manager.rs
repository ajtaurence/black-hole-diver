use crate::camera::Camera;
use crate::scene::Scene;
use image::RgbImage;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub struct RenderManager<C: Camera> {
    working: Arc<Mutex<bool>>,
    previous_render: Arc<Mutex<Option<(RgbImage, Duration)>>>,
    previous_scene: Option<Scene<C>>,
}

impl<C: Camera> Default for RenderManager<C> {
    fn default() -> Self {
        Self {
            working: Arc::new(Mutex::new(false)),
            previous_render: Arc::new(Mutex::new(None)),
            previous_scene: None,
        }
    }
}

impl<C: Camera> RenderManager<C> {
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

    pub fn new_render(&mut self, scene: Scene<C>)
    where
        C: Camera + Send + Sync,
        Scene<C>: PartialEq,
    {
        // if the scene is the same as the last scene then don't re-render it
        if let Some(previous_scene) = &self.previous_scene {
            if scene == *previous_scene {
                return;
            }
        }

        let mut working = self.working.lock().unwrap();

        if !*working {
            *working = true;
            drop(working);

            self.previous_scene = Some(scene.clone());

            let working = self.working.clone();
            let previous_render = self.previous_render.clone();

            // render on a new thread
            thread::spawn(move || {
                // rendering logic
                let start = Instant::now();
                let render = scene.render();

                // save render
                *previous_render.lock().unwrap() = Some((render, Instant::now() - start));
                // update working to false
                *working.lock().unwrap() = false;
            });
        }
    }
}
