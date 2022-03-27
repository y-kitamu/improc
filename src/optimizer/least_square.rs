//! Implementation of least square minimization algorithm.
use anyhow::Result;
use nalgebra as na;

use crate::linalg::matrix::lstsq;

use super::ObservedData;

const MAX_ITERATION: usize = 100;
const STOP_THRESHOLD: f64 = 1e-7;

pub fn least_square_fitting<'a, DataClass: ObservedData<'a>>(
    data: &'a [na::Point2<f64>],
) -> Result<na::DVector<f64>> {
    let weights: Vec<f64> = vec![1.0; data.len()];
    least_square_fitting_with_weight::<DataClass>(data, &weights)
}

pub fn least_square_fitting_with_weight<'a, DataClass: ObservedData<'a>>(
    data: &'a [na::Point2<f64>],
    weight: &[f64],
) -> Result<na::DVector<f64>> {
    let data_container = DataClass::new(data);
    let mat = data_container.matrix(weight);
    lstsq(&mat)
}

pub fn iterative_reweight<'a, DataClass: ObservedData<'a>>(
    data: &'a [na::Point2<f64>],
) -> Result<na::DVector<f64>> {
    let mut params = least_square_fitting::<DataClass>(data)?;
    let mut previous: na::DVector<f64> =
        na::DVector::<f64>::from_iterator(params.len(), (0..params.len()).map(|_| 0.0));

    let data_container = DataClass::new(data);
    for _ in 0..MAX_ITERATION {
        if previous[0] * params[0] < 0.0 {
            previous *= -1.0;
        }
        if (params.clone() - previous).norm() < STOP_THRESHOLD {
            break;
        }
        let weights = data_container.weights(&params);
        previous = params.clone();
        params = least_square_fitting_with_weight::<DataClass>(data, &weights).unwrap();
    }
    Ok(params)
}
