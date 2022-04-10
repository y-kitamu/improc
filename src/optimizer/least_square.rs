//! Implementation of least square minimization algorithm.
use anyhow::Result;
use nalgebra as na;

use crate::linalg::matrix::lstsq;

use super::ObservedData;

const MAX_ITERATION: usize = 4;
const STOP_THRESHOLD: f64 = 1e-5;

pub fn least_square_fitting<'a, DataClass: ObservedData<'a>>(
    data: &'a [na::Point2<f64>],
) -> Result<na::DVector<f64>> {
    let data_container = DataClass::new(data);
    let weights: Vec<f64> = vec![1.0; data_container.len() * data_container.num_equation().pow(2)];
    least_square_fitting_with_weight::<DataClass>(data, &weights)
}

fn least_square_fitting_with_weight<'a, DataClass: ObservedData<'a>>(
    data: &'a [na::Point2<f64>],
    weights: &[f64],
) -> Result<na::DVector<f64>> {
    let data_container = DataClass::new(data);
    let mat = data_container.matrix(weights);
    lstsq(&mat)
}

pub fn iterative_reweight<'a, DataClass: ObservedData<'a>>(
    data: &'a [na::Point2<f64>],
) -> Result<na::DVector<f64>> {
    let data_container = DataClass::new(data);
    // calculate first iteration
    let default_weights: Vec<f64> =
        vec![1.0; data_container.len() * data_container.num_equation().pow(2)];
    let mut params = least_square_fitting_with_weight::<DataClass>(data, &default_weights)?;
    let mut previous: na::DVector<f64> =
        na::DVector::<f64>::from_iterator(params.len(), (0..params.len()).map(|_| 0.0));
    // calculate residual (for avoiding instability caused by SVD)
    let mut residual = &params.transpose() * &data_container.matrix(&default_weights) * &params;

    for _ in 0..MAX_ITERATION {
        if previous[0] * params[0] < 0.0 {
            params *= -1.0;
        }
        if (&params - &previous).norm() < STOP_THRESHOLD {
            break;
        }
        let weights = data_container.weights(&params);
        previous = params.clone();
        let mat = data_container.matrix(&weights);
        let updated = lstsq(&mat)?;
        // check whether residual is decreasing
        {
            let res = &updated.transpose() * &mat * &updated;
            if res > residual * 10.0 {
                println!("Residual is not decreasing. Break iteration.");
                break;
            }
            residual = res;
        }
        params = updated;
    }
    Ok(params)
}
