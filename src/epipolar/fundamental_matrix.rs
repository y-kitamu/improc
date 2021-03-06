//! Calculate fundamental matrix
use anyhow::Result;
use nalgebra as na;

use crate::{
    linalg::{get_identity_mat, get_zero_mat, matrix::pseudo_inverse},
    optimizer::ObservedData,
};

/// Struct for computing fundamental matrix.
/// - `data` is observed points in the two images. [image0_pt0, image1_pt0, image0_pt1, ....].
/// - `delta` is offset for optimal point which is calculated in geometric error minimization.
/// (Optimal point is at `data` + `delta`.)
pub struct FundamentalMatrixData<'a> {
    data: &'a [na::Point2<f64>],
    scale: f64,
    delta: Vec<na::Point2<f64>>,
}

impl<'a> ObservedData<'a> for FundamentalMatrixData<'a> {
    /// `data` format : [image0_pt0, image1_pt0, image0_pt1, image1_pt1, image0_pt2, image1_pt2, ....]
    fn new(data: &'a [na::Point2<f64>]) -> Self {
        // let scale = data
        //     .iter()
        //     .fold(0.0f64, |acc, pt| acc + pt[0].abs() + pt[1].abs())
        //     / (data.len() as f64 * 2.0);
        let scale = 1.0;
        FundamentalMatrixData {
            data,
            scale,
            delta: vec![na::Point2::new(0.0, 0.0); data.len()],
        }
    }

    fn len(&self) -> usize {
        self.data.len() / 2
    }

    fn vector(&self, data_index: usize) -> na::DVector<f64> {
        let pt0 = self.data[data_index * 2];
        let d0 = self.delta[data_index * 2];
        let pt1 = self.data[data_index * 2 + 1];
        let d1 = self.delta[data_index * 2 + 1];
        let (x0, y0) = (pt0[0] + d0[0], pt0[1] + d0[1]);
        let (x1, y1) = (pt1[0] + d1[0], pt1[1] + d1[1]);
        let f0 = self.scale;
        na::DVector::<f64>::from_vec(vec![
            x0 * x1,
            x0 * y1,
            f0 * x0,
            y0 * x1,
            y0 * y1,
            f0 * y0,
            f0 * x1,
            f0 * y1,
            f0 * f0,
        ])
    }

    fn matrix(&self, weight_vector: &[f64]) -> na::DMatrix<f64> {
        (0..self.len()).fold(get_zero_mat(self.vec_size()), |acc, idx| {
            let xi = self.vector(idx);
            acc + weight_vector[idx] * &xi * &xi.transpose()
        })
    }

    fn variance(&self, data_index: usize) -> na::DMatrix<f64> {
        let pt0 = self.data[data_index * 2];
        let d0 = self.delta[data_index * 2];
        let pt1 = self.data[data_index * 2 + 1];
        let d1 = self.delta[data_index * 2 + 1];
        let (x0, y0) = (pt0[0] + d0[0], pt0[1] + d0[1]);
        let (x02, y02) = (x0 * x0, y0 * y0);
        let (x1, y1) = (pt1[0] + d1[0], pt1[1] + d1[1]);
        let (x12, y12) = (x1 * x1, y1 * y1);
        let f0 = self.scale;
        let f02 = f0 * f0;
        let vec_size = self.vec_size();
        #[rustfmt::skip]
        let mat = na::DMatrix::<f64>::from_row_slice(vec_size, vec_size, &[
            x02 + x12, x1 * y1,   f0 * x1, x0 * y0,   0.0,       0.0,     f0 * x0, 0.0,     0.0,
            x1 * y1,   x02 + y12, f0 * y1, 0.0,       x0 * y0,   0.0,     0.0,     f0 * x1, 0.0,
            f0 * x1,   f0 * y1,   f02,     0.0,       0.0,       0.0,     0.0,     0.0,     0.0,
            x0 * y0,   0.0,       0.0,     y02 + x12, x1 * y1,   f0 * x1, f0 * y0, 0.0,     0.0,
            0.0,       x0 * y0,   0.0,     x1 * y1,   y02 + y12, f0 * y1, 0.0,     f0 * y0, 0.0,
            0.0,       0.0,       0.0,     f0 * x1,   f0 * y1,   f02,     0.0,     0.0,     0.0,
            f0 * x0,   0.0,       0.0,     f0 * y0,   0.0,       0.0,     f02,     0.0,     0.0,
            0.0,       f0 * x0,   0.0,     0.0,       f0 * y0,   0.0,     0.0,     f02,     0.0,
            0.0,       0.0,       0.0,     0.0,       0.0,       0.0,     0.0,     0.0,     0.0,
        ]);
        mat
    }

    fn weights(&self, params: &na::DVector<f64>) -> Vec<f64> {
        if params.as_slice().iter().any(|&val| val.abs() < 1e-5) {
            return vec![1.0; self.data.len()];
        }
        (0..self.len())
            .map(|idx| {
                let var_mat = self.variance(idx);
                1.0 / params.dot(&(&var_mat * params))
            })
            .collect()
    }

    fn update_delta(&mut self, params: &na::DVector<f64>) -> f64 {
        #[rustfmt::skip]
        let param_mat = na::Matrix2x3::new(
            params[0], params[1], params[2],
            params[3], params[4], params[5],
        );
        (0..self.len())
            .map(|idx| {
                let xi = self.vector(idx);
                let var_mat = self.variance(idx);
                let mut delta_sum = 0.0;
                for off in 0..2 {
                    let i = idx * 2 + off;
                    self.delta[i].coords = -(xi.transpose() * params)[(0, 0)]
                        / (params.transpose() * &var_mat * params)[(0, 0)]
                        * param_mat
                        * na::Vector3::new(
                            self.data[i][0] + self.delta[i][0],
                            self.data[i][1] + self.delta[i][1],
                            self.scale,
                        );
                    delta_sum += self.delta[i].coords.norm_squared();
                }
                delta_sum
            })
            .sum::<f64>()
            / self.len() as f64
    }

    fn get_data(&self) -> Vec<na::Point2<f64>> {
        self.data
            .iter()
            .zip(self.delta.iter())
            .map(|(x, d)| na::Point2::new(x[0] + d[0], x[1] + d[1]))
            .collect()
    }
}

