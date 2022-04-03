use anyhow::{Context, Result};
use nalgebra as na;

use crate::{
    linalg::{get_rotation_matrix_from_omega, get_zero_mat, matrix::reordered_svd},
    optimizer::ObservedData,
};

use super::fundamental_matrix::FundamentalMatrixData;

const MAX_ITERATION: usize = 10;

fn sampson_error(data_container: &FundamentalMatrixData, matrix: &na::DMatrix<f64>) -> f64 {
    let params = na::DVector::from_row_slice(&[
        matrix[(0, 0)],
        matrix[(0, 1)],
        matrix[(0, 2)],
        matrix[(1, 0)],
        matrix[(1, 1)],
        matrix[(1, 2)],
        matrix[(2, 0)],
        matrix[(2, 1)],
        matrix[(2, 2)],
    ]);
    (0..data_container.len()).fold(0.0, |acc, idx| {
        let xi = data_container.vector(idx);
        let var_mat = data_container.variance(idx);
        acc + ((xi.transpose() * &params)[(0, 0)].powi(2)
            / (params.transpose() * var_mat * &params)[(0, 0)])
            .abs()
    }) / data_container.len() as f64
}

/// Fundamental matrix optimization.
/// `matrix` is 3x3 matrix of rank 3. (rank of the matrix is not corrected.)
pub fn latent_variable_method(
    data: &[na::Point2<f64>],
    matrix: na::DMatrix<f64>,
) -> Result<na::DMatrix<f64>> {
    let data_container = FundamentalMatrixData::new(data);

    println!(
        "Sampson error before rank correction : {}",
        sampson_error(&data_container, &matrix)
    );
    // rank correction by svd decomposition
    let (mut u, mut diag, mut v) = reordered_svd(matrix)?;
    diag[2] = 0.0;
    let phi = (diag[0] / (diag[0] * diag[0] + diag[1] * diag[1]).sqrt()).acos();
    diag[0] = phi.cos();
    diag[1] = phi.sin();
    let mut matrix = &u * na::DMatrix::from_diagonal(&diag) * v.transpose();
    println!(
        "Sampson error after SVD rank correction : {}",
        sampson_error(&data_container, &matrix)
    );

    let mut j = sampson_error(&data_container, &matrix);
    let mut c = 1e-4;

    // LM optimization
    for tmp_j in 0..MAX_ITERATION {
        #[rustfmt::skip]
        let f_u = na::DMatrix::from_row_slice(9, 3, &[
            0.0, matrix[(2, 0)], -matrix[(1, 0)],
            0.0, matrix[(2, 1)], -matrix[(1, 1)],
            0.0, matrix[(2, 2)], -matrix[(1, 2)],
            -matrix[(2, 0)], 0.0, matrix[(0, 0)],
            -matrix[(2, 1)], 0.0, matrix[(0, 1)],
            -matrix[(2, 2)], 0.0, matrix[(0, 2)],
            matrix[(1, 0)], -matrix[(0, 0)], 0.0,
            matrix[(1, 1)], -matrix[(0, 1)], 0.0,
            matrix[(1, 2)], -matrix[(0, 2)], 0.0,
        ]);
        #[rustfmt::skip]
        let f_v = na::DMatrix::from_row_slice(9, 3, &[
            0.0, matrix[(0, 2)], -matrix[(0, 1)],
            -matrix[(0, 2)], 0.0, matrix[(0, 0)],
            matrix[(0, 1)], -matrix[(0, 0)], 0.0,
            0.0, matrix[(1, 2)], -matrix[(1, 1)],
            -matrix[(1, 2)], 0.0, matrix[(1, 0)],
            matrix[(1, 1)], -matrix[(1, 0)], 0.0,
            0.0, matrix[(2, 2)], -matrix[(2, 1)],
            -matrix[(2, 2)], 0.0, matrix[(2, 0)],
            matrix[(2, 1)], -matrix[(2, 0)], 0.0,
        ]);
        #[rustfmt::skip]
        let t_phi = na::DVector::from_row_slice(&[
            diag[0] * u[(0, 1)] * v[(0, 1)] - diag[1] * u[(0, 0)] * v[(0, 0)],
            diag[0] * u[(0, 1)] * v[(1, 1)] - diag[1] * u[(0, 0)] * v[(1, 0)],
            diag[0] * u[(0, 1)] * v[(2, 1)] - diag[1] * u[(0, 0)] * v[(2, 0)],
            diag[0] * u[(1, 1)] * v[(0, 1)] - diag[1] * u[(1, 0)] * v[(0, 0)],
            diag[0] * u[(1, 1)] * v[(1, 1)] - diag[1] * u[(1, 0)] * v[(1, 0)],
            diag[0] * u[(1, 1)] * v[(2, 1)] - diag[1] * u[(1, 0)] * v[(2, 0)],
            diag[0] * u[(2, 1)] * v[(0, 1)] - diag[1] * u[(2, 0)] * v[(0, 0)],
            diag[0] * u[(2, 1)] * v[(1, 1)] - diag[1] * u[(2, 0)] * v[(1, 0)],
            diag[0] * u[(2, 1)] * v[(2, 1)] - diag[1] * u[(2, 0)] * v[(2, 0)],
        ]);

        let params = na::DVector::<f64>::from_row_slice(&[
            matrix[(0, 0)],
            matrix[(0, 1)],
            matrix[(0, 2)],
            matrix[(1, 0)],
            matrix[(1, 1)],
            matrix[(1, 2)],
            matrix[(2, 0)],
            matrix[(2, 1)],
            matrix[(2, 2)],
        ]);
        let params_t = params.transpose();
        let m =
            (0..data_container.len()).fold(get_zero_mat(data_container.vec_size()), |acc, idx| {
                let xi = data_container.vector(idx);
                let var_mat = data_container.variance(idx);
                acc + &xi * xi.transpose() / (&params_t * var_mat * &params)[(0, 0)]
            }) / data_container.len() as f64;
        let l =
            (0..data_container.len()).fold(get_zero_mat(data_container.vec_size()), |acc, idx| {
                let xi = data_container.vector(idx);
                let var_mat = data_container.variance(idx);
                let nume = ((params.transpose() * xi)[(0, 0)]).powi(2);
                let denomi = ((params.transpose() * &var_mat * &params)[(0, 0)]).powi(2);
                acc + nume / denomi * &var_mat
            }) / data_container.len() as f64;
        let x = m - l;

        // first-order derivatives
        let du = 2.0 * f_u.transpose() * &x * &params;
        let dv = 2.0 * f_v.transpose() * &x * &params;
        let dp = 2.0 * t_phi.transpose() * &x * &params;
        // second-order derivatives
        let duu = 2.0 * f_u.transpose() * &x * &f_u;
        let dvv = 2.0 * f_v.transpose() * &x * &f_v;
        let duv = 2.0 * f_u.transpose() * &x * &f_v;
        let dpp = 2.0 * t_phi.transpose() * &x * &t_phi;
        let dup = 2.0 * f_u.transpose() * &x * &t_phi;
        let dvp = 2.0 * f_v.transpose() * &x * &t_phi;

        // hessian matrix
        #[rustfmt::skip]
        let h = na::DMatrix::from_row_slice(7, 7, &[
            duu[(0, 0)], duu[(0, 1)], duu[(0, 2)], duv[(0, 0)], duv[(0, 1)], duv[(0, 2)], dup[(0, 0)],
            duu[(1, 0)], duu[(1, 1)], duu[(1, 2)], duv[(1, 0)], duv[(1, 1)], duv[(1, 2)], dup[(1, 0)],
            duu[(2, 0)], duu[(2, 1)], duu[(2, 2)], duv[(2, 0)], duv[(2, 1)], duv[(2, 2)], dup[(2, 0)],
            duv[(0, 0)], duv[(1, 0)], duv[(2, 0)], dvv[(0, 0)], dvv[(0, 1)], dvv[(0, 2)], dvp[(0, 0)],
            duv[(0, 1)], duv[(1, 1)], duv[(2, 1)], dvv[(1, 0)], dvv[(1, 1)], dvv[(1, 2)], dvp[(1, 0)],
            duv[(0, 2)], duv[(1, 2)], duv[(2, 2)], dvv[(2, 0)], dvv[(2, 1)], dvv[(2, 2)], dvp[(2, 0)],
            dup[(0, 0)], dup[(1, 0)], dup[(2, 0)], dvp[(0, 0)], dvp[(1, 0)], dvp[(2, 0)], dpp[(0, 0)],
        ]);
        let dh = na::DMatrix::from_diagonal(&h.diagonal());
        #[rustfmt::skip]
        let b = - na::DVector::from_row_slice(&[
            du[0], du[1], du[2], dv[0], dv[1], dv[2], dp[0]
        ]);

        let mut f_hat = na::DMatrix::<f64>::from_element(0, 0, 0.0);
        let mut u_hat = na::DMatrix::<f64>::from_element(0, 0, 0.0);
        let mut v_hat = na::DMatrix::<f64>::from_element(0, 0, 0.0);
        let mut p_hat = 0.0;
        for tmp_i in 0..5 {
            let delta = (&h + c * &dh)
                .lu()
                .solve(&b)
                .context("Failed to LU decomposition")?;
            u_hat = get_rotation_matrix_from_omega(&[delta[0], delta[1], delta[2]]) * &u;
            v_hat = get_rotation_matrix_from_omega(&[delta[3], delta[4], delta[5]]) * &v;
            p_hat = phi + delta[6];
            f_hat = &u_hat
                * na::DMatrix::from_diagonal(&na::DVector::<f64>::from_row_slice(&[
                    p_hat.cos(),
                    p_hat.sin(),
                    0.0,
                ]))
                * v_hat.transpose();

            let j_hat = sampson_error(&data_container, &f_hat);
            if j_hat < j * 1.001 {
                {
                    println!(
                        "i = {}, j_hat = {}, c = {}, delta = {:.3}, {:.3}, {:.3}, {:.3}, {:.3}, {:.3}, {:.3}",
                        tmp_i, j_hat, c, delta[0], delta[1], delta[2], delta[3], delta[4], delta[5], delta[6]
                    );
                }
                if (&matrix - &f_hat).lp_norm(2) < 1e-3 {
                    println!("Finish at loop = {:}", tmp_j);
                    return Ok(matrix);
                }
                j = j_hat;
                matrix = f_hat;
                u = u_hat;
                v = v_hat;
                diag[0] = p_hat.cos();
                diag[1] = p_hat.sin();
                break;
            }
            println!("i = {}, j_hat = {}", tmp_i, j_hat);
            c *= 10.0;
        }
        c /= 10.0;
    }
    Ok(matrix)
}

#[cfg(test)]
mod tests {
    use crate::{
        epipolar::fundamental_matrix::tests::{
            assert_result, create_test_data, create_test_data_with_params,
        },
        optimizer::{least_square::least_square_fitting, taubin::taubin},
    };

    use super::*;

    #[test]
    fn test_latent_variable_method() {
        let (_, data) = create_test_data_with_params(0.1);
        // let res = taubin::<FundamentalMatrixData>(&data).unwrap();
        let res = least_square_fitting::<FundamentalMatrixData>(&data).unwrap();
        let res = latent_variable_method(&data, na::DMatrix::from_row_slice(3, 3, res.as_slice()))
            .unwrap();
        let r = assert_result(na::DVector::from_fn(9, |i, _| res[(i / 3, i % 3)]), data);
        assert!(r < 1e-1, "res = {}", r);
    }
}
