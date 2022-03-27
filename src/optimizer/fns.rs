//! Implementation of FNS (Fundamental Numerial Scheme)
use anyhow::Result;
use nalgebra as na;

use crate::linalg::{get_zero_mat, matrix::lstsq};

use super::ObservedData;

const MAX_ITERATION: usize = 100;
const STOP_THRESHOLD: f64 = 1e-7;

pub fn fns<'a, DataClass: ObservedData<'a>>(
    data: &'a [na::Point2<f64>],
) -> Result<na::DVector<f64>> {
    let data_container = DataClass::new(data);
    let mut previous = na::DVector::<f64>::from_vec(vec![0.0; data_container.vec_size()]);
    let mut params = step(&data_container, &previous)?;

    for _ in 0..MAX_ITERATION {
        if previous[0] * params[0] < 0.0 {
            previous *= -1.0;
        }
        if (params.clone() - previous.clone()).norm() < STOP_THRESHOLD {
            break;
        }
        previous = params.clone();
        params = step(&data_container, &params)?;
    }
    Ok(params)
}

fn step<'a, DataClass: ObservedData<'a>>(
    data: &DataClass,
    params: &na::DVector<f64>,
) -> Result<na::DVector<f64>> {
    let vec_size = data.vec_size();
    let weights = data.weights(params);
    let m = data.matrix(&weights);
    let l = (0..data.len())
        .zip(weights.iter())
        .fold(get_zero_mat(vec_size), |acc, (idx, w)| {
            let xi = data.vector(idx);
            let vm = data.variance(idx);
            let dot = params.dot(&xi);
            acc + (*w) * dot * dot * vm
        })
        / data.len() as f64;
    lstsq(&na::DMatrix::<f64>::from_column_slice(
        vec_size,
        vec_size,
        (m - l).data.as_slice(),
    ))
}
