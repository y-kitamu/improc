#[cfg(test)]
mod tests {
    use crate::{
        ellipse::{
            test_utility::test_util::{compare_vecs_without_sign, normalize},
            EllipseData,
        },
        optimizer::taubin::renormalization,
        optimizer::taubin::taubin,
    };

    use nalgebra as na;
    use rand::Rng;

    #[test]
    fn test_taubin() {
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

        let params = taubin::<EllipseData>(&points).unwrap();
        let normed = normalize(params.as_slice());
        println!("{:?}", normed);
        compare_vecs_without_sign(&ans, &normed, 1e-5);
    }

    #[test]
    fn test_renormalization() {
        // x^2 + y^2 - 1 = 0
        let ans = normalize(&[1.0, 0.0, 1.0, 0.0, 0.0, -1.0]);
        let std_dev = 0.05;

        let mut rng = rand::thread_rng();
        // let mut rng = ChaCha20Rng::seed_from_u64(2);
        for _ in 0..100 {
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
}
