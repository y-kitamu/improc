use anyhow::Result;
use nalgebra as na;

use crate::linalg::matrix::pseudo_inverse_with_rank;

const MAX_ITER: usize = 50;

pub fn optimal_correction(
    p0: &na::DMatrix<f64>,
    p1: &na::DMatrix<f64>,
    p2: &na::DMatrix<f64>,
    x0: &na::Point2<f64>,
    x1: &na::Point2<f64>,
    x2: &na::Point2<f64>,
    f0: f64,
) -> Result<Vec<na::DVector<f64>>> {
    let mut error: f64 = 1e9;
    let x = na::DVector::from_vec(vec![x0[0] / f0, x0[1] / f0, 1.0]);
    let y = na::DVector::from_vec(vec![x1[0] / f0, x1[1] / f0, 1.0]);
    let z = na::DVector::from_vec(vec![x2[0] / f0, x2[1] / f0, 1.0]);

    let mut x_h = x.clone();
    let mut y_h = y.clone();
    let mut z_h = z.clone();

    let mut dx = na::DVector::from_vec(vec![0.0; 3]);
    let mut dy = na::DVector::from_vec(vec![0.0; 3]);
    let mut dz = na::DVector::from_vec(vec![0.0; 3]);

    let trifocal_tensor = calc_trifocal_tensor(p0, p1, p2);
    #[rustfmt::skip]
    let pk: Vec<na::DVector<f64>> = vec![
        na::DVector::from_vec(vec![1.0, 0.0, 0.0]),
        na::DVector::from_vec(vec![0.0, 1.0, 0.0]),
        na::DVector::from_vec(vec![0.0, 0.0, 0.0]),
    ];

    for _ in 0..MAX_ITER {
        let p = (0..3)
            .map(|idx| calc_t(&trifocal_tensor, &pk[idx], &y_h, &z_h))
            .collect::<Vec<na::DMatrix<f64>>>();
        let q = (0..3)
            .map(|idx| calc_t(&trifocal_tensor, &x_h, &pk[idx], &z_h))
            .collect::<Vec<na::DMatrix<f64>>>();
        let r = (0..3)
            .map(|idx| calc_t(&trifocal_tensor, &x_h, &y_h, &pk[idx]))
            .collect::<Vec<na::DMatrix<f64>>>();

        let c = na::DMatrix::from_columns(
            &(0..9)
                .map(|rs| {
                    let ir = rs / 3;
                    let is = rs % 3;
                    let vp =
                        na::DVector::from_vec(vec![p[0][(ir, is)], p[1][(ir, is)], p[2][(ir, is)]]);
                    let vq =
                        na::DVector::from_vec(vec![q[0][(ir, is)], q[1][(ir, is)], q[2][(ir, is)]]);
                    let vr =
                        na::DVector::from_vec(vec![r[0][(ir, is)], r[1][(ir, is)], r[2][(ir, is)]]);
                    let mat_pq = calc_t(&trifocal_tensor, &vp, &y_h, &z_h)
                        + calc_t(&trifocal_tensor, &x_h, &vq, &z_h)
                        + calc_t(&trifocal_tensor, &x_h, &y_h, &vr);
                    na::DVector::from_vec(mat_pq.transpose().as_slice().to_vec())
                })
                .collect::<Vec<na::DVector<f64>>>(),
        );
        let f = calc_t(&trifocal_tensor, &x_h, &y_h, &z_h)
            + calc_t(&trifocal_tensor, &dx, &y_h, &z_h)
            + calc_t(&trifocal_tensor, &x_h, &dy, &z_h)
            + calc_t(&trifocal_tensor, &x_h, &y_h, &dz);
        let c_inv = pseudo_inverse_with_rank(&c, 3)?;
        let lambda = c_inv * f;

        dx = na::DVector::from_fn(3, |idx, _| {
            na::DVector::from_row_slice(p[idx].transpose().as_slice()).dot(&lambda)
        });
        dy = na::DVector::from_fn(3, |idx, _| {
            na::DVector::from_row_slice(q[idx].transpose().as_slice()).dot(&lambda)
        });
        dz = na::DVector::from_fn(3, |idx, _| {
            na::DVector::from_row_slice(r[idx].transpose().as_slice()).dot(&lambda)
        });
        x_h = &x - &dx;
        y_h = &y - &dy;
        z_h = &z - &dz;

        let e = dx.norm_squared() + dy.norm_squared() + dz.norm_squared();
        if (e - error).abs() < 1e-3 {
            break;
        }
        error = e;
    }
    Ok(vec![x_h, y_h, z_h])
}

fn calc_trifocal_tensor(
    p0: &na::DMatrix<f64>,
    p1: &na::DMatrix<f64>,
    p2: &na::DMatrix<f64>,
) -> Vec<na::DMatrix<f64>> {
    let ps = vec![p0, p1, p2];
    (0..3)
        .map(|idx| {
            na::DMatrix::from_fn(3, 3, |r, c| {
                if r == c {
                    return 0.0;
                }
                let l0 = (idx + 1) % 3;
                let l1 = (idx + 2) % 3;
                #[rustfmt::skip]
                let mat = na::DMatrix::from_row_slice(4, 4, &[
                    ps[0][(l0, 0)], ps[0][(l0, 1)], ps[0][(l0, 2)], ps[0][(l0, 3)],
                    ps[0][(l1, 0)], ps[0][(l1, 1)], ps[0][(l1, 2)], ps[0][(l1, 3)],
                    ps[1][(r, 0)], ps[1][(r, 1)], ps[1][(r, 2)], ps[1][(r, 3)],
                    ps[2][(c, 0)], ps[2][(c, 1)], ps[2][(c, 2)], ps[2][(c, 3)],
                ]);
                mat.determinant()
            })
        })
        .collect()
}

fn calc_t(
    tri_tensor: &[na::DMatrix<f64>],
    x: &na::DVector<f64>,
    y: &na::DVector<f64>,
    z: &na::DVector<f64>,
) -> na::DMatrix<f64> {
    na::DMatrix::from_fn(3, 3, |r, c| {
        (0..3)
            .map(|idx| {
                let t = &tri_tensor[idx];
                let r1 = (r + 1) % 3;
                let r2 = (r + 2) % 3;
                let c1 = (c + 1) % 3;
                let c2 = (c + 2) % 3;
                x[idx]
                    * (t[(r1, c1)] * y[r2] * z[c2]
                        - t[(r2, c1)] * y[r1] * z[c2]
                        - t[(r1, c2)] * y[r2] * z[c1]
                        + t[(r2, c2)] * y[r1] * z[c1])
            })
            .sum::<f64>()
    })
}
