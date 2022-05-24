use anyhow::{Context, Result};
use nalgebra as na;

/// - observed_pts : Observed points. (2d vector : [index of camera][index of point])
pub fn projective_self_calibration(
    observed_points: &[Vec<na::Point2<f64>>],
) -> Result<(na::DMatrix<f64>, na::DMatrix<f64>)> {
    projective_reconstruction();
    euclide_reconstruction();
}

fn projective_reconstruction() {}

///
/// - observed_pts : Observed points. (2d vector : [index of camera][index of point])
/// - return tuple of (cameras' motion matrix, shape matrix)
fn primary_method(
    observed_points: &[Vec<na::Point2<f64>>],
) -> Result<(na::DMatrix<f64>, na::DMatrix<f64>)> {
    let epsilon = 10.0; // unit : pixel
    let n_cameras = observed_points.len();
    let n_points = observed_points[0].len();
    let mut zs = na::DMatrix::from_element(n_cameras, n_points, 1.0);

    let inner_product = |pt: &na::Point2<f64>, mat: &na::DMatrix<f64>, row: usize, col: usize| {
        pt.x * mat[(row, col)] + pt.y * mat[(row, col + 1)] + mat[(row, col + 2)]
    };
    let point_norm = |pt: &na::Point2<f64>| (pt.x * pt.x + pt.y * pt.y + 1.0).sqrt();

    loop {
        let observed_mat = get_observed_matrix(observed_points, &zs);
        let svd = observed_mat.svd(true, true);
        svd.sort_by_singular_values();
        let (motion_mat, shape_mat) = get_motion_and_shape_from_svd(&svd)?;

        if calculate_reprojection_error(observed_points, &motion_mat, &shape_mat) < epsilon {
            return Ok((motion_mat, shape_mat));
        }

        (0..n_points).map(|ip| {
            let a: na::DMatrix<f64> = na::DMatrix::from_fn(n_cameras, n_cameras, |r, c| {
                let rpt: na::Point2<f64> = observed_points[r][ip];
                let cpt: na::Point2<f64> = observed_points[c][ip];
                let nume = (0..4).fold(0.0, |accum, idx| {
                    accum
                        + inner_product(&rpt, &motion_mat, idx, 3 * r)
                            * inner_product(&cpt, &motion_mat, idx, 3 * c)
                });
                let deno = point_norm(&rpt) * point_norm(&cpt);
                nume / deno
            });
            let eigen = a.symmetric_eigen();
            let xi = eigen.eigenvectors.column(eigen.eigenvalues.imax());

            (0..n_cameras)
                .for_each(|ic| zs[(ic, ip)] = xi[ic] / point_norm(&observed_points[ic][ip]));
        });
    }
}

fn dual_method() {}

fn euclide_reconstruction() {}

fn get_observed_matrix(
    observed_points: &[Vec<na::Point2<f64>>],
    zs: &na::DMatrix<f64>,
) -> na::DMatrix<f64> {
    let n_cameras = observed_points.len();
    let n_points = observed_points[0].len();

    na::DMatrix::from_fn(n_cameras * 3, n_points, |r, c| {
        let cam_idx = r / 3;
        let coord_idx = r % 3;
        observed_points[cam_idx][c][coord_idx] * zs[(cam_idx, c)]
    })
}

fn get_motion_and_shape_from_svd(
    svd: &na::SVD<f64, na::Dynamic, na::Dynamic>,
) -> Result<(na::DMatrix<f64>, na::DMatrix<f64>)> {
}

fn calculate_reprojection_error(
    observed_points: &[Vec<na::Point2<f64>>],
    motion_mat: &na::DMatrix<f64>,
    shape_mat: &na::DMatrix<f64>,
) -> f64 {
}
