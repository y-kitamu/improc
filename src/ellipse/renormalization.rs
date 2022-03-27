//! Implementation of re-normalization method.
//! Re-normalization method is algorithm for fitting data points to ellipse.
#[cfg(test)]
mod tests {
    use crate::{
        ellipse::{
            test_utility::test_util::{compare_vecs_without_sign, normalize},
            EllipseData,
        },
        optimizer::taubin::renormalization,
    };

    use nalgebra as na;
    use rand::prelude::*;

    #[test]
    fn test_renormalization() {
        // x^2 + y^2 - 1 = 0
        let ans = normalize(&[1.0, 0.0, 1.0, 0.0, 0.0, -1.0]);
        let std_dev = 0.05;
        let mut rng = rand::thread_rng();
        let points: Vec<na::Point2<f64>> = (0..1000)
            .map(|_| {
                let rad: f64 = rng.gen::<f64>() * std::f64::consts::PI * 2.0;
                let dx = (rng.gen::<f64>() - 0.5) * std_dev;
                let dy = (rng.gen::<f64>() - 0.5) * std_dev;
                na::Point2::new(rad.cos() + dx, rad.sin() + dy)
            })
            .collect();

        let pred = renormalization::<EllipseData>(&points).unwrap();
        let normed = normalize(pred.as_slice());
        compare_vecs_without_sign(&ans, &normed, 1e-2);
    }
}
