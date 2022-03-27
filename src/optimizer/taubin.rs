//! Implementation for Taubin method and renormalization.
use anyhow::Result;
use nalgebra as na;

use crate::linalg::{get_zero_mat, matrix::constrained_lstsq};

use super::ObservedData;

const MAX_ITERATION: usize = 100;
const STOP_THRESHOLD: f64 = 1e-7;

pub fn taubin<'a, DataClass: ObservedData<'a>>(
    data: &'a [na::Point2<f64>],
) -> Result<na::DVector<f64>> {
    let data_container = DataClass::new(data);
    let weights = vec![1.0; data_container.len()];
    taubin_with_weight::<DataClass>(data, &weights)
}

pub fn taubin_with_weight<'a, DataClass: ObservedData<'a>>(
    data: &'a [na::Point2<f64>],
    weights: &[f64],
) -> Result<na::DVector<f64>> {
    let data_container = DataClass::new(data);
    let vec_size = data_container.vec_size();
    let mat = data_container.matrix(weights);
    let var_mat = (0..data_container.len()).fold(get_zero_mat(vec_size), |acc, idx| {
        let var = data_container.variance(idx);
        let w = weights[idx];
        acc + w * 4.0 * var
    }) / data_container.len() as f64;
    constrained_lstsq(
        &na::DMatrix::from_column_slice(vec_size, vec_size, mat.as_slice()),
        &na::DMatrix::from_column_slice(vec_size, vec_size, var_mat.as_slice()),
    )
}

pub fn renormalization<'a, DataClass: ObservedData<'a>>(
    data: &'a [na::Point2<f64>],
) -> Result<na::DVector<f64>> {
    let mut params = taubin::<DataClass>(data)?;
    let mut previous: na::DVector<f64> =
        na::DVector::<f64>::from_iterator(params.len(), (0..params.len()).map(|_| 0.0));

    let data_container = DataClass::new(data);
    for _ in 1..MAX_ITERATION {
        if previous[0] * params[0] < 0.0 {
            previous *= -1.0;
        }
        if (params.clone() - previous).norm() < STOP_THRESHOLD {
            break;
        }
        let weight = data_container.weights(&params);
        previous = params.clone();
        params = taubin_with_weight::<DataClass>(data, &weight)?;
    }
    Ok(params)
}
