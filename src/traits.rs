pub trait Interpolate {
    fn interpolate(&self, other: &Self, factor: f32) -> Self;
}

impl Interpolate for f32 {
    fn interpolate(&self, other: &Self, factor: f32) -> Self {
        (1_f32 - factor) * self + factor * other
    }
}

impl Interpolate for f64 {
    fn interpolate(&self, other: &Self, factor: f32) -> Self {
        ((1_f32 - factor) * *self as f32 + factor * *other as f32) as f64
    }
}

impl Interpolate for i32 {
    fn interpolate(&self, other: &Self, factor: f32) -> Self {
        ((1_f32 - factor) * *self as f32 + factor * *other as f32) as i32
    }
}

impl Interpolate for u32 {
    fn interpolate(&self, other: &Self, factor: f32) -> Self {
        ((1_f32 - factor) * *self as f32 + factor * *other as f32) as u32
    }
}
