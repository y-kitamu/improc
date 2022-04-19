use std::cmp::PartialOrd;

use anyhow::{Context, Result};
use nalgebra as na;

use crate::linalg::get_identity_mat;

/// self calibration (calculate camera pose) using homography.
/// - `homography_mat`
/// - `data` : observed data.
/// - `focal_length0` : focal length of the first camera.
/// - `focal_length1` : focal length of the second camera.
/// - `f0` : scale constant.
pub fn plane_self_calibration(
    homography_mat: &na::DMatrix<f64>,
    focal_length0: f64,
    focal_length1: f64,
    f0: f64,
) -> Result<Vec<(na::DMatrix<f64>, na::DVector<f64>)>> {
    let fl = na::DMatrix::from_diagonal(&na::DVector::from_vec(vec![f0, f0, focal_length1]));
    let fr = na::DMatrix::from_diagonal(&na::DVector::from_vec(vec![
        1.0 / f0,
        1.0 / f0,
        1.0 / focal_length0,
    ]));
    let h_hat: na::DMatrix<f64> = fl * homography_mat * fr;
    let det = h_hat.determinant();
    let h_hat = h_hat * det.powf(-1.0 / 3.0);

    let mut svd = h_hat.clone().svd(true, true);
    let sings = svd.singular_values.as_mut_slice();
    sings.sort_by(|lhs, rhs| rhs.partial_cmp(lhs).unwrap());
    let v: na::DMatrix<f64> = svd.v_t.context("Failed to calc svd.")?.transpose();

    let coeff0 = (sings[0].powi(2) - sings[1].powi(2)).powf(0.5);
    let coeff1 = (sings[1].powi(2) - sings[2].powi(2)).powf(0.5);
    let h = sings[1] / (sings[0] - sings[2]);
    let n0 = (coeff0 * v.column(0) + coeff1 * v.column(2)).normalize();
    let n1 = (coeff0 * v.column(0) - coeff1 * v.column(2)).normalize();
    let ns = vec![n0.clone(), n1.clone(), -n0, -n1];
    let t0 = (-sings[2] * coeff0 * v.column(0) + sings[0] * coeff1 * v.column(2)).normalize();
    let t1 = (-sings[2] * coeff0 * v.column(0) - sings[0] * coeff1 * v.column(2)).normalize();
    let ts = vec![t0.clone(), t1.clone(), -t0, -t1];
    let rts: Vec<(na::DMatrix<f64>, na::DVector<f64>)> = ns
        .iter()
        .zip(ts.iter())
        .map(|(n, t)| {
            let s2 = sings[1];
            let s23 = sings[1].powi(3);
            let r =
                1.0 / s2 * (get_identity_mat(3) + s23 * n * t.transpose() / h) * h_hat.transpose();
            (r, t.clone())
        })
        .collect();
    Ok(rts)
}

#[cfg(test)]
mod tests {
    use crate::{
        linalg::{get_rotation_matrix_from_omega, matrix::pseudo_inverse},
        PrintDebug,
    };

    use super::*;
    use rand::Rng;

    fn calc_homography(
        rot: &na::DMatrix<f64>,
        trans: &na::DVector<f64>,
        a: f64,
        b: f64,
        c: f64,
    ) -> na::DMatrix<f64> {
        #[rustfmt::skip]
        let C = na::DMatrix::from_row_slice(4, 3, &[
            1.0, 0.0, 0.0,
            0.0, 1.0, 0.0,
            a, b, c,
            0.0, 0.0, 1.0
        ]);
        #[rustfmt::skip]
        let p0 = na::DMatrix::from_row_slice(3, 4, &[
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
        ]);
        let rtt = -rot.transpose() * trans;
        #[rustfmt::skip]
        let p1 = na::DMatrix::from_row_slice(3, 4, &[
            rot[(0, 0)], rot[(1, 0)], rot[(2, 0)], rtt[0],
            rot[(0, 1)], rot[(1, 1)], rot[(2, 1)], rtt[1],
            rot[(0, 2)], rot[(1, 2)], rot[(2, 2)], rtt[2],
        ]);
        let homography = p1 * &C * pseudo_inverse(&(p0 * &C)).unwrap();
        homography
    }

    #[test]
    fn test_plane_self_calibration() {
        let trial = 100;
        let success: usize = (0..trial)
            .map(|_| {
                let mut rng = rand::thread_rng();
                let mut random_rot = || rng.gen::<f64>() * std::f64::consts::PI * 2.0;
                let rot =
                    get_rotation_matrix_from_omega(&[random_rot(), random_rot(), random_rot()]);
                let trans = na::DVector::from_vec(vec![
                    rng.gen::<f64>(),
                    rng.gen::<f64>(),
                    rng.gen::<f64>(),
                ])
                .normalize();

                // let (a, b, c) = (rng.gen::<f64>(), rng.gen::<f64>(), rng.gen::<f64>());
                let (a, b, c) = (1.0, 1.0, 1.0);
                let homography = calc_homography(&rot, &trans, a, b, c);
                let res = plane_self_calibration(&homography, 1.0, 1.0, 1.0).unwrap();

                let min = (res.iter().map(|(r, t)| {
                    assert!((r.determinant() - 1.0) < 1e-2);
                    assert!((t.norm() - 1.0) < 1e-2);
                    let homo = calc_homography(r, t, a, b, c).normalize();
                    let error = homo - homography.normalize();
                    error.norm()
                }))
                .min_by(|x, y| x.partial_cmp(y).unwrap())
                .unwrap();
                // println!("error = {}", min);
                (min < 1e-1) as usize
            })
            .sum();
        println!("success / trial = {} / {}", success, trial);
        assert!(success as f64 > trial as f64 * 0.7);
    }
}
