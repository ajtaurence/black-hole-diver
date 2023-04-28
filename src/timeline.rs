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
    previewing: bool,
    keyframes: BTreeMap<i32, Scene>,
}

impl Default for Timeline {
    fn default() -> Self {
        let mut keyframes = BTreeMap::new();
        keyframes.insert(1, Scene::default());

        Self {
            start_frame: 1,
            end_frame: 120,
            fps: 30_f32,
            current_frame: 1,
            previewing: false,
            keyframes,
        }
    }
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
            previewing: false,
            keyframes,
        }
    }

    pub fn start_preview(&mut self) {
        self.previewing = true;
        self.current_frame = self.current_frame.clamp(self.start_frame, self.end_frame);
        if self.current_frame == self.end_frame {
            self.current_frame = self.start_frame
        }
    }

    pub fn stop_preview(&mut self) {
        self.previewing = false;
        self.current_frame = self.current_frame.clamp(self.start_frame, self.end_frame);
    }

    pub fn next_keyframe(&self, frame: i32) -> Option<(&i32, &Scene)> {
        self.keyframes.range(frame + 1..).next()
    }

    pub fn previous_keyframe(&self, frame: i32) -> Option<(&i32, &Scene)> {
        self.keyframes.range(..frame).last()
    }

    pub fn get_scene(&self, frame: i32) -> Scene {
        if let Some(scene) = self.keyframes.get(&frame) {
            return Clone::clone(scene);
        }

        let left = self.previous_keyframe(frame);
        let right = self.next_keyframe(frame);

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
        if self.keyframes.len() == 1 {
            return;
        }

        self.keyframes.remove(&frame);
    }

    pub fn move_keyframe(&mut self, from_frame: i32, to_frame: i32) {
        if let Some((_, scene)) = self.keyframes.remove_entry(&from_frame) {
            self.keyframes.insert(to_frame, scene);
        }
    }

    pub fn clear_keyframes(&mut self) {
        let scene = self.get_current_scene();
        self.keyframes = BTreeMap::new();
        self.keyframes.insert(self.start_frame, scene);
    }

    pub fn add_current_keyframe(&mut self) {
        self.keyframes
            .insert(self.current_frame, self.get_current_scene());
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        // preview spacebar
        if ui.input(|r| r.key_pressed(egui::Key::Space)) {
            if self.previewing {
                self.stop_preview();
            } else {
                self.start_preview();
            }
        }

        if self.previewing {
            // update current frame
            self.current_frame += (ui.input(|r| r.stable_dt) * self.fps as f32).ceil() as i32;

            if self.current_frame > self.end_frame {
                self.current_frame = self.end_frame;
                self.stop_preview();
            }
        }

        ui.vertical(|ui| {
            ui.columns(3, |columns| {
                columns[0].horizontal(|ui| {
                    // preview buttons
                    if ui.button("⏮").on_hover_text("Jump to start").clicked() {
                        self.current_frame = self.start_frame;
                    }
                    if ui
                        .button("⏪")
                        .on_hover_text("Jump to previous keyframe")
                        .clicked()
                    {
                        if let Some((&frame, _)) = self.previous_keyframe(self.current_frame) {
                            self.current_frame = frame
                        }
                    }
                    if self.previewing {
                        if ui.button("⏸").on_hover_text("Pause").clicked() {
                            self.stop_preview();
                        }
                    } else {
                        if ui.button("▶").on_hover_text("Play").clicked() {
                            self.start_preview();
                        }
                    }
                    if ui
                        .button("⏩")
                        .on_hover_text("Jump to next keyframe")
                        .clicked()
                    {
                        if let Some((&frame, _)) = self.next_keyframe(self.current_frame) {
                            self.current_frame = frame
                        }
                    }
                    if ui.button("⏭").on_hover_text("Jump to end").clicked() {
                        self.current_frame = self.end_frame;
                    }
                    if ui.button("⏺").on_hover_text("Add keyframe").clicked() {
                        self.add_current_keyframe();
                    }
                    if ui
                        .button("Clear")
                        .on_hover_text("Clear keyframes")
                        .clicked()
                    {
                        self.clear_keyframes();
                    }
                });
                columns[1]
                    .vertical_centered(|ui| ui.add(egui::DragValue::new(&mut self.current_frame)));
                columns[2].with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    // start and end frames
                    ui.add(
                        egui::DragValue::new(&mut self.end_frame)
                            .clamp_range(self.start_frame..=i32::MAX),
                    );
                    ui.label("End");
                    ui.add(
                        egui::DragValue::new(&mut self.start_frame)
                            .clamp_range(i32::MIN..=self.end_frame),
                    );
                    ui.label("Start");
                });
            });

            // allocate space for timeline
            let (rect, response) = ui.allocate_at_least(
                egui::Vec2::new(ui.available_width(), 50_f32),
                egui::Sense::click_and_drag(),
            );

            // gets the x position in egui coordinates for a frame on the timeline
            let frame_to_xpos = |frame: i32| {
                let left_most_frame = *self
                    .keyframes
                    .first_key_value()
                    .unwrap()
                    .0
                    .min(&self.start_frame)
                    .min(&self.current_frame);

                let right_most_frame = *self
                    .keyframes
                    .last_key_value()
                    .unwrap()
                    .0
                    .max(&self.end_frame)
                    .max(&self.current_frame);

                rect.left()
                    + rect.width() * (frame - left_most_frame) as f32 / right_most_frame as f32
            };

            // gets the frame on the timeline for an x position in egui coordinates
            let xpos_to_frame = |xpos: f32| {
                let left_most_frame = *self
                    .keyframes
                    .first_key_value()
                    .unwrap()
                    .0
                    .min(&self.start_frame)
                    .min(&self.current_frame);

                let n_frames = *self
                    .keyframes
                    .last_key_value()
                    .unwrap()
                    .0
                    .max(&self.end_frame)
                    .max(&self.current_frame)
                    - left_most_frame;

                left_most_frame
                    + (n_frames as f32 * (xpos - rect.left()) / rect.width()).floor() as i32
            };

            let selectabled_visuals = ui.style().interact_selectable(&response, true);

            // fill animation area
            ui.painter().rect_filled(
                egui::Rect::from_x_y_ranges(
                    frame_to_xpos(self.start_frame)..=frame_to_xpos(self.end_frame + 1),
                    rect.y_range(),
                ),
                egui::Rounding::none(),
                ui.visuals().faint_bg_color,
            );

            // keyframes
            let mut drag_keyframes = Vec::new();
            for (&frame, _) in self.keyframes.iter() {
                const KEYFRAME_SIZE: f32 = 5_f32;
                let keyframe_rect = egui::Rect::from_center_size(
                    egui::pos2(frame_to_xpos(frame), rect.left_center().y),
                    egui::Vec2::splat(KEYFRAME_SIZE),
                );

                let get_keyframe_response =
                    |ui: &mut egui::Ui| ui.allocate_rect(keyframe_rect, egui::Sense::click());

                let id = egui::Id::new("keyframe").with(frame);
                let dragging = ui.memory(|r| r.is_being_dragged(id));

                let keyframe_response = if dragging {
                    let layer_id = egui::LayerId::new(egui::Order::Tooltip, id);
                    let keyframe_response =
                        ui.with_layer_id(layer_id, get_keyframe_response).response;

                    if let Some(pos) = ui.ctx().pointer_interact_pos() {
                        ui.ctx().translate_layer(
                            layer_id,
                            egui::vec2(pos.x - frame_to_xpos(frame), 0_f32),
                        );

                        if ui.input(|r| r.pointer.any_released()) {
                            // if the keyframe was dragged add to a vector of keyframes that need to be moved
                            // vector contains tuples (from_frame, to_frame)
                            drag_keyframes.push((frame, xpos_to_frame(pos.x)));
                        }
                    };

                    keyframe_response
                } else {
                    let rect = get_keyframe_response(ui).rect;
                    ui.interact(rect, id, egui::Sense::click_and_drag())
                };

                if keyframe_response.hovered() || dragging {
                    ui.painter().circle(
                        egui::pos2(frame_to_xpos(frame), rect.left_center().y),
                        KEYFRAME_SIZE,
                        selectabled_visuals.bg_fill,
                        selectabled_visuals.fg_stroke,
                    );
                } else {
                    ui.painter().circle_filled(
                        egui::pos2(frame_to_xpos(frame), rect.left_center().y),
                        KEYFRAME_SIZE,
                        selectabled_visuals.bg_fill,
                    );
                };
            }

            // playhead
            let mut play_head_stroke = ui.visuals().widgets.noninteractive.bg_stroke;
            play_head_stroke.width = 2_f32;
            ui.painter().vline(
                frame_to_xpos(self.current_frame),
                rect.y_range(),
                play_head_stroke,
            );

            // move keyframes
            for (from_frame, to_frame) in drag_keyframes {
                self.move_keyframe(from_frame, to_frame);
            }
        });
    }
}
