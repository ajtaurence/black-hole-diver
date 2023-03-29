use std::f64::consts::PI;

use quadrature::integrate;

const PHI_ERROR: f64 = 1e-6;

fn n_mod_m<T: std::ops::Rem<Output = T> + std::ops::Add<Output = T> + Copy>(n: T, m: T) -> T {
    ((n % m) + m) % m
}

/// Returns whether the photon at this rain angle incoming or outgoing
fn photon_is_incoming(theta_rain: f64, r: f64, m: f64) -> bool {
    theta_rain.cos() < (r / (2_f64 * m)).sqrt() && theta_rain.cos() < (2_f64 * m / r).sqrt()
}

fn impact_parameter(theta_rain: f64, r: f64, m: f64) -> f64 {
    1_f64
        / ((r - 2_f64 * m * theta_rain.cos().powi(2)).powi(2)
            / (theta_rain.sin().powi(2)
                * r.powi(3)
                * (m + r
                    + 2_f64 * (2_f64 * m * r).sqrt() * theta_rain.cos()
                    + m * (2_f64 * theta_rain).cos())))
        .sqrt()
}

fn turning_point(b: f64, m: f64) -> f64 {
    6_f64 * m / (1_f64 - 2_f64 * ((1_f64 - 54_f64 * m.powi(2) / b.powi(2)).asin() / 3_f64).sin())
}

fn critical_rain_angle(r: f64, m: f64) -> f64 {
    ((27_f64 * (2_f64 * m.powi(5) * r).sqrt() + r * (r * (6_f64 * m + r)).sqrt() * (r - 3_f64 * m))
        / (54_f64 * m.powi(3) + r.powi(3)))
    .acos()
}

fn map_angle_from_impact_parameter(theta_rain: f64, b: f64, m: f64, r: f64) -> f64 {
    fn integrand(x: f64, b: f64, m: f64) -> f64 {
        b / ((1_f64 / (x - 1_f64).powi(4)
            - (b.powi(2) * (1_f64 + 2_f64 * m * (x - 1_f64)) / (x - 1_f64).powi(2)))
        .sqrt()
            * (x - 1_f64).powi(2))
    }

    if photon_is_incoming(theta_rain, r, m) {
        integrate(|x| integrand(x, b, m), 1., (r - 1_f64) / r, PHI_ERROR).integral
    } else {
        let rtp = turning_point(b, m);

        integrate(|x| integrand(x, b, m), 1., (rtp - 1_f64) / rtp, PHI_ERROR).integral
            - integrate(
                |x| integrand(x, b, m),
                (rtp - 1_f64) / rtp,
                (r - 1_f64) / r,
                PHI_ERROR,
            )
            .integral
    }
}

// gives the map theta, phi from the rain theta, phi
pub fn map_coords_from_rain_coords(rain_coords: (f64, f64), r: f64, m: f64) -> Option<(f64, f64)> {
    let (theta_rain, phi_rain) = rain_coords;

    if theta_rain < critical_rain_angle(r, m) {
        return None;
    }

    let b = impact_parameter(theta_rain, r, m);

    let theta_map = PI - map_angle_from_impact_parameter(theta_rain, b, m, r);

    // set theta_map back to range 0->pi
    let theta_map_normalized = theta_map.cos().acos();

    let mut phi_map = phi_rain;

    // flip phi if needed (not sure why but ==1.0 produces the correct image rather than ==-1.0?)
    if theta_map.sin().signum() == 1.0 {
        phi_map = n_mod_m(phi_map + PI, 2_f64 * PI);
    }

    Some((theta_map_normalized, phi_map))
}
