use std::f64::{consts::PI, INFINITY};

use quadrature::integrate;

const PHI_ERROR: f64 = 1e-6;
const B_ERROR: f64 = 1e-6;
const B_ITER: usize = 30;

fn find_root<F>(mut a: f64, mut b: f64, mut f: F, max_iter: usize, tolerance: f64) -> f64
where
    F: FnMut(f64) -> f64,
{
    for _ in 0..max_iter {
        let y_a = f(a);

        if <f64>::abs(y_a) < tolerance {
            return a;
        } else {
            let c = a + 0.5 * (b - a);
            if <f64>::signum(y_a * f(c)) == -1.0 {
                b = c;
            } else {
                a = c;
            }
        }
    }
    // return the best guess anyway
    return a;
}

fn n_mod_m<T: std::ops::Rem<Output = T> + std::ops::Add<Output = T> + Copy>(n: T, m: T) -> T {
    ((n % m) + m) % m
}

fn F(b: f64, r: f64, m: f64) -> f64 {
    (1_f64 - b.powi(2) / r.powi(2) * (1_f64 - 2_f64 * m / r)).sqrt()
}

fn max_b(r: f64, m: f64) -> f64 {
    if r <= 2_f64 * m {
        return INFINITY;
    } else {
        (r.powi(3) / (r - 2_f64 * m)).sqrt()
    }
}

pub fn impact_parameter_root_find(theta_rain: f64, r: f64, m: f64) -> f64 {
    // incoming vs outgoing root function for finding b
    if photon_is_incoming(theta_rain, r, m) {
        // use the parameterization b = tan(pi x / 2) to map 0 -> oo to 0 -> 1
        let f = |x: f64| {
            ((PI * x / 2_f64).tan().powi(2) / r.powi(2) * (2_f64 * m / r).sqrt()
                - F((PI * x / 2_f64).tan(), r, m))
                / (1_f64 + 2_f64 * m * (PI * x / 2_f64).tan().powi(2) / r.powi(3))
                - theta_rain.cos()
        };
        (PI * find_root(
            0_f64,
            2_f64 * (max_b(r, m)).atan() / PI,
            &f,
            B_ITER,
            B_ERROR,
        ) / 2_f64)
            .tan()
    } else {
        let f = |x: f64| {
            ((PI * x / 2_f64).tan().powi(2) / r.powi(2) * (2_f64 * m / r).sqrt()
                + F((PI * x / 2_f64).tan(), r, m))
                / (1_f64 + 2_f64 * m * (PI * x / 2_f64).tan().powi(2) / r.powi(3))
                - theta_rain.cos()
        };
        (PI * find_root(
            0_f64,
            2_f64 * (max_b(r, m)).atan() / PI,
            &f,
            B_ITER,
            B_ERROR,
        ) / 2_f64)
            .tan()
    }
}

/// Returns whether the photon at this rain angle incoming or outgoing
fn photon_is_incoming(theta_rain: f64, r: f64, m: f64) -> bool {
    theta_rain.cos() < (r / (2_f64 * m)).sqrt() && theta_rain.cos() < (2_f64 * m / r).sqrt()
}

pub fn impact_parameter(theta_rain: f64, r: f64, m: f64) -> f64 {
    1_f64
        / ((r - 2_f64 * m * theta_rain.cos().powi(2)).powi(2)
            / (theta_rain.sin().powi(2)
                * r.powi(3)
                * (m + r
                    + 2_f64 * (2_f64 * m * r).sqrt() * theta_rain.cos()
                    + m * (2_f64 * theta_rain).cos())))
        .sqrt()
}

pub fn turning_point(b: f64, m: f64) -> f64 {
    6_f64 * m / (1_f64 - 2_f64 * ((1_f64 - 54_f64 * m.powi(2) / b.powi(2)).asin() / 3_f64).sin())
}

pub fn critical_impact_parameter(m: f64) -> f64 {
    27_f64.sqrt() * m
}

pub fn map_angle_from_impact_parameter(theta_rain: f64, b: f64, m: f64, r: f64) -> f64 {
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
            + integrate(|x| integrand(x, b, m), (rtp - 1_f64) / rtp, r, PHI_ERROR).integral
    }
}

// gives the map theta, phi from the rain theta, phi
pub fn map_coords_from_rain_coords(rain_coords: (f64, f64), r: f64, m: f64) -> Option<(f64, f64)> {
    let (theta_rain, phi_rain) = rain_coords;

    let b = impact_parameter(theta_rain, r, m);

    if b < critical_impact_parameter(m) && !photon_is_incoming(theta_rain, r, m) {
        None
    } else {
        let theta_map = PI + map_angle_from_impact_parameter(theta_rain, b, m, r);

        // set theta_map back to range 0->pi
        let theta_map_normalized = theta_map.cos().acos();

        let mut phi_map = phi_rain;

        // flip phi if needed
        if theta_map.sin().signum() == -1.0 {
            phi_map = n_mod_m(phi_map + PI, 2_f64 * PI);
        }

        Some((theta_map_normalized, phi_map))
    }
}

#[test]
fn test() {
    let theta = 0.47 * PI;
    let r = 1000.;

    println!("{:?}", photon_is_incoming(theta, r, 1.));
    println!(
        "{:?}, {:?}",
        map_coords_from_rain_coords((theta, 0.), r, 1.),
        theta
    )
}
