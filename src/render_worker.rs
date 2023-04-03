use crate::environment::Image;
use crate::{camera::Camera, environment::Environment};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Mutex;
use std::{sync::Arc, thread};

pub struct RenderRequest<C: Camera, E: Environment> {
    pub camera: C,
    pub env: Arc<E>,
    pub gr: bool,
}

impl<C: Camera, E: Environment> RenderRequest<C, E> {
    pub fn new(camera: C, env: Arc<E>, gr: bool) -> RenderRequest<C, E> {
        Self { camera, env, gr }
    }
}

pub struct RenderWorker<C: Camera, E: Environment> {
    queue: Arc<Mutex<Vec<RenderRequest<C, E>>>>,
    working: Arc<Mutex<bool>>,
    worker_handle: Option<thread::JoinHandle<()>>,
    pub receiver: Receiver<Image>,
}

impl<C: Camera + Send + 'static, E: Environment + 'static> Default for RenderWorker<C, E> {
    fn default() -> Self {
        RenderWorker::<C, E>::new()
    }
}

impl<C: Camera + Sync + Send + 'static, E: Environment + 'static> RenderWorker<C, E> {
    pub fn new() -> RenderWorker<C, E> {
        let (tx, rx) = channel();

        let working = Arc::new(Mutex::new(false));

        let queue = Arc::new(Mutex::new(Vec::new()));

        let worker_handle = Some(thread::spawn({
            let queue = queue.clone();
            let working = working.clone();
            move || {
                loop {
                    thread::park();

                    *working.lock().unwrap() = true;

                    // Check if there are any new requests in the queue
                    let mut queue = queue.lock().unwrap();

                    debug_assert!(queue.len() == 1, "Render queue should have one item");

                    // If there is a request in the queue, process it
                    let request = queue.remove(0);
                    drop(queue); // Release the lock before rendering
                    let result = render(&request); // This function should render the request
                    tx.send(result).unwrap(); // Send the result back to the requester

                    *working.lock().unwrap() = false;
                }
            }
        }));

        RenderWorker {
            queue,
            working,
            worker_handle,
            receiver: rx,
        }
    }

    pub fn render(&self, request: RenderRequest<C, E>) {
        // get the queue
        let mut queue = self.queue.lock().unwrap();

        // if there are already items in the queue then ignore the request
        if queue.len() > 0 || *self.working.lock().unwrap() {
            return;
        }

        // add the item to the queue
        queue.push(request);

        // notify the worker that there is a new request
        self.worker_handle.as_ref().unwrap().thread().unpark();
    }
}

fn render<C: Camera, E: Environment>(request: &RenderRequest<C, E>) -> Image {
    // rendering logic
    if request.gr {
        request.camera.render(request.env.as_ref())
    } else {
        request.camera.render_no_gr(request.env.as_ref())
    }
}

#[test]
fn test() {
    use crate::camera::PerspectiveCamera;
    use crate::environment::ImageEnvironment;
    use cgmath::Vector2;

    let cam = PerspectiveCamera::new(10., Vector2::new(100, 100), 1.5);

    let env = Arc::new(
        ImageEnvironment::new(
            image::load_from_memory(include_bytes!("../sky.tif"))
                .unwrap()
                .into_rgb8(),
        )
        .unwrap(),
    );

    let worker = RenderWorker::new();

    let request = RenderRequest::new(cam, env.clone(), true);

    worker.render(request);

    worker.receiver.recv().unwrap();

    let request = RenderRequest::new(cam, env.clone(), true);
    worker.render(request);

    worker.receiver.recv().unwrap();
}
