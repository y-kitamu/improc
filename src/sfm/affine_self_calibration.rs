use anyhow::{ensure, Context, Result};
use nalgebra as na;

/// Self calibration using affine camera model.
///
/// - Return : Tuple of (motion matrix (stacked camera matrices), shape matrix (stacked 3d points)).
pub fn affine_self_calibration(
    observed_pts: &[Vec<na::Point2<f64>>],
) -> Result<(na::DMatrix<f64>, na::DMatrix<f64>)> {
    ensure!(!observed_pts.is_empty(), "observed_pts must not be empty");
    let n_points = observed_pts.len();
    let n_cameras = observed_pts[0].len();

    let observation_matrix: na::DMatrix<f64> = na::DMatrix::from_row_slice(
        n_points,
        n_cameras,
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
