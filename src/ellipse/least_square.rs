//! Functions for fitting data points to ellipse using least-square method.
#[cfg(test)]
mod tests {
    use crate::{
        ellipse::{
            test_utility::test_util::{calc_residual, compare_vecs_without_sign, normalize},
            EllipseData,
        },
        optimizer::least_square::iterative_reweight,
        optimizer::least_square::least_square_fitting,
    };

    use nalgebra as na;
    use rand::Rng;

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

    #[test]
    fn test_iterative_reweight() {
        // x^2 + y^2 - 1 = 0
        let ans = normalize(&[1.0, 0.0, 1.0, 0.0, 0.0, -1.0]);
        let std_dev = 0.05;
        let mut rng = rand::thread_rng();

        let std_dev = 0.05;

        let mut rng = rand::thread_rng();
        // let mut rng = ChaCha20Rng::seed_from_u64(2);
        for _ in 0..100 {
            // create answer
            let a = rng.gen::<f64>() + 0.5;
            let c = rng.gen::<f64>() + 0.5;
            let d = rng.gen::<f64>();
            let e = rng.gen::<f64>();
            let ans = normalize(&[a, 0.0, c, d, e, -1.0]);

            // create input points
            let radius = (d * d / a + e * e / c + 1.0).sqrt();
            let points: Vec<na::Point2<f64>> = (0..1000)
                .map(|_| {
                    let rad: f64 = rng.gen::<f64>() * std::f64::consts::PI * 2.0;
                    let dx = (rng.gen::<f64>() - 0.5) * std_dev;
                    let dy = (rng.gen::<f64>() - 0.5) * std_dev;
                    let x = rad.cos() * radius / a.sqrt() - d / a;
                    let y = rad.sin() * radius / c.sqrt() - e / c;
                    na::Point2::new(x + dx, y + dy)
                })
                .collect();

            // pred & eval
            let pred = iterative_reweight::<EllipseData>(&points).unwrap();
            let normed = normalize(pred.as_slice());
            compare_vecs_without_sign(&ans, &normed, 1e-2);
        }
    }
}
