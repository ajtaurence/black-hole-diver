#[derive(Clone, Copy, PartialEq)]
pub struct Diver {
    pub rinit: f64,
    pub time: f64,
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
