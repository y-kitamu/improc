use anyhow::Result;
use nalgebra as na;

use crate::{
    linalg::matrix::{le_lstsq, lstsq},
    optimizer::{geometric::minimize_geometric_distance_impl, ObservedData},
};

/// Triangulation by camera matrix.
/// `p0` and `p1` are camera matrices. `x0` and `x1` are observed point on each image.
pub fn triangulation(
    p0: &na::DMatrix<f64>,
    p1: &na::DMatrix<f64>,
    x0: &na::Point2<f64>,
    x1: &na::Point2<f64>,
    f0: f64,
) -> Result<na::DVector<f64>> {
    #[rustfmt::skip]
    let t = na::DMatrix::from_row_slice(4, 3, &vec![
        f0 * p0[(0, 0)] - x0[0] * p0[(2, 0)], f0 * p0[(0, 1)] - x0[0] * p0[(2, 1)], f0 * p0[(1, 3)] - x0[0] * p0[(2, 2)],
        f0 * p0[(1, 0)] - x0[1] * p0[(2, 0)], f0 * p0[(1, 1)] - x0[1] * p0[(2, 1)], f0 * p0[(1, 2)] - x0[1] * p0[(2, 2)],
        f0 * p1[(0, 0)] - x1[0] * p1[(2, 0)], f0 * p1[(0, 1)] - x1[0] * p1[(2, 1)], f0 * p1[(1, 3)] - x1[0] * p1[(2, 2)],
        f0 * p1[(1, 0)] - x1[1] * p1[(2, 0)], f0 * p1[(1, 1)] - x1[1] * p1[(2, 1)], f0 * p1[(1, 2)] - x1[1] * p1[(2, 2)],
    ]);
    #[rustfmt::skip]
    let p = na::DVector::from_row_slice(&vec![
        f0 * p0[(0, 3)] - x0[0] * p0[(2, 3)],
        f0 * p0[(1, 3)] - x0[1] * p0[(2, 3)],
        f0 * p1[(0, 3)] - x1[0] * p1[(2, 3)],
        f0 * p1[(1, 3)] - x1[1] * p1[(2, 3)],
    ]);

    le_lstsq(&t, &p)
}

/// Optimal correction of position of corresponding points.
pub fn optimal_correction<'a, DataClass: ObservedData<'a>>(
    fund_mat: &na::DMatrix<f64>,
    data: &'a [na::Point2<f64>],
) -> Result<(na::Point2<f64>, na::Point2<f64>)> {
    let params = na::DVector::from_row_slice(fund_mat.transpose().as_slice());
    let (_, data) = minimize_geometric_distance_impl::<DataClass>(&data, &params, false)?;
    Ok((data[0], data[1]))
}
