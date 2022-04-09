use anyhow::{ensure, Context, Result};
use nalgebra as na;

use crate::linalg::{scalar_triple_product, vector_cross_matrix};

/// Self-calibration from two image.
/// `data` is observed points.
/// Order of observed points in `data` must be like: [p0_camera1, p0_camera2, p1_camera1, ...].
pub fn self_calibration(
    fund_mat: &na::DMatrix<f64>,
    data: &[na::Point2<f64>],
    f0: f64,
) -> Result<(na::DMatrix<f64>, na::DMatrix<f64>)> {
    let (f, f_hat) = calc_focal_lengths(fund_mat, 1.0)?;

    let fmat =
        na::DMatrix::from_diagonal(&na::DVector::from_vec(vec![1.0 / f0, 1.0 / f0, 1.0 / f]));
    let fhmat = na::DMatrix::from_diagonal(&na::DVector::from_vec(vec![
        1.0 / f0,
        1.0 / f0,
        1.0 / f_hat,
    ]));
    let essential_mat = fmat * fund_mat * fhmat;
    let (rot, trans) = calc_motion_params(&essential_mat, data, f, f_hat)?;

    // calculate camera matrix from `f`, `f_hat`, `rot` and `trans`.
    #[rustfmt::skip]
    let camera_matrix0 = na::DMatrix::from_row_slice(3, 4, &[
        f, 0.0, 0.0, 0.0,
        0.0, f, 0.0, 0.0,
        0.0, 0.0, f0, 0.0,
    ]);
    let rt = rot.transpose() * trans;
    #[rustfmt::skip]
    let camera_matrix1 = na::DMatrix::from_row_slice(3, 4, &[
        f_hat * rot[(0, 0)], f_hat * rot[(1, 0)], f_hat * rot[(2, 0)], f_hat * -rt[0],
        f_hat * rot[(0, 1)], f_hat * rot[(1, 1)], f_hat * rot[(2, 1)], f_hat * -rt[1],
        f0 * rot[(0, 2)],    f0 * rot[(1, 2)],    f0 * rot[(2, 2)],    f0 * -rt[2],
    ]);
    Ok((camera_matrix0, camera_matrix1))
}

/// calculate focal lengths.
/// Return value is tuple of focal lengths of (first camera, second camera).
fn calc_focal_lengths(fund_mat: &na::DMatrix<f64>, f0: f64) -> Result<(f64, f64)> {
    let fft = fund_mat * fund_mat.transpose();
    let ftf = fund_mat.transpose() * fund_mat;
    let e = get_minimum_eigenvector(&fft);
    let e_hat = get_minimum_eigenvector(&ftf);

    let k = na::DVector::from_vec(vec![0.0, 0.0, 1.0]);

    let fk: na::DVector<f64> = fund_mat * &k;
    let ftk: na::DVector<f64> = fund_mat.transpose() * &k;
    let kfk = k.dot(&fk);
    let ehat_k = e_hat.cross(&k).norm_squared();
    let ek = e.cross(&k).norm_squared();
    ensure!(kfk.abs() > 1e-7, "Failed to calculate focal length.");

    let xi_nume = fk.norm_squared() - k.dot(&(&fft * &fk)) * ehat_k / kfk;
    let xi_deno = ehat_k * ftk.norm_squared() - kfk * kfk;
    let xi = xi_nume / xi_deno;

    let eta_nume = ftk.norm_squared() - k.dot(&(&fft * &fk)) * ek / kfk;
    let eta_deno = ek * fk.norm_squared() - kfk * kfk;
    let eta = eta_nume / eta_deno;

    Ok((f0 / (1.0 + xi).sqrt(), f0 / (1.0 + eta).sqrt()))
}

fn calc_motion_params(
    essential_mat: &na::DMatrix<f64>,
    data: &[na::Point2<f64>],
    f: f64,
    f_hat: f64,
) -> Result<(na::DMatrix<f64>, na::DVector<f64>)> {
    let mut trans = get_minimum_eigenvector(&(essential_mat * essential_mat.transpose()));

    let n_pts = data.len() / 2;
    let sum: f64 = (0..n_pts)
        .map(|idx| {
            let c0 = data[idx * 2];
            let c1 = data[idx * 2 + 1];
            let x0 = na::DVector::from_vec(vec![c0[0] / f, c0[1] / f, 1.0]);
            let x1 = na::DVector::from_vec(vec![c1[0] / f_hat, c1[1] / f_hat, 1.0]);
            scalar_triple_product(&trans, &x0, &(essential_mat * &x1))
        })
        .sum();
    if sum < 0.0 {
        trans *= -1.0;
    }

    let k = -vector_cross_matrix(&trans) * essential_mat;
    let svd = k.svd(true, true);
    let u = svd.u.context("Failed to calc svd")?;
    let v_t = svd.v_t.context("Failed to calc svd")?;
    let det_uv = (&u * &v_t).determinant();
    let rot =
        &u * na::DMatrix::from_diagonal(&na::DVector::from_vec(vec![1.0, 1.0, det_uv])) * &v_t;
    Ok((rot, trans))
}

