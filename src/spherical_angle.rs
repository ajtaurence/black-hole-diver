use crate::math::{hits_black_hole, rain_angle_to_map_angle};
use cgmath::{vec3, Rotation3, Vector3};
use std::f64::consts::PI;

pub trait SphericalAngle {
    fn theta(&self) -> f64;
    fn phi(&self) -> f64;
    fn new(theta: f64, phi: f64) -> Self;

    fn from_vector(vec: Vector3<f64>) -> Self
    where
        Self: Sized,
    {
        let theta = (vec.z / (vec.x.powi(2) + vec.y.powi(2) + vec.z.powi(2)).sqrt()).acos();
        let mut phi = vec.y.atan2(vec.x);

        if phi.is_sign_negative() {
            phi = phi + 2_f64 * PI;
        }

        Self::new(theta, phi)
    }

    fn to_vector(&self) -> Vector3<f64> {
        vec3(
            self.theta().sin() * self.phi().cos(),
            self.theta().sin() * self.phi().sin(),
            self.theta().cos(),
        )
    }

    fn rotate<R>(&self, rot: R) -> Self
    where
        R: Rotation3<Scalar = f64>,
        Self: Sized,
    {
        Self::from_vector(rot.rotate_vector(self.to_vector()))
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
        Self::new(theta, phi)
    }
}

impl RainAngle {
    pub fn try_to_map_angle(self, r: f64) -> Option<MapAngle> {
        let angle = rain_angle_to_map_angle(self.theta, self.phi, r)?;
        Some(MapAngle::new(angle.0, angle.1))
    }
    pub fn try_to_map_angle_no_gr(self, r: f64) -> Option<MapAngle> {
        if hits_black_hole(self.theta, r) {
            return None;
        } else {
            return Some(MapAngle::new(self.theta, self.phi));
        }
    }
    pub fn new(theta: f64, phi: f64) -> Self {
        RainAngle { theta, phi }
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
        Self::new(theta, phi)
    }
}

impl MapAngle {
    pub fn new(theta: f64, phi: f64) -> Self {
        MapAngle { theta, phi }
    }
}
