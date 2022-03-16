//! Functions for fitting data points to ellipse using least-square method.
use anyhow::{ensure, Context, Result};
use nalgebra as na;

/// calculate least square solution of eigenvalue problem.
/// Minimize |Ax| subject to |x| = 1.
pub fn lstsq(matrix: &na::DMatrix<f64>) -> Result<na::DVector<f64>> {
    let svd = matrix.clone().svd(false, true);
    let v_t: na::DMatrix<f64> = svd.v_t.context("Failed to get SVD value")?;
    let (row, _) = svd.singular_values.argmin();
    Ok(v_t.row(row).transpose().clone_owned())
}

/// calculate least square solution of a generalized eigenvalue problem.
/// Minimize |Ax| subject to |Cx| = 1.
pub fn constrained_lstsq(
    matrix: &na::DMatrix<f64>,
    constrained: &na::DMatrix<f64>,
) -> Result<na::DVector<f64>> {
    ensure!(
        matrix.ncols() != constrained.ncols(),
        "Invalid matrix size."
    );
    let svd = constrained.clone().svd(false, true);
    let sing_vals = svd.singular_values;
    let v_t: na::DMatrix<f64> = svd.v_t.context("Failed to get SVD value")?;
    // A' = A * V^T
    let a_hat = matrix * v_t.transpose();
    // A' columns where corresponding singular value is not 0.
    let mut a_hat1_vec: Vec<na::DVector<f64>> = vec![];
    // A' columns where corresponding singular value is 0.
    let mut a_hat2_vec: Vec<na::DVector<f64>> = vec![];
    // Non zero singular values
    let mut diag: Vec<f64> = vec![];
    for i in 0..a_hat.ncols() {
        if sing_vals[i].abs() < 1e-5 {
            a_hat2_vec.push(a_hat.column(i).clone_owned());
        } else {
            a_hat1_vec.push(a_hat.column(i).clone_owned());
            diag.push(sing_vals[i]);
        }
    }
    let d1_inv: na::DMatrix<f64> = na::Matrix::from_diagonal(&na::DVector::from_vec(
        diag.iter().map(|val| 1.0 / val).collect(),
    ));

    let a_hat1: na::DMatrix<f64> = na::Matrix::from_columns(&a_hat1_vec);
    let a_hat2: na::DMatrix<f64> = na::Matrix::from_columns(&a_hat2_vec);
    let a_hat2_inv = pseudo_inverse(&a_hat2).context("Failed to calculate pseudo inverse.")?;
    // A'' = (A'_2 * A'_2^+ - I) * A'_1 D_1^-1
    let a_hhat = a_hat2 * a_hat2_inv.clone() - a_hat1.clone() * d1_inv.clone();
    let x_hhat: na::DVector<f64> = lstsq(&a_hhat)?;
    let x1_hat: na::DVector<f64> = d1_inv * x_hhat;
    let x2_hat: na::DVector<f64> = a_hat2_inv * a_hat1 * x1_hat.clone();
    let x_hat = na::DVector::from_iterator(
        x1_hat.len() + x2_hat.len(),
        x1_hat.iter().chain(x2_hat.iter()).copied(),
    );
    Ok(v_t.transpose() * x_hat)
}

/// Calculate pseudo inverse of a given matrix.
pub fn pseudo_inverse(matrix: &na::DMatrix<f64>) -> Result<na::DMatrix<f64>> {
    let svd = matrix.clone().svd(true, true);
    let inv_d = na::Matrix::from_diagonal(&na::DVector::from_vec(
        svd.singular_values
            .iter()
            .map(|val| if *val < 1e-5 { 0.0 } else { 1.0 / val })
            .collect::<Vec<f64>>(),
    ));
    Ok(svd.v_t.context("Failed to get SVD value")?.transpose()
        * inv_d
        * svd.u.context("Failed to get SVD value")?)
}

/// Fit given `data` points to ellipse by least square method.
pub fn least_square_fitting(data: &[na::Point2<f64>], scale: f64) -> Result<na::DVector<f64>> {
    ensure!(
        data.len() >= 5,
        format!("Data point must be 5 or more, not {}", data.len())
    );
    let weight: na::DVector<f64> =
        na::DVector::<f64>::from_iterator(data.len(), (0..data.len()).map(|_| 1.0));
    least_square_fitting_with_weight(data, scale, weight.as_slice())
}

pub fn least_square_fitting_with_weight(
    data: &[na::Point2<f64>],
    scale: f64,
    weight: &[f64],
) -> Result<na::DVector<f64>> {
    let mat: na::Matrix6<f64> =
        data.iter()
            .zip(weight.iter())
            .fold(na::Matrix6::<f64>::zeros(), |mut acc, (pt, w)| {
                let x = pt[0];
                let y = pt[1];
                let xi = na::Vector6::new(
                    x * x,
                    2.0 * x * y,
                    y * y,
                    2.0 * scale * x,
                    2.0 * scale * y,
                    scale * scale,
                );
                acc += *w * xi * xi.transpose();
                acc
            })
            / data.len() as f64;
    lstsq(&na::DMatrix::from_row_slice(6, 6, mat.data.as_slice()))
}

/// Calculate residual for a given point (`pt`).
pub fn calc_residual(pt: &na::Point2<f64>, params: &[f64]) -> f64 {
    let x = pt[0];
    let y = pt[1];
    params[0] * x * x
        + 2.0 * params[1] * x * y
        + params[2] * y * y
        + 2.0 * (params[3] * x + params[4] * y)
        + params[5]
}

#[cfg(test)]
mod tests {
    use crate::ellipse::test_utility::test_util::{compare_vecs_without_sign, normalize};

    use super::*;
    use rand::prelude::*;

    #[test]
    fn test_svd() {
        let mat = na::Matrix2x3::<f64>::new(3.0, 1.0, 2.0, 3.0, 2.0, 1.0);
        let svd = mat.svd(false, true);
        let v = na::Matrix2x3::new(
            0.0,
            -1.0 / 2.0f64.sqrt(),
            1.0 / 2.0f64.sqrt(),
            2.0 / 6.0f64.sqrt(),
            1.0 / 6.0f64.sqrt(),
            1.0 / 6.0f64.sqrt(),
        );
        println!("v = {:?}", v);
        println!("v_t = {:?}", svd.v_t.unwrap());
        for r in 0..2 {
            for c in 0..3 {
                assert!((svd.v_t.unwrap()[r * 3 + c] - v[r * 3 + c]).abs() < 1e-5);
            }
        }
    }

    #[test]
    fn lsm_fit_circle() {
        // x^2 + y^2 - 1 = 0;
        let ans = normalize(&[1.0, 0.0, 1.0, 0.0, 0.0, -1.0]);
        let mut rng = rand::thread_rng();
        let points: Vec<na::Point2<f64>> = (0..1000)
            .map(|_| {
                let rad: f64 = rng.gen::<f64>() * std::f64::consts::PI * 2.0;
                na::Point2::new(rad.cos(), rad.sin())
            })
            .collect();
        points.iter().for_each(|p| {
            let val = calc_residual(&p, &ans);
            assert!(val.abs() < 1e-7, "val = {}", val);
        });

        let params = least_square_fitting(&points, 1.0).unwrap();
        compare_vecs_without_sign(&ans, params.as_slice(), 1e-5);
    }

    #[test]
    fn lsm_fit() {
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

        let params = least_square_fitting(&points, 1.0).unwrap();
        compare_vecs_without_sign(&ans, params.as_slice(), 1e-5);
    }
}
