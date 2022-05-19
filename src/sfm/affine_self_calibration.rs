use anyhow::{ensure, Context, Result};
use nalgebra as na;

use crate::linalg::get_zero_mat;

/// Self calibration using affine camera model.
/// - observed_pts : Observed points. (2d vector : [index of camera][index of point])
/// - Return : Tuple of (motion matrix (stacked camera matrices), shape matrix (stacked 3d points)).
pub fn affine_self_calibration(
    observed_pts: &[Vec<na::Point2<f64>>],
) -> Result<(na::DMatrix<f64>, na::DMatrix<f64>)> {
    ensure!(!observed_pts.is_empty(), "observed_pts must not be empty");
    let n_points = observed_pts.len();
    let n_cameras = observed_pts[0].len();

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

    // constrained least square method

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
    let shape_matrix =
        sigma * na::DMatrix::from_columns(&[v_t.column(0), v_t.column(1), v_t.column(2)]);
    Ok((motion_matrix, shape_matrix))
}
