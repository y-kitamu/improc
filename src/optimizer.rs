//! Trait definitions for optimization problems.
use nalgebra as na;

pub mod fns;
pub mod geometric;
pub mod least_square;
pub mod taubin;

/// Data trait definition
pub trait ObservedData<'a> {
    fn new(data: &'a [na::Point2<f64>]) -> Self;
    fn len(&self) -> usize;
    fn vector(&self, data_index: usize) -> na::DVector<f64>;
    fn matrix(&self, weight_vector: &[f64]) -> na::DMatrix<f64>;
    fn variance(&self, data_index: usize) -> na::DMatrix<f64>;
    fn weights(&self, params: &na::DVector<f64>) -> Vec<f64>;

    fn vec_size(&self) -> usize {
        self.vector(0).nrows()
    }

    fn update_delta(&mut self, params: &na::DVector<f64>) -> f64;
}
