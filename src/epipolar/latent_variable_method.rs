use anyhow::{Context, Result};
use nalgebra as na;

const MAX_ITERATION: usize = 100;

/// Fundamental matrix optimization.
/// `matrix` is 3x3 matrix of rank 3. (rank of the matrix is not corrected.)
pub fn latent_variable_method(matrix: na::DMatrix<f64>) -> Result<na::DMatrix<f64>> {
    let svd = matrix.svd(true, true);
    let singular_values = matrix.singular_values();
    let (idx, _) = singular_values.argmin();
    let mut diag = na::DMatrix::<f64>::from_diagonal(&singular_values);
    diag[idx] = 0.0;
    let u = svd.u.context("Failed to calc svd.")?;
    let v_t = svd.v_t.context("Failed to calc svd")?;
    let matrix = u * diag * v_t;

    let mut c = 1e-4;

    for i in 0..MAX_ITERATION {
        #[rustfmt::skip]
        let f_u = na::DMatrix::from_row_slice(9, 3, &[
            0.0, matrix[(3, 1)], -matrix[(2, 1)],
            0.0, matrix[(3, 2)], -matrix[(2, 2)],
            0.0, matrix[(3, 3)], -matrix[(2, 3)],
            -matrix[(3, 1)], 0.0, matrix[(1, 1)],
            -matrix[(3, 2)], 0.0, matrix[(1, 2)],
            -matrix[(3, 3)], 0.0, matrix[(1, 3)],
            matrix[(2, 1)], -matrix[(1, 1)], 0.0,
            matrix[(2, 2)], -matrix[(1, 2)], 0.0,
            matrix[(2, 3)], -matrix[(1, 3)], 0.0,
        ]);
        #[rustfmt::skip]
        let f_v = na::DMatrix::from_row_slice(9, 3, &[
            0.0, matrix[(1, 3)], -matrix[(1, 2)],
            -matrix[(1, 3)], 0.0, matrix[(1, 1)],
            matrix[(1, 2)], -matrix[(1, 1)], 0.0,
            0.0, matrix[(2, 3)], -matrix[(2, 2)],
            -matrix[(2, 3)], 0.0, matrix[(2, 1)],
            matrix[(2, 2)], -matrix[(2, 1)], 0.0,
            0.0, matrix[(3, 3)], -matrix[(3, 2)],
            -matrix[(3, 3)], 0.0, matrix[(3, 1)],
            matrix[(3, 2)], -matrix[(3, 1)], 0.0,
        ]);
        #[rustfmt::skip]
        let t_phi = na::DVector::from_row_slice(&[
            
        ]);
    }
}
