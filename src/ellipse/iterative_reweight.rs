//! Functions for fitting data points to ellipse using Iterative reweight method.
use anyhow::Result;
use nalgebra as na;

use super::least_square::{least_square_fitting, least_square_fitting_with_weight};

pub fn iterative_reweight(
    data: &[na::Point2<f64>],
    threshold: f64,
    max_iterate: usize,
) -> Result<na::DVector<f64>> {
    let mut params = least_square_fitting(data, 1.0)?;
    let mut previous: na::DVector<f64> =
        na::DVector::<f64>::from_iterator(params.len(), (0..params.len()).map(|_| 0.0));
    for _ in 1..max_iterate {
        if (params.clone() - previous).norm() < threshold {
            break;
        }
        let weight = data
            .iter()
            .map(|pt| {
                let x = pt[0];
                let y = pt[1];
                #[rustfmt::skip]
                let var_mat = na::Matrix6::<f64>::from_column_slice(&[
                    x * x, x * y, 0.0, x, 0.0, 0.0,
                    x * y, x * x + y * y, x * y, y, x, 0.0,
                    0.0, x * y, y * y, 0.0, y , 0.0,
                    x, y, 0.0, 1.0, 0.0, 0.0,
                    0.0, x, y, 0.0, 1.0, 0.0,
                    0.0, 0.0, 0.0, 0.0, 0.0, 0.0
                ]);
                1.0 / params.dot(&(var_mat * params.clone()))
            })
            .collect::<Vec<f64>>();
        previous = params.clone();
        params = least_square_fitting_with_weight(data, 1.0, &weight).unwrap();
    }
    Ok(params)
}

#[cfg(test)]
mod tests {
    use crate::ellipse::test_utility::test_util::{compare_vecs_without_sign, normalize};

    use super::*;
    use rand::prelude::*;

    #[test]
    fn test_iterative_reweight() {
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

        let pred = iterative_reweight(&points, 1e-7, 100).unwrap();
        compare_vecs_without_sign(&ans, pred.as_slice(), 1e-2);
    }
}