const MAX_ITERATION: usize = 10;
const STOP_THRESHOLD: f64 = 1e-5;

/// optimal correction for fundamental matrix.
pub fn optimal_correction(
    data: &[na::Point2<f64>],
    params: na::DVector<f64>,
) -> Result<na::DVector<f64>> {
    let data_container = FundamentalMatrixData::new(data);
    let weights = data_container.weights(&params);
    let pers_mat = get_identity_mat(data_container.vec_size()) - &params * params.transpose();
    let mat =
        (0..data_container.len()).fold(get_zero_mat(data_container.vec_size()), |acc, idx| {
            let pers = &pers_mat * data_container.vector(idx);
            acc + weights[idx] * &pers * pers.transpose()
        }) / data_container.len() as f64;
    let mut var_mat: na::DMatrix<f64> = pseudo_inverse(&mat)? / data_container.len() as f64;

    let mut updated = params;
    for _ in 0..MAX_ITERATION {
        let cofactors = na::DVector::<f64>::from_row_slice(&[
            updated[4] * updated[8] - updated[7] * updated[5],
            updated[5] * updated[6] - updated[8] * updated[3],
            updated[3] * updated[7] - updated[6] * updated[4],
            updated[7] * updated[2] - updated[1] * updated[8],
            updated[8] * updated[0] - updated[2] * updated[6],
            updated[6] * updated[1] - updated[0] * updated[7],
            updated[1] * updated[5] - updated[4] * updated[2],
            updated[2] * updated[3] - updated[5] * updated[0],
            updated[0] * updated[4] - updated[3] * updated[1],
        ]);
        updated -= (cofactors.transpose() * &updated)[(0, 0)] * &var_mat * &cofactors
            / (3.0 * cofactors.transpose() * &var_mat * &cofactors)[(0, 0)];
        updated = updated.normalize();
        if (cofactors.transpose() * &updated)[(0, 0)].abs() < STOP_THRESHOLD {
            break;
        }
        let pers_mat = get_identity_mat(data_container.vec_size()) - &updated * updated.transpose();
        var_mat = &pers_mat * var_mat * &pers_mat;
    }
    Ok(updated)
}

#[cfg(test)]
pub mod tests {
    use crate::optimizer::{
        fns::fns,
        geometric::minimize_geometric_distance,
        least_square::{iterative_reweight, least_square_fitting},
        taubin::{renormalization, taubin},
    };

    use super::*;
    use rand::Rng;

    const LOOP_NUM: usize = 50;

    pub fn create_test_data() -> (na::Matrix3<f64>, Vec<na::Point2<f64>>) {
        let std_dev = 0.05;
        create_test_data_with_params(std_dev)
    }

    pub fn create_test_data_with_params(std_dev: f64) -> (na::Matrix3<f64>, Vec<na::Point2<f64>>) {
        let mut rng = rand::thread_rng();
        let theta: f64 = rng.gen::<f64>() * std::f64::consts::PI * 2.0;
        let dx: f64 = rng.gen::<f64>();
        let dy: f64 = rng.gen::<f64>();
        #[rustfmt::skip]
        let homo = na::Matrix3::new(
            theta.cos(), -theta.sin(), dx,
            theta.sin(), theta.cos(), dy,
            0.0, 0.0, 1.0
        );
        let points: Vec<na::Point2<f64>> = (0..100)
            .map(|_| {
                let x = (rng.gen::<f64>() - 0.5) * 5.0; // -5.0 ~ 5.0
                let y = (rng.gen::<f64>() - 0.5) * 5.0; // -5.0 ~ 5.0
                let dx = (rng.gen::<f64>() - 0.5) * std_dev;
                let dy = (rng.gen::<f64>() - 0.5) * std_dev;
                let pt1: na::Point3<f64> = homo * na::Point3::new(x, y, 1.0);
                vec![
                    na::Point2::<f64>::new(x, y),
                    na::Point2::<f64>::new(pt1[0] + dx, pt1[1] + dy),
                ]
            })
            .flatten()
            .collect();
        (homo, points)
    }

