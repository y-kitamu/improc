//! Calculate fundamental matrix
use nalgebra as na;

use crate::{linalg::get_zero_mat, optimizer::ObservedData};

struct FundamentalMatrixData<'a> {
    data: &'a [na::Point2<f64>],
    scale: f64,
}

impl<'a> ObservedData<'a> for FundamentalMatrixData<'a> {
    /// `data` format : [image0_pt0, image1_pt0, image0_pt1, image1_pt1, image0_pt2, image1_pt2, ....]
    fn new(data: &'a [na::Point2<f64>]) -> Self {
        // let scale = data
        //     .iter()
        //     .fold(0.0f64, |acc, pt| acc + pt[0].abs() + pt[1].abs())
        //     / (data.len() as f64 * 2.0);
        let scale = 1.0;
        FundamentalMatrixData { data, scale }
    }

    fn len(&self) -> usize {
        self.data.len() / 2
    }

    fn vector(&self, data_index: usize) -> na::DVector<f64> {
        let pt0 = self.data[data_index * 2];
        let pt1 = self.data[data_index * 2 + 1];
        let (x0, y0) = (pt0[0], pt0[1]);
        let (x1, y1) = (pt1[0], pt1[1]);
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
        let pt1 = self.data[data_index * 2 + 1];
        let (x0, y0) = (pt0[0], pt0[1]);
        let (x02, y02) = (x0 * x0, y0 * y0);
        let (x1, y1) = (pt1[0], pt1[1]);
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
}

#[cfg(test)]
mod tests {
    use crate::optimizer::{
        fns::fns,
        least_square::{iterative_reweight, least_square_fitting},
        taubin::{renormalization, taubin},
    };

    use super::*;
    use rand::Rng;

    fn create_test_data() -> (na::Matrix3<f64>, Vec<na::Point2<f64>>) {
        let std_dev = 5.0;

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
                let x = (rng.gen::<f64>() - 0.5) * std_dev;
                let y = (rng.gen::<f64>() - 0.5) * std_dev;
                let pt1: na::Point3<f64> = homo * na::Point3::new(x, y, 1.0);
                vec![
                    na::Point2::<f64>::new(x, y),
                    na::Point2::<f64>::new(pt1[0], pt1[1]),
                ]
            })
            .flatten()
            .collect();
        (homo, points)
    }

    fn assert_result(res: na::DVector<f64>, points: Vec<na::Point2<f64>>) {
        let fund_mat = na::Matrix3::from_row_slice(res.as_slice());
        let n_data = points.len() / 2;
        let res = (0..n_data).fold(0.0, |acc, idx| {
            let p0 = points[idx * 2];
            let p1 = points[idx * 2 + 1];
            let v0 = na::Vector3::new(p0[0], p0[1], 1.0);
            let v1 = na::Vector3::new(p1[0], p1[1], 1.0);
            println!(
                "p0 = {:?}, p1 = {:?}, res = {:.3}, {:.3}",
                p0.coords.as_slice(),
                p1.coords.as_slice(),
                (v0.transpose() * fund_mat * v1)[(0, 0)],
                (v0.transpose() * fund_mat.transpose() * v1)[(0, 0)]
            );
            let res = (v0.transpose() * fund_mat * v1)[(0, 0)];
            assert!(res.abs() < 1e-3);
            acc + res
        }) / n_data as f64;
        assert!(res.abs() < 1e-5);
    }

    #[test]
    fn test_least_square() {
        for i in 0..10 {
            println!("Trial = {}", i);
            let (_, points) = create_test_data();
            let res = least_square_fitting::<FundamentalMatrixData>(&points).unwrap();
            assert_result(res, points);
        }
    }

    #[test]
    fn test_iterative_reweight() {
        for i in 0..100 {
            println!("Trial = {}", i);
            let (_, points) = create_test_data();
            let res = iterative_reweight::<FundamentalMatrixData>(&points).unwrap();
            assert_result(res, points);
        }
    }

    #[test]
    fn test_taubin() {
        for i in 0..10 {
            println!("Trial = {}", i);
            let (_, points) = create_test_data();
            let res = taubin::<FundamentalMatrixData>(&points).unwrap();
            assert_result(res, points);
        }
    }

    #[test]
    fn test_renormalization() {
        for i in 0..100 {
            println!("Trial = {}", i);
            let (_, points) = create_test_data();
            let res = renormalization::<FundamentalMatrixData>(&points).unwrap();
            assert_result(res, points);
        }
    }

    #[test]
    fn test_fns() {
        for i in 0..100 {
            println!("Trial = {}", i);
            let (_, points) = create_test_data();
            let res = fns::<FundamentalMatrixData>(&points).unwrap();
            assert_result(res, points);
        }
    }
}
