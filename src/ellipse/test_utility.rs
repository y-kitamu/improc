//! Test utility functions (only build with test code)

/// Assert if two float values are almost same.
#[macro_export]
macro_rules! assert_eq_float {
    ($lhs:expr, $rhs: expr) => {{
        assert!(($lhs - $rhs).abs() < 1e-5);
    }};
    ($lhs:expr, $rhs: expr, $($args:tt)+) => {{
        assert!(($lhs - $rhs).abs() < 1e-5, $($args)+)
    }};
}

#[cfg(test)]
pub mod test_util {
    use nalgebra as na;

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

    /// Normalize `vec` to |`vec`| = 1
    pub fn normalize(vec: &[f64]) -> Vec<f64> {
        let sum: f64 = vec.iter().map(|x| x * x).sum();
        vec.iter().map(|val| val / sum.sqrt()).collect()
    }

    /// Compare `true_vec` and `pred_vec` without sign.
    pub fn compare_vecs_without_sign(true_vec: &[f64], pred_vec: &[f64], threshold: f64) {
        assert_eq!(true_vec.len(), pred_vec.len());

        let mut pvec = na::DVector::from_row_slice(pred_vec);
        if true_vec[0] * pvec[0] < 0.0 {
            pvec *= -1.0;
        }
        true_vec.iter().zip(pvec.iter()).for_each(|(tval, pval)| {
            assert!(
                (tval - pval).abs() < threshold,
                "tval = {:?}, pval = {:?}",
                tval,
                pval
            );
        })
    }

    /// Assert if two matrices `true_mat` and `pred_mat` have same elements.
    pub fn compare_matrix(true_mat: &na::DMatrix<f64>, pred_mat: &na::DMatrix<f64>) {
        assert_eq!(true_mat.nrows(), pred_mat.nrows());
        assert_eq!(true_mat.ncols(), pred_mat.ncols());

        (0..true_mat.nrows()).for_each(|r| {
            (0..true_mat.nrows()).for_each(|c| assert_eq_float!(true_mat[(r, c)], pred_mat[(r, c)]))
        });
    }

    ///
    pub fn compare_vector(true_vec: &na::DVector<f64>, pred_vec: &na::DVector<f64>) {
        assert_eq!(true_vec.nrows(), pred_vec.nrows());
        true_vec
            .as_slice()
            .iter()
            .zip(pred_vec.as_slice().iter())
            .for_each(|(lhs, rhs)| assert_eq_float!(lhs, rhs));
    }
}
