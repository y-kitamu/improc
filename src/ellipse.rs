//! Module for ellipse fitting algorithms.
use nalgebra as na;

use crate::{linalg::get_zero_mat, optimizer::ObservedData};

pub mod fns;
pub mod least_square;
pub mod taubin;
pub mod test_utility;

struct EllipseData<'a> {
    data: &'a [na::Point2<f64>],
    scale: f64,
}

impl<'a> ObservedData<'a> for EllipseData<'a> {
    fn new(data: &'a [na::Point2<f64>]) -> Self {
        // let scale = data
        //     .iter()
        //     .fold(0.0f64, |acc, pt| acc + pt[0].abs() + pt[1].abs())
        //     / (data.len() as f64 * 2.0);
        let scale = 1.0;
        EllipseData { data, scale }
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    /// Calculate xi : xi = (x^2, 2xy, y^2, 2fx, 2fy, f^2)
    fn vector(&self, data_index: usize) -> na::DVector<f64> {
        let x = self.data[data_index][0];
        let y = self.data[data_index][1];
        na::DVector::from_vec(vec![
            x * x,
            2.0 * x * y,
            y * y,
            2.0 * self.scale * x,
            2.0 * self.scale * y,
            self.scale * self.scale,
        ])
    }

    /// Calculate 6 x 6 matrix of xi * xi^T.
    /// xi = (x^2, 2xy, y^2, 2fx, 2fy, f^2)
    fn matrix(&self, weight_vector: &[f64]) -> na::DMatrix<f64> {
        (0..self.data.len()).fold(get_zero_mat(6), |mut acc, idx| {
            let xi = &self.vector(idx);
            acc += weight_vector[idx] * xi * xi.transpose();
            acc
        }) / self.data.len() as f64
    }

    /// Calculate variance matrix
    fn variance(&self, data_index: usize) -> na::DMatrix<f64> {
        let x = self.data[data_index][0];
        let y = self.data[data_index][1];
        let scale = self.scale;
        #[rustfmt::skip]
        let mat = na::DMatrix::<f64>::from_row_slice(6, 6, &[
            x * x,     x * y,         0.0,       scale * x,     0.0,           0.0,
            x * y,     x * x + y * y, x * y,     scale * y,     scale * x,     0.0,
            0.0,       x * y,         y * y,     0.0,           scale * y,     0.0,
            scale * x, scale * y,     0.0,       scale * scale, 0.0,           0.0,
            0.0,       scale * x,     scale * y, 0.0,           scale * scale, 0.0,
            0.0,       0.0,           0.0,       0.0,           0.0,           0.0,
        ]);
        mat
    }

    fn weights(&self, params: &na::DVector<f64>) -> Vec<f64> {
        if params.as_slice().iter().any(|&val| val.abs() < 1e-5) {
            return vec![1.0; self.data.len()];
        }
        (0..self.data.len())
            .map(|idx| {
                let var_mat = self.variance(idx);
                1.0 / params.dot(&(&var_mat * params))
            })
            .collect()
    }
}
