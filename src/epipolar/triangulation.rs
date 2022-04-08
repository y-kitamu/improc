use anyhow::Result;
use nalgebra as na;

use crate::{
    linalg::matrix::{le_lstsq, lstsq},
    optimizer::{geometric::minimize_geometric_distance_impl, ObservedData},
};

/// Triangulation by camera matrix. Calculate position of the point in world coordinates.
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
        f0 * p0[(0, 0)] - x0[0] * p0[(2, 0)], f0 * p0[(0, 1)] - x0[0] * p0[(2, 1)], f0 * p0[(0, 2)] - x0[0] * p0[(2, 2)],
        f0 * p0[(1, 0)] - x0[1] * p0[(2, 0)], f0 * p0[(1, 1)] - x0[1] * p0[(2, 1)], f0 * p0[(1, 2)] - x0[1] * p0[(2, 2)],
        f0 * p1[(0, 0)] - x1[0] * p1[(2, 0)], f0 * p1[(0, 1)] - x1[0] * p1[(2, 1)], f0 * p1[(0, 2)] - x1[0] * p1[(2, 2)],
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

#[cfg(test)]
mod tests {
    use rand::Rng;

    use crate::{epipolar::fundamental_matrix::FundamentalMatrixData, linalg::vector_cross_matrix};

    use super::*;

    #[test]
    fn test_triangulation() {
        #[rustfmt::skip]
        let p0 = na::DMatrix::from_row_slice(3, 4, &[
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0
        ]);
        #[rustfmt::skip]
        let p1 = na::DMatrix::from_row_slice(3, 4, &[
            1.0, 2.0, 3.0, 4.0,
            5.0, 6.0, 7.0, 8.0,
            9.0, 0.0, 1.0, 2.0
        ]);
        let gpt = na::DVector::from_vec(vec![5.0, 4.0, 3.0, 1.0]);
        let x0 = &p0 * &gpt;
        let x1 = &p1 * &gpt;

        let pt = triangulation(
            &p0,
            &p1,
            &na::Point2::new(x0[0] / x0[2], x0[1] / x0[2]),
            &na::Point2::new(x1[0] / x1[2], x1[1] / x1[2]),
            1.0,
        )
        .unwrap();
        println!("GT = {:?}", gpt.as_slice());
        println!("PR = {:?}", pt.as_slice());
        assert!((gpt[0].abs() - pt[0].abs()).abs() < 1e-5);
        assert!((gpt[1].abs() - pt[1].abs()).abs() < 1e-5);
        assert!((gpt[2].abs() - pt[2].abs()).abs() < 1e-5);
    }

    #[test]
    fn test_optimal_correction() {
        let mut rng = rand::thread_rng();
        let theta: f64 = rng.gen::<f64>() * std::f64::consts::PI * 2.0;
        #[rustfmt::skip]
        let rot = na::DMatrix::from_row_slice(3, 3, &[
            theta.cos(), -theta.sin(), 0.0,
            theta.sin(), theta.cos(), 0.0,
            0.0, 0.0, 1.0
        ]);
        let trans_vec = na::DVector::from_vec(vec![1.0, 2.0, 3.0]);
        let trans = vector_cross_matrix(&trans_vec);
        let fund_mat = &trans * &rot;

        let x0 = na::DVector::from_vec(vec![0.5, 0.3, 1.0]);
        let mut x1 = &rot * &x0 + &trans_vec;
        x1 /= x1[2];

        let scale = 0.00;
        let data = vec![
            na::Point2::new(
                x0[0] + (rng.gen::<f64>() - 0.5) * scale,
                x0[1] + (rng.gen::<f64>() - 0.5) * scale,
            ),
            na::Point2::new(
                x1[0] + (rng.gen::<f64>() - 0.5) * scale,
                x1[1] + (rng.gen::<f64>() - 0.5) * scale,
            ),
        ];

        let (r0, r1) = optimal_correction::<FundamentalMatrixData>(&fund_mat, &data).unwrap();
        println!("x0 = {:?}", x0.as_slice());
        println!("x1 = {:?}", x1.as_slice());
        println!("r0 = {:?}", r0.coords.as_slice());
        println!("r1 = {:?}", r1.coords.as_slice());
        assert!((r0[0] - x0[0]).abs() < 1e-1);
        assert!((r0[1] - x0[1]).abs() < 1e-1);
        assert!((r1[0] - x1[0]).abs() < 1e-1);
        assert!((r1[1] - x1[1]).abs() < 1e-1);
    }
}
