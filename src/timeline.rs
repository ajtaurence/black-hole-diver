use crate::{
    animation::{Animation, Frame},
    math_utils::first_digit,
    scene::Scene,
    traits::Interpolate,
};
use std::collections::BTreeMap;

pub struct Timeline {
    pub start_frame: i32,
    pub end_frame: i32,
    pub fps: f32,
    pub current_frame: i32,
    // frame and time on which the preview was started
    preview_start: Option<(i32, f64)>,
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
            preview_start: None,
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
            preview_start: None,
            keyframes,
        }
    }

    pub fn start_preview(&mut self, ui: &egui::Ui) {
        self.current_frame = self.current_frame.clamp(self.start_frame, self.end_frame);
        if self.current_frame == self.end_frame {
            self.current_frame = self.start_frame
        }

        self.preview_start = Some((self.current_frame, ui.input(|r| r.time)));
    }

    pub fn stop_preview(&mut self) {
        self.preview_start = None;
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
        // if this is the last keyframe then let clear keyframes handle it
        if self.keyframes.len() == 1 {
            self.clear_keyframes();
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
        self.current_frame = self.start_frame;
    }

    pub fn add_current_keyframe(&mut self) {
        self.keyframes
            .insert(self.current_frame, self.get_current_scene());
    }

    /// returns the left most frame to be drawn to the timeline
    pub fn left_most_frame(&self) -> i32 {
        *self
            .keyframes
            .first_key_value()
            .unwrap()
            .0
            .min(&self.start_frame)
            .min(&self.current_frame)
    }

    /// returns the right most frame to be drawn to the timeline
    pub fn right_most_frame(&self) -> i32 {
        *self
            .keyframes
            .last_key_value()
            .unwrap()
            .0
            .max(&self.end_frame)
            .max(&self.current_frame)
    }

    /// returns the number of frames to be drawn to the timeline
    pub fn n_frames_on_timeline(&self) -> i32 {
        self.right_most_frame() - self.left_most_frame()
    }

    /// returns the number of frames to step by when drawing the frame number
    pub fn frame_step(&self) -> i32 {
        // target step size
        let target_step = self.n_frames_on_timeline() as f32 / 10_f32;

        // floor to a number whose first digit is 1, 2 or 5
        (match first_digit(target_step) {
            0 | 1 => 1,
            2 | 3 | 4 => 2,
            5 | 6 | 7 | 8 | 9 => 5,
            _ => unreachable!(),
        }) * 10_i32.pow(target_step.log10().floor() as u32)
    }

    fn show_timeline_controls(&mut self, ui: &mut egui::Ui) {
        ui.columns(3, |columns| {
            columns[0].horizontal(|ui| {
                // preview buttons
                if ui
                    .button("⏮")
                    .on_hover_text("Jump to start (Shift + ⬅)")
                    .clicked()
                {
                    self.current_frame = self.start_frame;
                }
                if ui
                    .button("⏪")
                    .on_hover_text("Jump to previous keyframe (Ctrl + ⬅)")
                    .clicked()
                {
                    if let Some((&frame, _)) = self.previous_keyframe(self.current_frame) {
                        self.current_frame = frame
                    }
                }
                if self.preview_start.is_some() {
                    if ui.button("⏸").on_hover_text("Pause (Space)").clicked() {
                        self.stop_preview();
                    }
                } else {
                    if ui.button("▶").on_hover_text("Play (Space)").clicked() {
                        self.start_preview(&ui);
                    }
                }
                if ui
                    .button("⏩")
                    .on_hover_text("Jump to next keyframe (Ctrl + ➡)")
                    .clicked()
                {
                    if let Some((&frame, _)) = self.next_keyframe(self.current_frame) {
                        self.current_frame = frame
                    }
                }
                if ui
                    .button("⏭")
                    .on_hover_text("Jump to end (Shift + ➡)")
                    .clicked()
                {
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
            columns[1].vertical_centered(|ui| {
                ui.add(egui::DragValue::new(&mut self.current_frame))
                    .on_hover_text("Current frame")
            });
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
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        // preview start/stop with spacebar
        if ui.input(|r| r.key_pressed(egui::Key::Space)) {
            if self.preview_start.is_some() {
                self.stop_preview();
            } else {
                self.start_preview(&ui);
            }
        }

        // move current frame with arrow keys
        if ui.input_mut(|r| r.consume_key(egui::Modifiers::SHIFT, egui::Key::ArrowRight)) {
            self.current_frame = self.end_frame
        }
        if ui.input_mut(|r| r.consume_key(egui::Modifiers::SHIFT, egui::Key::ArrowLeft)) {
            self.current_frame = self.start_frame
        }
        if ui.input_mut(|r| r.consume_key(egui::Modifiers::NONE, egui::Key::ArrowRight)) {
            self.current_frame += 1;
        }
        if ui.input_mut(|r| r.consume_key(egui::Modifiers::NONE, egui::Key::ArrowLeft)) {
            self.current_frame -= 1;
        }
        if ui.input_mut(|r| r.consume_key(egui::Modifiers::COMMAND, egui::Key::ArrowRight)) {
            if let Some((&frame, _)) = self.next_keyframe(self.current_frame) {
                self.current_frame = frame
            }
        }
        if ui.input_mut(|r| r.consume_key(egui::Modifiers::COMMAND, egui::Key::ArrowLeft)) {
            if let Some((&frame, _)) = self.previous_keyframe(self.current_frame) {
                self.current_frame = frame
            }
        }

        // update current frame if previewing
        if let Some((start_frame, start_time)) = self.preview_start {
            self.current_frame = (((ui.input(|r| r.time) - start_time) * self.fps as f64) as i32
                + start_frame)
                .rem_euclid(self.end_frame - self.start_frame)
                + self.start_frame
        }

        ui.add_space(ui.style().spacing.item_spacing.y);
        self.show_timeline_controls(ui);

        // allocate space for the timeline header
        let (timeline_rect, _) = ui.allocate_at_least(ui.available_size(), egui::Sense::hover());

        // rect to contain the timeline header
        let header_rect = egui::Rect::from_min_max(
            timeline_rect.min,
            egui::pos2(
                timeline_rect.right(),
                timeline_rect.top() + ui.style().spacing.interact_size.y,
            ),
        );

        // rect to contain the timeline body
        let body_rect = egui::Rect::from_min_max(
            egui::pos2(header_rect.left(), header_rect.bottom()),
            timeline_rect.max,
        );

        // gets the x position in egui coordinates for a frame on the timeline
        let frame_to_xpos = |frame: i32| {
            timeline_rect.left()
                + timeline_rect.width() * (frame - self.left_most_frame()) as f32
                    / (self.n_frames_on_timeline() + 1) as f32
        };

        // gets the frame on the timeline for an x position in egui coordinates
        let xpos_to_frame = |xpos: f32| {
            self.left_most_frame()
                + ((self.n_frames_on_timeline() + 1) as f32 * (xpos - timeline_rect.left())
                    / timeline_rect.width())
                .floor() as i32
        };

        // fill animation area
        ui.painter().rect_filled(
            egui::Rect::from_x_y_ranges(
                frame_to_xpos(self.start_frame)..=frame_to_xpos(self.end_frame + 1),
                body_rect.y_range(),
            ),
            egui::Rounding::none(),
            ui.visuals().faint_bg_color,
        );

        // draw frame steps
        let frame_step = self.frame_step();
        ((self.left_most_frame() as f32 / frame_step as f32).floor() as i32 * frame_step
            ..=self.right_most_frame() + 1)
            .step_by(frame_step as usize)
            .for_each(|i| {
                // frame number text
                ui.painter().text(
                    egui::pos2(frame_to_xpos(i), header_rect.top()),
                    egui::Align2::CENTER_TOP,
                    i.to_string(),
                    egui::TextStyle::Body.resolve(ui.style()),
                    ui.visuals().text_color(),
                );
                // vertical line below
                ui.painter().line_segment(
                    [
                        egui::pos2(frame_to_xpos(i), header_rect.bottom()),
                        egui::pos2(frame_to_xpos(i), timeline_rect.bottom()),
                    ],
                    ui.visuals().widgets.noninteractive.bg_stroke,
                )
            });

        // keyframes
        let mut drag_keyframes = Vec::new();
        let mut delete_keyframes = Vec::new();
        for (&frame, _) in self.keyframes.iter() {
            const KEYFRAME_SIZE: f32 = 5_f32;
            let get_keyframe_response = |ui: &mut egui::Ui| {
                // create a rect where the keyframe goes
                let keyframe_rect = egui::Rect::from_center_size(
                    egui::pos2(frame_to_xpos(frame), body_rect.left_center().y),
                    egui::Vec2::splat(KEYFRAME_SIZE),
                );
                // allocate the rect
                let keyframe_response = ui.allocate_rect(keyframe_rect, egui::Sense::click());

                let visuals = ui.visuals().selection;
                // draw the keyframe
                ui.painter().circle(
                    egui::pos2(frame_to_xpos(frame), body_rect.left_center().y),
                    KEYFRAME_SIZE,
                    visuals.bg_fill,
                    visuals.stroke,
                );

                return keyframe_response;
            };

            // create an id for the keyframe
            let id = egui::Id::new("keyframe").with(frame);

            // check if keyframe is being dragged already
            // let dragging = ui.memory(|r| r.is_being_dragged(id));
            let dragging = false;

            if dragging {
                let layer_id = egui::LayerId::new(egui::Order::Tooltip, id);

                let _ = ui.with_layer_id(layer_id, get_keyframe_response);

                if let Some(pos) = ui.ctx().pointer_interact_pos() {
                    // translate the ui to the cursor
                    ui.ctx()
                        .translate_layer(layer_id, egui::vec2(pos.x - frame_to_xpos(frame), 0_f32));

                    if ui.input(|r| r.pointer.any_released()) {
                        // if the keyframe was dragged add to a vector of keyframes that need to be moved
                        // vector contains tuples (from_frame, to_frame)
                        drag_keyframes.push((frame, xpos_to_frame(pos.x)));
                    }
                };
            } else {
                let keyframe_response = get_keyframe_response(ui);
                let keyframe_response =
                    ui.interact(keyframe_response.rect, id, egui::Sense::drag());
                /*
                if keyframe_response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                }
                */

                keyframe_response.context_menu(|ui| {
                    if ui.button("Delete").clicked() {
                        delete_keyframes.push(frame)
                    }
                });
            };
        }

        // playhead
        let mut play_head_stroke = ui.visuals().widgets.inactive.fg_stroke;
        play_head_stroke.width = 1_f32;
        let playhead_xpos = frame_to_xpos(self.current_frame);
        ui.painter()
            .vline(playhead_xpos, body_rect.y_range(), play_head_stroke);

        ui.painter().add(egui::Shape::convex_polygon(
            vec![
                egui::pos2(playhead_xpos, body_rect.top()),
                egui::pos2(playhead_xpos - 5_f32, body_rect.top() - 7.5),
                egui::pos2(playhead_xpos + 5_f32, body_rect.top() - 7.5),
            ],
            play_head_stroke.color,
            play_head_stroke,
        ));

        // delete keyframes
        for frame in delete_keyframes {
            self.delete_keyframe(frame);
        }

        // move keyframes
        for (from_frame, to_frame) in drag_keyframes {
            self.move_keyframe(from_frame, to_frame);
        }

        ui.add_space(ui.style().spacing.item_spacing.y);
    }
}
