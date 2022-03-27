//! Calculate fundamental matrix
use anyhow::Result;
use nalgebra as na;

use crate::optimizer::ObservedData;

struct FundamentalMatrixData<'a> {
    data: &'a [na::Point2<f64>],
    scale: f64,
}

impl<'a> ObservedData<'a> for FundamentalMatrixData<'a> {
    fn new(data: &'a [na::Point2<f64>]) -> Self {
        todo!()
    }

    fn len(&self) -> usize {
        self.data[0].len()
    }

    fn vector(&self, data_index: usize) -> na::DVector<f64> {
        todo!()
    }

    fn matrix(&self, weight_vector: &[f64]) -> na::DMatrix<f64> {
        todo!()
    }

    fn variance(&self, data_index: usize) -> na::DMatrix<f64> {
        todo!()
    }

    fn weights(&self, params: &na::DVector<f64>) -> Vec<f64> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_least_square() {}
}
