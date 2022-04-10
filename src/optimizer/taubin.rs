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
    let weights = vec![1.0; data_container.len() * data_container.num_equation().pow(2)];
    taubin_with_weight::<DataClass>(data, &weights)
}

pub fn renormalization<'a, DataClass: ObservedData<'a>>(
    data: &'a [na::Point2<f64>],
) -> Result<na::DVector<f64>> {
    let mut params = taubin::<DataClass>(data)?;
    let mut previous: na::DVector<f64> =
        na::DVector::<f64>::from_iterator(params.len(), (0..params.len()).map(|_| 0.0));
    let data_container = DataClass::new(data);
    // calculate residual (for avoiding instability caused by SVD)
    let default_matrix = data_container.matrix(&vec![
        1.0;
        data_container.len()
            * data_container.num_equation().pow(2)
    ]);
    let mut residual = &params.transpose() * &default_matrix * &params;

    for _ in 1..MAX_ITERATION {
        if previous[0] * params[0] < 0.0 {
            previous *= -1.0;
        }
        if (params.clone() - previous).norm() < STOP_THRESHOLD {
            break;
        }
        let weights = data_container.weights(&params);
        previous = params.clone();
        let updated = taubin_with_weight::<DataClass>(data, &weights)?;
        // check whether residual is decreasing
        {
            let res = &updated.transpose() * &default_matrix * &updated;
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

fn taubin_with_weight<'a, DataClass: ObservedData<'a>>(
    data: &'a [na::Point2<f64>],
    weights: &[f64],
) -> Result<na::DVector<f64>> {
    let data_container = DataClass::new(data);
    let vec_size = data_container.vec_size();
    let num_eqs = data_container.num_equation();
    let num_eqs_square = num_eqs.pow(2);
    let mat = data_container.matrix(weights);
    let var_mat = (0..data_container.len()).fold(get_zero_mat(vec_size), |acc, idx| {
        acc + (0..num_eqs)
            .map(|i| {
                (0..num_eqs)
                    .map(|j| {
                        let k = idx * num_eqs_square + i * num_eqs + j;
                        let var = data_container.variance(k);
                        let w = weights[k];
                        w * 4.0 * var
                    })
                    .sum::<na::DMatrix<f64>>()
            })
            .sum::<na::DMatrix<f64>>()
    }) / data_container.len() as f64;
    constrained_lstsq(&mat, &var_mat)
}
