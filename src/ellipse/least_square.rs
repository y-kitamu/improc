//! Functions for fitting data points to ellipse using least-square method.

use nalgebra as na;

pub fn least_square_fitting(data: &[na::Point2<f64>]) -> Vec<f64> {
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lsm_fit() {
        let scale = 2.0f64.sqrt();
        let points = vec![
            na::Point2::new(2.0 * scale, 0.0),
            na::Point2::new(0.0, 1.0 * scale),
            na::Point2::new(-2.0 * scale, 0.0),
            na::Point2::new(0.0, -1.0 * scale),
        ];

        let params = least_square_fitting(&points);
        assert_eq!(params.len(), 6);
    }
}
