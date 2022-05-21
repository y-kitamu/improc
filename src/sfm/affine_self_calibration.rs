use anyhow::{ensure, Context, Result};
use nalgebra as na;

use crate::{
    linalg::{get_zero_mat, matrix::lstsq},
    PrintDebug,
};

/// Self calibration using affine camera model.
/// - observed_pts : Observed points. (2d vector : [index of camera][index of point])
/// - Return : Tuple of (motion matrix (stacked camera matrices), shape matrix (stacked 3d points)).
pub fn affine_self_calibration(
    observed_pts: &[Vec<na::Point2<f64>>],
) -> Result<(na::DMatrix<f64>, na::DMatrix<f64>)> {
    ensure!(!observed_pts.is_empty(), "observed_pts must not be empty");
    let n_points = observed_pts[0].len();
    let n_cameras = observed_pts.len();

    let observation_matrix: na::DMatrix<f64> = na::DMatrix::from_row_slice(
        2 * n_cameras,
        n_points,
        &observed_pts
            .iter()
            .map(|pts| {
                let mut vec = pts.iter().map(|pt| pt.x).collect::<Vec<f64>>();
                vec.append(&mut pts.iter().map(|pt| pt.y).collect::<Vec<f64>>());
                vec
            })
            .flatten()
            .collect::<Vec<f64>>(),
    );
    let (motion_mat, shape_mat) = calc_motion_and_shape_matrix(&observation_matrix)?;

    // calculate metric condition (affine reconstruction)
    let bs: na::DMatrix<f64> = (0..n_cameras).fold(get_zero_mat(9), |acc, idx| {
        let (mut tx, mut ty) = observed_pts.iter().fold((0.0, 0.0), |(xsum, ysum), pts| {
            (xsum + pts[idx].x, ysum + pts[idx].y)
        });
        tx /= n_points as f64;
        ty /= n_points as f64;
        let a = tx * ty;
        let c = tx * tx - ty * ty;

        let u1 = motion_mat.row(idx * 2);
        let u2 = motion_mat.row(idx * 2 + 1);
        let u1mat = na::DMatrix::from_rows(&[u1, u1, u1]);
        let u2mat = na::DMatrix::from_rows(&[u2, u2, u2]);

        let coeffs: na::DMatrix<f64> = a * u1mat.component_mul(&u1mat.transpose())
            - c * u1mat.component_mul(&u2mat.transpose())
            - a * u2mat.component_mul(&u2mat.transpose());
        let b = na::DMatrix::from_fn(9, 9, |r, c| coeffs[(r / 3, r % 3)] * coeffs[(c / 3, c % 3)]);
        acc + b
    });

    let r2 = 2.0f64.sqrt();
    #[rustfmt::skip]
    let bmat = na::DMatrix::from_row_slice(6, 6, &[
        bs[(0, 0)], bs[(0, 4)], bs[(0, 8)], r2 * bs[(0, 5)], r2 * bs[(0, 6)], r2 * bs[(0, 1)],
        bs[(4, 0)], bs[(4, 4)], bs[(4, 8)], r2 * bs[(4, 5)], r2 * bs[(4, 6)], r2 * bs[(4, 1)],
        bs[(8, 0)], bs[(8, 4)], bs[(8, 8)], r2 * bs[(8, 5)], r2 * bs[(8, 6)], r2 * bs[(8, 1)],
        r2 * bs[(5, 0)], r2 * bs[(5, 4)], r2 * bs[(5, 8)], bs[(5, 5)], bs[(5, 6)], bs[(5, 1)],
        r2 * bs[(6, 0)], r2 * bs[(6, 4)], r2 * bs[(6, 8)], bs[(6, 5)], bs[(6, 6)], bs[(6, 1)],
        r2 * bs[(1, 0)], r2 * bs[(1, 4)], r2 * bs[(1, 8)], bs[(1, 5)], bs[(1, 6)], bs[(1, 1)],
    ]);

    // least square solution
    let tau: na::DVector<f64> = lstsq(&bmat)?;
    #[rustfmt::skip]
    let mut metrix_mat: na::DMatrix<f64> = na::DMatrix::from_row_slice(3, 3, &[
        tau[0], tau[5] / r2, tau[4] / r2,
        tau[5] / r2, tau[1], tau[3] / r2,
        tau[4] / r2, tau[3] / r2, tau[2]
    ]);
    if metrix_mat.determinant() < 0.0 {
        metrix_mat *= -1.0;
    }

    let affine: na::DMatrix<f64> = metrix_mat
        .cholesky()
        .context("Failed to cholesky decomposition")?
        .l();
    let affine_inv = affine
        .clone()
        .try_inverse()
        .context("Failed to calculate inverse matrix")?;

    let motion_mat = motion_mat * &affine;
    let shape_mat = &affine_inv * shape_mat;
    Ok((motion_mat, shape_mat))
}

fn calc_motion_and_shape_matrix(
    observation_matrix: &na::DMatrix<f64>,
) -> Result<(na::DMatrix<f64>, na::DMatrix<f64>)> {
    let mut svd = observation_matrix.clone().svd(true, true);
    svd.sort_by_singular_values();
    let singulars = svd.singular_values;
    let u: na::DMatrix<f64> = svd.u.context("Failed to get SVD value")?;
    let v_t: na::DMatrix<f64> = svd.v_t.context("Failed to get SVD Value")?;
    let sigma = na::DMatrix::from_diagonal(&na::DVector::from_row_slice(&[
        singulars[0],
        singulars[1],
        singulars[2],
    ]));
    let motion_matrix = na::DMatrix::from_columns(&[u.column(0), u.column(1), u.column(2)]);
    // println!("sigma = {:?}", sigma);
    // println!("v_t = {:?}", v_t);
    let shape_matrix = sigma * na::DMatrix::from_rows(&[v_t.row(0), v_t.row(1), v_t.row(2)]);
    Ok((motion_matrix, shape_matrix))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_affine_self_calibration() {
        #[rustfmt::skip]
        let observed_mat = vec![
            vec![na::Point2::new(1.0, 1.0), na::Point2::new(0.0, 0.0),
                 na::Point2::new(0.0, 1.0), na::Point2::new(1.0, 0.0)],
            vec![na::Point2::new(0.0, 1.0), na::Point2::new(-1.0, 0.0),
                 na::Point2::new(-1.0, 1.0), na::Point2::new(0.0, 0.0)],
            vec![na::Point2::new(0.0, 0.0), na::Point2::new(-1.0, -1.0),
                 na::Point2::new(-1.0, 0.0), na::Point2::new(0.0, -1.0)],
        ];
        let (motion, shape) = affine_self_calibration(&observed_mat).unwrap();
        println!("motion = {:?}", motion);
        println!("shape = {:?}", shape);
        assert_eq!(motion.ncols(), 3);
        assert_eq!(shape.nrows(), 3);
    }

    #[test]
    fn test_calc_motion_and_shape_mat() {
        #[rustfmt::skip]
        let observed_mat = na::DMatrix::from_row_slice(6, 4, &[
            1.0, 0.0, 0.0, 1.0,
            1.0, 0.0, 1.0, 0.0,
            0.0, -1.0, -1.0, 0.0,
            1.0, 0.0, 1.0, 0.0,
            0.0, -1.0, -1.0, 0.0,
            0.0, -1.0, 0.0, -1.0
        ]);
        let (motion, shape) = calc_motion_and_shape_matrix(&observed_mat).unwrap();
        println!("{:?}", motion);
        println!("{:?}", shape);
        assert_eq!(motion.ncols(), 3);
        assert_eq!(shape.nrows(), 3);
    }
}
