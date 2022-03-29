//! Implementations for rank correction algorithms.
use anyhow::{Context, Result};
use nalgebra as na;

pub fn svd_rank_correction(matrix: na::DMatrix<f64>) -> Result<na::DMatrix<f64>> {
    let mut svd = matrix.svd(true, true);
    let (idx, _) = svd.singular_values.argmin();
    let diag = svd.singular_values.as_mut_slice();
    diag[idx] = 0.0;
    let d = na::DMatrix::<f64>::from_diagonal(&na::DVector::from_vec(diag.to_vec()));
    Ok(svd.u.context("Failed to get SVD value")?
        * d
        * svd.v_t.context("Failed to get SVD value")?)
}

#[cfg(test)]
mod tests {

    use crate::ellipse::test_utility::test_util::compare_matrix;

    use super::*;
    use rand::Rng;

    #[test]
    fn test_svd_rank_correction() {
        let mut rng = rand::thread_rng();
        let s_x = rng.gen::<f64>() * std::f64::consts::PI * 2.0;
        let s_y = rng.gen::<f64>() * std::f64::consts::PI * 2.0;
        let s_z = rng.gen::<f64>() * std::f64::consts::PI * 2.0;
        let v_x = rng.gen::<f64>() * std::f64::consts::PI * 2.0;
        let v_y = rng.gen::<f64>() * std::f64::consts::PI * 2.0;
        let v_z = rng.gen::<f64>() * std::f64::consts::PI * 2.0;
        let d1 = rng.gen::<f64>() + 2.0;
        let d2 = rng.gen::<f64>() + 1.0;
        let d3 = rng.gen::<f64>();

        let s = na::Rotation3::from_euler_angles(s_x, s_y, s_z);
        let v = na::Rotation3::from_euler_angles(v_x, v_y, v_z);
        let d = na::DMatrix::<f64>::from_diagonal(&na::DVector::from_row_slice(&[d1, d2, d3]));
        let input: na::Matrix3<f64> = s.matrix() * d * v.matrix();

        let res = svd_rank_correction(na::DMatrix::<f64>::from_column_slice(
            3,
            3,
            input.as_slice(),
        ))
        .unwrap();

        let d = na::DMatrix::<f64>::from_diagonal(&na::DVector::from_row_slice(&[d1, d2, 0.0]));
        let expect =
            na::DMatrix::<f64>::from_column_slice(3, 3, (s.matrix() * d * v.matrix()).as_slice());
        compare_matrix(&expect, &res);
    }
}
