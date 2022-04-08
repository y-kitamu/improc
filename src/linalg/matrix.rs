//! Solver for eigenvalue problems
use anyhow::{ensure, Context, Result};
use nalgebra as na;

use crate::linalg::get_identity_mat;

/// calculate least square solution of linear equation.
/// Find x which minimize |Ax - b|.
pub fn le_lstsq(matrix: &na::DMatrix<f64>, params: &na::DVector<f64>) -> Result<na::DVector<f64>> {
    Ok(pseudo_inverse(matrix).context("Failed to calc pseudo inverse.")? * params)
}

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
        matrix.ncols() == constrained.ncols(),
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
        if sing_vals[i].abs() < 1e-15 {
            a_hat2_vec.push(a_hat.column(i).clone_owned());
        } else {
            a_hat1_vec.push(a_hat.column(i).clone_owned());
            diag.push(sing_vals[i]);
        }
    }
    ensure!(
        !a_hat1_vec.is_empty(),
        "Invalid value : a_hat1_vec is empty."
    );
    let d1_inv: na::DMatrix<f64> = na::Matrix::from_diagonal(&na::DVector::from_vec(
        diag.iter().map(|val| 1.0 / val).collect(),
    ));
    let a_hat1: na::DMatrix<f64> = na::Matrix::from_columns(&a_hat1_vec);
    // If a_hat2 is empty, objective is minimizing |A_hat1 * x_hat| subject to |x_hat| = 1.
    if a_hat2_vec.is_empty() {
        let x_hat = lstsq(&a_hat1)?;
        return Ok(v_t.transpose() * x_hat);
    }

    let a_hat2: na::DMatrix<f64> = na::Matrix::from_columns(&a_hat2_vec);
    let a_hat2_inv = pseudo_inverse(&a_hat2).context("Failed to calculate pseudo inverse.")?;
    // A'' = (A'_2 * A'_2^+ - I) * A'_1 D_1^-1
    let a_hhat: na::DMatrix<f64> = (a_hat2 * a_hat2_inv.clone() - get_identity_mat(matrix.nrows()))
        * a_hat1.clone()
        * d1_inv.clone();
    let x_hhat: na::DVector<f64> = lstsq(&a_hhat)?;
    let x1_hat: na::DVector<f64> = d1_inv * x_hhat;
    let x2_hat: na::DVector<f64> = -a_hat2_inv * a_hat1 * x1_hat.clone();
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
        * svd.u.context("Failed to get SVD value")?.transpose())
}

/// apply SVD decomposition to `matrix`.
/// Rows or columns of the resulting matrices is ordered by singular value.
pub fn reordered_svd(
    matrix: na::DMatrix<f64>,
) -> Result<(na::DMatrix<f64>, na::DVector<f64>, na::DMatrix<f64>)> {
    let svd = matrix.svd(true, true);
    let singular_values = svd.singular_values.as_slice();
    let mut indices: Vec<usize> = (0..singular_values.len()).collect();
    indices.sort_by(|&lhs, &rhs| {
        singular_values[rhs]
            .partial_cmp(&singular_values[lhs])
            .unwrap()
    });
    let diag = na::DVector::<f64>::from_iterator(
        indices.len(),
        indices.iter().map(|&idx| singular_values[idx]),
    );
    let u: na::DMatrix<f64> = svd.u.context("Failed to calc svd.")?;
    let u = na::DMatrix::<f64>::from_fn(u.nrows(), u.ncols(), |r, c| u[(r, indices[c])]);
    let v_t: na::DMatrix<f64> = svd.v_t.context("Failed to calc svd.")?;
    let v = na::DMatrix::<f64>::from_fn(v_t.ncols(), v_t.nrows(), |r, c| v_t[(c, indices[r])]);
    Ok((u, diag, v))
}

#[cfg(test)]
mod tests {
    use crate::ellipse::test_utility::test_util::{compare_matrix, compare_vector};

    use super::*;

    #[test]
    fn test_le_lstsq() {
        #[rustfmt::skip]
        let mat = na::DMatrix::from_row_slice(4, 3, &[
            3.0, 2.0, 4.0,
            -1.0, 1.0, 1.0,
            1.0, 1.0, -1.0,
            1.0, 1.0, -1.0,
        ]);
        let b = na::DVector::from_vec(vec![28.0, 5.0, 1.0, 1.0]);
        let ans = le_lstsq(&mat, &b).unwrap();
        assert!((ans[0] - 2.0).abs() < 1e-5);
        assert!((ans[1] - 3.0).abs() < 1e-5);
        assert!((ans[2] - 4.0).abs() < 1e-5);
    }

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

    #[test]
    fn test_constrained_lstsq() {
        // identity matrix case (normal eigenvalue problem)
        let matrix = na::DMatrix::<f64>::from_diagonal(&na::DVector::from_row_slice(&[
            5.0, 4.0, 3.0, 2.0, 1.0,
        ]));
        let constrained = na::DMatrix::<f64>::identity(5, 5);
        let res = constrained_lstsq(&matrix, &constrained).unwrap();
        compare_vector(
            &na::DVector::<f64>::from_vec(vec![0.0, 0.0, 0.0, 0.0, 1.0]),
            &res,
        );

        //  identity matrix case (2)
        let matrix = na::DMatrix::<f64>::from_diagonal(&na::DVector::from_row_slice(&[
            5.0, 4.0, 3.0, 2.0, 0.0,
        ]));
        let res = constrained_lstsq(&matrix, &constrained).unwrap();
        compare_vector(
            &na::DVector::<f64>::from_vec(vec![0.0, 0.0, 0.0, 0.0, 1.0]),
            &res,
        );
    }

    #[test]
    fn test_reorder_svd() {
        let mat =
            na::DMatrix::from_row_slice(3, 3, &[1.0, 3.0, 2.0, -1.0, 0.0, 1.0, 2.0, 3.0, 0.0]);
        let (u, d, v) = reordered_svd(mat.clone()).unwrap();

        let res = u * na::DMatrix::from_diagonal(&d) * v.transpose();
        println!("{:?}", d);
        println!("{:?}", mat);
        println!("{:?}", res);
        compare_matrix(&mat, &res);
    }
}
