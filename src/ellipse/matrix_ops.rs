//! Matrix operation functions
use anyhow::{Context, Result};
use nalgebra as na;

pub fn get_identity_mat(size: usize) -> na::DMatrix<f64> {
    na::DMatrix::from_diagonal_element(size, size, 1.0)
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
        * svd.u.context("Failed to get SVD value")?.transpose())
}

#[cfg(test)]
mod tests {
    use crate::ellipse::test_utility::test_util::compare_matrix;

    use super::*;

    #[test]
    fn test_pseudo_inverse() {
        #[rustfmt::skip]
        let mat = na::DMatrix::from_row_slice(3, 3, &[
            1.0, 3.0, 2.0,
            -1.0, 0.0, 1.0,
            2.0, 3.0, 0.0,
        ]);
        #[rustfmt::skip]
        let ans = na::DMatrix::from_row_slice(3, 3, &[
            1.0, -2.0, -1.0,
            -2.0 / 3.0, 4.0 / 3.0, 1.0,
            1.0, -1.0, -1.0,
        ]);

        let res = pseudo_inverse(&mat).unwrap();
        compare_matrix(&ans, &res);
    }
}