    pub fn assert_result(res: na::DVector<f64>, points: Vec<na::Point2<f64>>) -> f64 {
        let fund_mat = na::Matrix3::from_row_slice(res.as_slice());
        let n_data = points.len() / 2;
        let res = (0..n_data).fold(0.0, |acc, idx| {
            let p0 = points[idx * 2];
            let p1 = points[idx * 2 + 1];
            let v0 = na::Vector3::new(p0[0], p0[1], 1.0);
            let v1 = na::Vector3::new(p1[0], p1[1], 1.0);
            // println!(
            //     "idx = {}, p0 = {:?}, p1 = {:?}, residual = {:.3}",
            //     idx,
            //     p0.coords.as_slice(),
            //     p1.coords.as_slice(),
            //     (v0.transpose() * fund_mat * v1)[(0, 0)],
            // );
            let res = (v0.transpose() * fund_mat * v1)[(0, 0)];
            acc + res.abs()
        }) / n_data as f64;
        // assert!(res.abs() < 1e-2, "res = {}", res);
        res
    }

    #[test]
    fn test_least_square() {
        let res: f64 = (0..10)
            .map(|_| {
                // println!("Trial = {}", i);
                let (_, points) = create_test_data();
                let res = least_square_fitting::<FundamentalMatrixData>(&points).unwrap();
                assert_result(res, points)
            })
            .sum::<f64>()
            / 10.0;
        assert!(res.abs() < 1e-2, "res = {}", res);
    }

    #[test]
    fn test_iterative_reweight() {
        let res: usize = (0..LOOP_NUM)
            .map(|_| {
                let (_, points) = create_test_data();
                let res = iterative_reweight::<FundamentalMatrixData>(&points).unwrap();
                assert_result(res, points)
            })
            .map(|val| if val.abs() < 1e-2 { 1 } else { 0 })
            .sum();

        assert!(
            res as f64 > LOOP_NUM as f64 * 0.9,
            "success : {} / {}",
            res,
            LOOP_NUM
        );
    }

    #[test]
    fn test_taubin() {
        let res: usize = (0..LOOP_NUM)
            .map(|_| {
                let (_, points) = create_test_data();
                let res = taubin::<FundamentalMatrixData>(&points).unwrap();
                assert_result(res, points)
            })
            .map(|val| if val.abs() < 1e-2 { 1 } else { 0 })
            .sum();

        assert!(
            res as f64 > LOOP_NUM as f64 * 0.9,
            "success : {} / {}",
            res,
            LOOP_NUM
        );
    }

    #[test]
    fn test_renormalization() {
        let res: usize = (0..LOOP_NUM)
            .map(|_| {
                let (_, points) = create_test_data();
                let res = renormalization::<FundamentalMatrixData>(&points).unwrap();
                assert_result(res, points)
            })
            .map(|val| if val.abs() < 1e-2 { 1 } else { 0 })
            .sum();

        assert!(
            res as f64 > LOOP_NUM as f64 * 0.9,
            "success : {} / {}",
            res,
            LOOP_NUM
        );
    }

    #[test]
    fn test_fns() {
        let res: f64 = (0..20)
            .map(|_| {
                let (_, points) = create_test_data();
                let res = fns::<FundamentalMatrixData>(&points).unwrap();
                assert_result(res, points)
            })
            .sum::<f64>()
            / 20.0;
        assert!(res.abs() < 1e-2, "res = {}", res);
    }

    #[test]
    fn test_optimal_correction() {
        let res: f64 = (0..20)
            .map(|_| {
                let (_, points) = create_test_data();
                let res = fns::<FundamentalMatrixData>(&points).unwrap();
                let res = optimal_correction(&points, res).unwrap();
                assert_result(res, points)
            })
            .sum::<f64>()
            / 20.0;
        assert!(res.abs() < 1e-2, "res = {}", res);
    }

    #[test]
    fn test_geometric() {
        let res: f64 = (0..20)
            .map(|_| {
                let (_, points) = create_test_data();
                let r = minimize_geometric_distance::<FundamentalMatrixData>(&points).unwrap();
                assert_result(r, points)
            })
            .sum::<f64>()
            / 20.0;
        assert!(res.abs() < 1e-2, "res = {}", res);
    }
}
