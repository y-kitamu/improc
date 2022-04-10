//! Trait definitions for optimization problems.
use nalgebra as na;

pub mod fns;
pub mod geometric;
pub mod least_square;
pub mod taubin;

/// Data trait definition
pub trait ObservedData<'a> {
    /// constructor
    fn new(data: &'a [na::Point2<f64>]) -> Self;
    /// Return the number of the observed points in one image.
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn vector(&self, data_index: usize) -> na::DVector<f64>;
    fn matrix(&self, weight_vector: &[f64]) -> na::DMatrix<f64>;
    /// Return covariance matrix of the data specified by `data_index`.
    fn variance(&self, data_index: usize) -> na::DMatrix<f64>;
    /// Return weights vector of each data.
    fn weights(&self, params: &na::DVector<f64>) -> Vec<f64>;
    fn vec_size(&self) -> usize {
        self.vector(0).nrows()
    }
    fn num_equation(&self) -> usize {
        1
    }
    fn update_delta(&mut self, params: &na::DVector<f64>) -> f64;
    /// Return all data
    fn get_data(&self) -> Vec<na::Point2<f64>>;
}
