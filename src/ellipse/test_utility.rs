//! Test utility functions (only build with test code)

#[cfg(test)]
pub mod test_util {
    use nalgebra as na;

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
                (tval - pval).abs() < 1e-5,
                "tval = {:?}, pval = {:?}",
                tval,
                pval
            );
        })
    }
}
