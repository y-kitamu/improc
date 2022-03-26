//! Implementation of FNS (Fundamental Numerial Scheme)
use anyhow::Result;
use nalgebra as na;
use num_traits::Zero;

use crate::{ellipse::iterative_reweight::calc_ellipse_var_mat, linalg::matrix::lstsq};

use super::least_square::calc_ellipse_data_mat;

pub fn fns(
    data: &[na::Point2<f64>],
    threshold: f64,
    max_iterate: usize,
) -> Result<na::DVector<f64>> {
    let mut weight: Vec<f64> = vec![1.0; data.len()];
    let mut previous = na::DVector::<f64>::from_vec(vec![0.0; 6]);
    let mut params = step(data, &weight, &previous)?;

    for _ in 0..max_iterate {
        if previous[0] * params[0] < 0.0 {
            previous *= -1.0;
        }
        if (params.clone() - previous.clone()).norm() < threshold {
            break;
        }
        previous = params.clone();
        weight = data
            .iter()
            .map(|pt| {
                let var_mat = calc_ellipse_var_mat(pt);
                1.0 / params.dot(&(var_mat * params.clone()))
            })
            .collect::<Vec<f64>>();
        params = step(data, &weight, &params)?;
    }
    Ok(params)
}

fn step(
    data: &[na::Point2<f64>],
    weight: &[f64],
    theta: &na::DVector<f64>,
) -> Result<na::DVector<f64>> {
    let m = calc_ellipse_data_mat(data, 1.0, weight);
    let l = data
        .iter()
        .zip(weight.iter())
        .fold(na::Matrix6::<f64>::zeros(), |acc, (pt, w)| {
            let x = pt[0];
            let y = pt[1];
            let xi = na::Vector6::new(
                x * x,
                2.0 * x * y,
                y * y,
                2.0 * 1.0 * x,
                2.0 * 1.0 * y,
                1.0 * 1.0,
            );
            let vm = calc_ellipse_var_mat(pt);
            let dot = theta.dot(&xi);
            acc + (*w) * dot * dot * vm
        })
        / data.len() as f64;
    lstsq(&na::DMatrix::<f64>::from_column_slice(
        6,
        6,
        (m - l).data.as_slice(),
    ))
}

#[cfg(test)]
mod tests {
    use crate::ellipse::test_utility::test_util::{compare_vecs_without_sign, normalize};

    use super::*;

    #[test]
    fn test_fns() {
        // x^2 + 4 * y^2 - 4 = 0
        let ans = normalize(&[1.0, 0.0, 4.0, 0.0, 0.0, -4.0]);
        let r45 = std::f64::consts::FRAC_PI_4;
        let r30 = std::f64::consts::FRAC_PI_6;
        let r60 = std::f64::consts::FRAC_PI_3;
        let points = vec![
            na::Point2::new(2.0, 0.0),
            na::Point2::new(-2.0, 0.0),
            na::Point2::new(0.0, 1.0),
            na::Point2::new(0.0, -1.0),
            na::Point2::new(2.0 * r45.cos(), 1.0 * r45.sin()),
            na::Point2::new(-2.0 * r45.cos(), 1.0 * r45.sin()),
            na::Point2::new(-2.0 * r45.cos(), -1.0 * r45.sin()),
            na::Point2::new(2.0 * r30.cos(), 1.0 * r30.sin()),
            na::Point2::new(-2.0 * r30.cos(), 1.0 * r30.sin()),
            na::Point2::new(-2.0 * r30.cos(), -1.0 * r30.sin()),
            na::Point2::new(2.0 * r60.cos(), 1.0 * r60.sin()),
            na::Point2::new(-2.0 * r60.cos(), 1.0 * r60.sin()),
            na::Point2::new(-2.0 * r60.cos(), -1.0 * r60.sin()),
        ];
        let params = fns(&points, 1e-5, 100).unwrap();
        compare_vecs_without_sign(&ans, params.as_slice(), 1e-5);
    }
}
