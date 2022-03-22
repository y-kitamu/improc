use anyhow::Result;
use nalgebra as na;
use num_traits::Zero;

use super::least_square::{calc_ellipse_data_mat, constrained_lstsq};

pub fn taubin(data: &[na::Point2<f64>], scale: f64) -> Result<na::DVector<f64>> {
    let weight: Vec<f64> = (0..data.len()).map(|val| val as f64).collect();
    taubin_with_weight(data, scale, &weight)
}

pub fn taubin_with_weight(
    data: &[na::Point2<f64>],
    scale: f64,
    weight: &[f64],
) -> Result<na::DVector<f64>> {
    let mat = calc_ellipse_data_mat(data, scale, weight);
    let var_mat = calc_ellipse_data_var_mat(data, scale, weight);
    constrained_lstsq(
        &na::DMatrix::from_column_slice(6, 6, mat.as_slice()),
        &na::DMatrix::from_column_slice(6, 6, var_mat.as_slice()),
    )
}

pub fn calc_ellipse_data_var_mat(
    data: &[na::Point2<f64>],
    scale: f64,
    weight: &[f64],
) -> na::Matrix6<f64> {
    // One::one()
    data.iter()
        .zip(weight.iter())
        .fold(Zero::zero(), |acc: na::Matrix6<f64>, (pt, w)| {
            let x = pt[0];
            let y = pt[1];
            #[rustfmt::skip]
        let mat = na::Matrix6::new(
            x * x,     x * y,         0.0,       scale * x,     0.0,           0.0,
            x * y,     x * x + y * y, x * y,     scale * y,     scale * x,     0.0,
            0.0,       x * y,         y * y,     0.0,           scale * y,     0.0,
            scale * x, scale * y,     0.0,       scale * scale, 0.0,           0.0,
            0.0,       scale * x,     scale * y, 0.0,           scale * scale, 0.0,
            0.0,       0.0,           0.0,       0.0,           0.0,           0.0,
        );
            acc + w * 4.0 * mat
        })
        / data.len() as f64
}

#[cfg(test)]
mod tests {
    use crate::ellipse::{
        least_square::calc_residual,
        test_utility::test_util::{compare_vecs_without_sign, normalize},
    };

    use super::*;

    #[test]
    fn test_taubin() {
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
        points.iter().for_each(|p| {
            let val = calc_residual(&p, &ans);
            assert!(val.abs() < 1e-7, "val = {}", val);
        });

        let params = taubin(&points, 1.0).unwrap();
        let normed = normalize(params.as_slice());
        println!("{:?}", normed);
        compare_vecs_without_sign(&ans, &normed, 1e-5);
    }
}
