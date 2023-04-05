// Contains the math for calculating conversions between rain angles and map angles in the vicinity of a Schwarzschild black hole

use num_traits::{AsPrimitive, Float};
use quadrature::integrate;
use std::f64::consts::PI;

pub fn n_mod_m<T: std::ops::Rem<Output = T> + std::ops::Add<Output = T> + Copy>(n: T, m: T) -> T {
    ((n % m) + m) % m
}

/// Returns whether the photon at this rain angle incoming or outgoing
fn photon_is_incoming<T: Float + 'static>(theta_rain: T, r: T) -> bool
where
    i32: AsPrimitive<T>,
{
    theta_rain.cos() < (r / 2.as_()).sqrt() && theta_rain.cos() < (2.as_() / r).sqrt()
}

/// Returns the impact parameter of the photon at this rain angle
fn impact_parameter<T: Float + 'static>(theta_rain: T, r: T) -> T
where
    i32: AsPrimitive<T>,
{
    r * theta_rain.sin() / ((2.as_() / r).sqrt() * theta_rain.cos() - 1.as_())
}

/// Returns the radius of the turning point given the impact parameter
fn turning_point<T: Float + 'static>(b: T) -> T
where
    i32: AsPrimitive<T>,
{
    6_i32.as_()
        / (1_i32.as_()
            - 2_i32.as_() * ((1_i32.as_() - 54_i32.as_() / b.powi(2)).asin() / 3_i32.as_()).sin())
}

/// Returns the critical rain angle for this radius
fn critical_rain_angle<T: Float + 'static>(r: T) -> T
where
    i32: AsPrimitive<T>,
{
    ((27_i32.as_() * (2_i32.as_() * r).sqrt()
        + r * (r * (6_i32.as_() + r)).sqrt() * (r - 3_i32.as_()))
        / (54_i32.as_() + r.powi(3)))
    .acos()
}

/// Returns the map angle not garanteed to be normalized to any range
fn map_angle_from_impact_parameter<T: Float + 'static>(theta_rain: T, b: T, r: T) -> T
where
    i32: AsPrimitive<T>,
    f64: AsPrimitive<T>,
    T: AsPrimitive<f64>,
{
    // Acceptable error in phi angle
    const PHI_ERROR: f64 = 1e-6;

    // We transform the usual integrand in order to numerically integrate from and infinite radius
    // This also changes the bounds of integration below
    fn integrand(x: f64, b: f64) -> f64 {
        b / ((1_f64 / (x - 1_f64).powi(4)
            - (b.powi(2) * (1_f64 + 2_f64 * (x - 1_f64)) / (x - 1_f64).powi(2)))
        .sqrt()
            * (x - 1_f64).powi(2))
    }

    if photon_is_incoming(theta_rain, r) {
        integrate(
            |x| integrand(x, b.as_()),
            1_f64,
            (r.as_() - 1_f64) / r.as_(),
            PHI_ERROR,
        )
        .integral
        .as_()
    } else {
        let rtp = turning_point(b);

        (integrate(
            |x| integrand(x, b.as_()),
            1_f64,
            (rtp.as_() - 1_f64) / rtp.as_(),
            PHI_ERROR,
        )
        .integral
            - integrate(
                |x| integrand(x, b.as_()),
                (rtp.as_() - 1_f64) / rtp.as_(),
                (r.as_() - 1_f64) / r.as_(),
                PHI_ERROR,
            )
            .integral)
            .as_()
    }
}

/// Returns true if the photon at this rain angle hits the black hole
pub fn hits_black_hole<T: Float + 'static>(theta_rain: T, r: T) -> bool
where
    i32: AsPrimitive<T>,
{
    theta_rain < critical_rain_angle(r)
}

/// Returns the spherical map angle from the rain angle
pub fn rain_angle_to_map_angle<T: Float + 'static>(
    theta_rain: T,
    phi_rain: T,
    r: T,
) -> Option<(T, T)>
where
    i32: AsPrimitive<T>,
    f64: AsPrimitive<T>,
    T: AsPrimitive<f64>,
{
    if hits_black_hole(theta_rain, r) {
        return None;
    }

    let b = impact_parameter(theta_rain, r);

    let theta_map = PI.as_() - map_angle_from_impact_parameter(theta_rain, b, r);

    // set theta_map back to range 0->pi
    let theta_map_normalized = theta_map.cos().acos();

    let mut phi_map = phi_rain;

    // flip phi if needed
    if theta_map.sin().signum() == -1.as_() {
        phi_map = phi_map + PI.as_();
    }

    Some((theta_map_normalized, phi_map))
}
