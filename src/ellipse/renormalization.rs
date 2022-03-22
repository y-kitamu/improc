//! Implementation of re-normalization method.
//! Re-normalization method is algorithm for fitting data points to ellipse.
use anyhow::Result;
use nalgebra as na;

use super::{
    iterative_reweight::calc_ellipse_var_mat,
    taubin::{taubin, taubin_with_weight},
};

pub fn renormalization(
    data: &[na::Point2<f64>],
    threshold: f64,
    max_iterate: usize,
) -> Result<na::DVector<f64>> {
    let mut params = taubin(data, 1.0)?;
    let mut previous: na::DVector<f64> =
        na::DVector::<f64>::from_iterator(params.len(), (0..params.len()).map(|_| 0.0));

    for _ in 1..max_iterate {
        if previous[0] * params[0] < 0.0 {
            previous *= -1.0;
        }
        if (params.clone() - previous).norm() < threshold {
            break;
        }
        let weight = data
            .iter()
            .map(|pt| {
                let var_mat = calc_ellipse_var_mat(pt);
                1.0 / params.dot(&(var_mat * params.clone()))
            })
            .collect::<Vec<f64>>();
        previous = params.clone();
        params = taubin_with_weight(data, 1.0, &weight).unwrap();
    }
    Ok(params)
}

#[cfg(test)]
mod tests {
    use crate::ellipse::test_utility::test_util::{compare_vecs_without_sign, normalize};

    use super::*;
    use rand::prelude::*;

    #[test]
    fn test_renormalization() {
        // x^2 + y^2 - 1 = 0
        let ans = normalize(&[1.0, 0.0, 1.0, 0.0, 0.0, -1.0]);
        let std_dev = 0.05;
        let mut rng = rand::thread_rng();
        let points: Vec<na::Point2<f64>> = (0..1000)
            .map(|_| {
                let rad: f64 = rng.gen::<f64>() * std::f64::consts::PI * 2.0;
                let dx = (rng.gen::<f64>() - 0.5) * std_dev;
                let dy = (rng.gen::<f64>() - 0.5) * std_dev;
                na::Point2::new(rad.cos() + dx, rad.sin() + dy)
            })
            .collect();

        let pred = renormalization(&points, 1e-7, 100).unwrap();
        let normed = normalize(pred.as_slice());
        compare_vecs_without_sign(&ans, &normed, 1e-2);
    }
}
