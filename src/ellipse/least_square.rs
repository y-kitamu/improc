//! Functions for fitting data points to ellipse using least-square method.
use anyhow::{ensure, Context, Result};
use nalgebra as na;

/// Fit given `data` points to ellipse by least square method.
pub fn least_square_fitting(data: &[na::Point2<f64>], scale: f64) -> Result<na::Vector6<f64>> {
    ensure!(
        data.len() >= 5,
        format!("Data point must be 5 or more, not {}", data.len())
    );

    let mat: na::Matrix6<f64> = data
        .iter()
        .fold(na::Matrix6::<f64>::zeros(), |mut acc, pt| {
            let x = pt[0];
            let y = pt[1];
            let xi = na::Vector6::new(
                x * x,
                2.0 * x * y,
                y * y,
                2.0 * scale * x,
                2.0 * scale * y,
                scale * scale,
            );
            acc += xi * xi.transpose();
            acc
        })
        / data.len() as f64;
    let svd = mat.svd(false, true);
    let v_t: na::SMatrix<f64, 6, 6> = svd.v_t.context("Failed to get SVD value")?;
    let (row, _) = svd.singular_values.argmin();
    Ok(v_t.row(row).transpose().clone_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let ans = [1.0, 0.0, 1.0, 0.0, 0.0, -1.0];
        let sum: f64 = ans.iter().map(|x| x * x).sum();
        let ans = ans.iter().map(|val| val / sum.sqrt()).collect::<Vec<f64>>();

        let mut rng = rand::thread_rng();
        let points: Vec<na::Point2<f64>> = (0..1000)
            .map(|_| {
                let rad: f64 = rng.gen::<f64>() * std::f64::consts::PI * 2.0;
                na::Point2::new(rad.cos(), rad.sin())
            })
            .collect();
        points.iter().for_each(|p| {
            let x = p[0];
            let y = p[1];
            let val = ans[0] * x * x
                + 2.0 * ans[1] * x * y
                + ans[2] * y * y
                + 2.0 * (ans[3] * x + ans[4] * y)
                + ans[5];
            assert!(val.abs() < 1e-7, "val = {}", val);
        });

        let mut params = least_square_fitting(&points, 1.0).unwrap();
        if ans[0] * params[0] < 0.0 {
            params *= -1.0;
        }

        assert_eq!(params.len(), 6);
        ans.iter().zip(params.iter()).for_each(|(tval, pval)| {
            assert!(
                (tval - pval).abs() < 1e-5,
                "tvals = {:?}, pvals = {:?}",
                ans,
                params
            );
        })
    }

    #[test]
    fn lsm_fit() {
        // x^2 + 4 * y^2 - 4 = 0
        let ans = [1.0, 0.0, 4.0, 0.0, 0.0, -4.0];
        let sum: f64 = ans.iter().map(|x| x * x).sum();
        let ans = ans.iter().map(|val| val / sum.sqrt()).collect::<Vec<f64>>();
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
            let x = p[0];
            let y = p[1];
            let val = ans[0] * x * x
                + 2.0 * ans[1] * x * y
                + ans[2] * y * y
                + 2.0 * (ans[3] * x + ans[4] * y)
                + ans[5];
            assert!(val.abs() < 1e-7, "val = {}", val);
        });

        let mut params = least_square_fitting(&points, 1.0).unwrap();
        if ans[0] * params[0] < 0.0 {
            params *= -1.0;
        }

        assert_eq!(params.len(), 6);
        ans.iter().zip(params.iter()).for_each(|(tval, pval)| {
            assert!(
                (tval - pval).abs() < 1e-5,
                "tvals = {:?}, pvals = {:?}",
                ans,
                params
            );
        })
    }
}
