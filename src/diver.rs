pub struct Diver {
    rinit: f64,
}

impl Diver {
    pub fn new(rinit: f64) -> Self {
        Diver { rinit }
    }

    pub fn time_from_position(&self, r: f64) -> f64 {
        2_f64.sqrt() / 3_f64 * (self.rinit.powf(3_f64 / 2_f64) - r.powf(3_f64 / 2_f64))
    }

    pub fn position_from_time(&self, t: f64) -> f64 {
        (self.rinit.powf(3_f64 / 2_f64) - 3_f64 * t / 2_f64.sqrt()).powf(2_f64 / 3_f64)
    }

    pub fn speed_from_time(&self, t: f64) -> f64 {
        (2_f64 / self.position_from_time(t)).sqrt()
    }

    pub fn speed_from_position(&self, r: f64) -> f64 {
        (2_f64 / r).sqrt()
    }
}
