use crate::traits::Interpolate;

#[derive(Clone, Copy, PartialEq)]
pub struct Diver {
    rinit: f64,
    time: f64,
}

impl Interpolate for Diver {
    fn interpolate(&self, other: &Self, factor: f32) -> Self {
        Diver::new(
            self.rinit.interpolate(&other.rinit, factor),
            self.time.interpolate(&other.time, factor),
        )
    }
}

impl Default for Diver {
    fn default() -> Self {
        Self {
            rinit: 10_f64,
            time: 0_f64,
        }
    }
}

impl Diver {
    pub fn new(rinit: f64, time: f64) -> Self {
        Diver { rinit, time }
    }

    pub fn initial_radius(&self) -> f64 {
        self.rinit
    }

    pub fn initial_radius_ref(&mut self) -> &mut f64 {
        &mut self.rinit
    }

    pub fn set_initial_radius(&mut self, radius: f64) {
        self.rinit = radius.max(0_f64);
    }

    pub fn time(&self) -> f64 {
        self.time
    }

    pub fn time_ref(&mut self) -> &mut f64 {
        &mut self.time
    }

    pub fn time_step(&mut self, time_step: f64) {
        self.set_time(self.time + time_step)
    }

    pub fn set_time(&mut self, time: f64) {
        self.time = time.min(self.final_time());
    }

    pub fn position(&self) -> f64 {
        (self.rinit.powf(3_f64 / 2_f64) - 3_f64 * self.time / 2_f64.sqrt())
            .powf(2_f64 / 3_f64)
            .max(0_f64)
    }

    pub fn speed(&self) -> f64 {
        (2_f64 / self.position()).sqrt()
    }

    pub fn final_time(&self) -> f64 {
        2_f64.sqrt() * self.rinit.powf(3_f64 / 2_f64) / 3_f64
    }

    pub fn remaining_time(&self) -> f64 {
        2_f64.sqrt() * self.position().powf(3_f64 / 2_f64) / 3_f64
    }
}