/// Get eigenvector of minimum eigenvalue.
/// `matrix` must be symmetric matrix.
fn get_minimum_eigenvector(matrix: &na::DMatrix<f64>) -> na::DVector<f64> {
    let eigen = matrix.clone().symmetric_eigen();
    let (idx, _) = eigen.eigenvalues.argmin();
    eigen.eigenvectors.column(idx).clone_owned()
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;
    use crate::PrintDebug;

    #[test]
    fn test_get_minimum_singular_value() {
        let matrix = na::DMatrix::from_vec(2, 2, vec![5.0, 2.0, 2.0, 2.0]);
        let vec = get_minimum_eigenvector(&matrix);
        let ans = na::DVector::from_vec(vec![-1.0, 2.0]).normalize();

        assert!((vec[0] - ans[0]).abs() < 1e-5);
        assert!((vec[1] - ans[1]).abs() < 1e-5);
    }

    #[test]
    fn test_calc_focal_length() {
        let mut rng = rand::thread_rng();
        let f0 = 1.0;
        let f = 2.0;
        let fh = 3.0;
        let theta: f64 = rng.gen::<f64>() * std::f64::consts::PI * 2.0;
        #[rustfmt::skip]
        let r = na::DMatrix::from_row_slice(3, 3, &[
            theta.cos(), 0.0, theta.sin(),
            0.0, 1.0, 0.0,
            -theta.sin(), 0.0, theta.cos(),
        ]);
        let t = na::DVector::from_vec(vec![1.0, 2.0, 3.0]);
        let fm = na::DMatrix::from_diagonal(&na::DVector::from_vec(vec![f0, f0, f]));
        let fhm = na::DMatrix::from_diagonal(&na::DVector::from_vec(vec![f0, f0, fh]));
        let tm = vector_cross_matrix(&t);
        let fund_mat = &fm * &tm * &r * &fhm;
        let (pf, pfh) = calc_focal_lengths(&fund_mat, f0).unwrap();
        assert!((pf - f).abs() < 1e-5);
        assert!((pfh - fh).abs() < 1e-5);
    }

    #[test]
    fn test_self_calibration() {
        let mut rng = rand::thread_rng();
        let f0 = 1.0;
        let f = 2.0;
        let fh = 3.0;
        let theta: f64 = rng.gen::<f64>() * std::f64::consts::PI * 2.0;
        #[rustfmt::skip]
        let r = na::DMatrix::from_row_slice(3, 3, &[
            theta.cos(), 0.0, theta.sin(),
            0.0, 1.0, 0.0,
            -theta.sin(), 0.0, theta.cos(),
        ]);
        let t = na::DVector::from_vec(vec![1.0, 2.0, 3.0]).normalize();
        {
            println!("rotation : ");
            r.print();
            println!("translation : {:?}", t.as_slice());
        }
        let rt = r.transpose() * &t;
        #[rustfmt::skip]
        let p0 = na::DMatrix::from_row_slice(3, 4, &[
            f, 0.0, 0.0, 0.0,
            0.0, f, 0.0, 0.0,
            0.0, 0.0, f0, 0.0,
        ]);
        #[rustfmt::skip]
        let p1 = na::DMatrix::from_row_slice(3, 4, &[
            fh * r[(0, 0)], fh * r[(1, 0)], fh * r[(2, 0)], fh * -rt[0],
            fh * r[(0, 1)], fh * r[(1, 1)], fh * r[(2, 1)], fh * -rt[1],
            f0 * r[(0, 2)], f0 * r[(1, 2)], f0 * r[(2, 2)], f0 * -rt[2],
        ]);
        {
            println!("GT : ");
            p0.print();
            println!("GT : ");
            p1.print();
        }

        let fm = na::DMatrix::from_diagonal(&na::DVector::from_vec(vec![f0, f0, f]));
        let fhm = na::DMatrix::from_diagonal(&na::DVector::from_vec(vec![f0, f0, fh]));
        let tm = vector_cross_matrix(&t);
        let fund_mat = &fm * &tm * &r * &fhm;

        let gdata: Vec<na::DVector<f64>> = (0..100)
            .map(|_| {
                let x: f64 = (rng.gen::<f64>() - 0.5) * 2.0;
                let y: f64 = (rng.gen::<f64>() - 0.5) * 2.0;
                let z: f64 = rng.gen::<f64>() * 2.0;
                na::DVector::from_vec(vec![x, y, z, 1.0])
            })
            .collect();
        let data: Vec<na::Point2<f64>> = gdata
            .iter()
            .map(|gx| {
                let x0 = &p0 * gx;
                let x1 = &p1 * gx;
                vec![
                    na::Point2::new(x0[0] / x0[2], x0[1] / x0[2]),
                    na::Point2::new(x1[0] / x1[2], x1[1] / x1[2]),
                ]
            })
            .flatten()
            .collect();
        let (pp0, pp1) = self_calibration(&fund_mat, &data, 1.0).unwrap();

        (0..100).for_each(|idx| {
            let x0 = &pp0 * &gdata[idx];
            let x1 = &pp1 * &gdata[idx];
            let cond = x0.dot(&(&fund_mat * &x1));
            assert!(cond.abs() < 1e-5, "cond = {}", cond);
        });
    }
}
