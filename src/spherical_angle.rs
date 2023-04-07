use crate::math::{hits_black_hole_no_gr, n_mod_m, rain_angle_to_map_angle};
use nalgebra::Vector3;
use std::f64::consts::PI;

pub trait SphericalAngle {
    fn theta(&self) -> f64;
    fn phi(&self) -> f64;
    fn new(theta: f64, phi: f64) -> Self;

    fn from_vector(vec: Vector3<f64>) -> Self
    where
        Self: Sized,
    {
        let theta = (vec.z / (vec.x * vec.x + vec.y * vec.y + vec.z * vec.z).sqrt()).acos();
        let phi = vec.y.atan2(vec.x);
        Self::new(theta, phi)
    }

    fn to_vector(&self) -> Vector3<f64> {
        Vector3::new(
            self.theta().sin() * self.phi().cos(),
            self.theta().sin() * self.phi().sin(),
            self.theta().cos(),
        )
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RainAngle {
    pub theta: f64,
    pub phi: f64,
}

impl SphericalAngle for RainAngle {
    fn theta(&self) -> f64 {
        self.theta
    }
    fn phi(&self) -> f64 {
        self.phi
    }
    fn new(theta: f64, phi: f64) -> Self {
        Self {
            theta,
            phi: n_mod_m(phi, 2_f64 * PI),
        }
    }
}

impl RainAngle {
    pub fn to_map_angle(self, r: f64) -> Option<MapAngle> {
        let angle = rain_angle_to_map_angle(self.theta, self.phi, r)?;
        Some(MapAngle::new(angle.0, angle.1))
    }

    pub fn try_to_map_angle_no_gr(self, r: f64) -> Option<MapAngle> {
        if hits_black_hole_no_gr(self.theta, r) {
            return None;
        } else {
            return Some(MapAngle::new(self.theta, self.phi));
        }
    }
}

#[derive(Debug, Default)]
pub struct MapAngle {
    pub theta: f64,
    pub phi: f64,
}

impl SphericalAngle for MapAngle {
    fn theta(&self) -> f64 {
        self.theta
    }
    fn phi(&self) -> f64 {
        self.phi
    }
    fn new(theta: f64, phi: f64) -> Self {
        Self {
            theta,
            phi: n_mod_m(phi, 2_f64 * PI),
        }
    }
}
