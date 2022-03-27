//! Functions for fitting data points to ellipse using least-square method.
#[cfg(test)]
mod tests {
    use crate::{
        ellipse::{
            test_utility::test_util::{calc_residual, compare_vecs_without_sign, normalize},
            EllipseData,
        },
        optimizer::least_square::least_square_fitting,
    };

    use nalgebra as na;
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

        let params = least_square_fitting::<EllipseData>(&points).unwrap();
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

        let params = least_square_fitting::<EllipseData>(&points).unwrap();
        println!("params = {:?}", params);
        println!("ans = {:?}", ans);
        compare_vecs_without_sign(&ans, params.as_slice(), 1e-5);
    }
}
