//! Implementation of FNS (Fundamental Numerial Scheme)
use anyhow::Result;
use nalgebra as na;

use crate::{
    linalg::{get_zero_mat, matrix::lstsq},
    PrintDebug,
};

use super::ObservedData;

const MAX_ITERATION: usize = 5;
const STOP_THRESHOLD: f64 = 1e-7;

pub fn fns<'a, DataClass: ObservedData<'a>>(
    data: &'a [na::Point2<f64>],
) -> Result<na::DVector<f64>> {
    let data_container = DataClass::new(data);
    let mut previous = na::DVector::<f64>::from_vec(vec![0.0; data_container.vec_size()]);
    let mut params = minimize_sampson_error(&data_container, &previous)?;
    // calculate residual (for avoiding instability caused by SVD)
    let default_matrix = data_container.matrix(&vec![
        1.0;
        data_container.len()
            * data_container.num_equation().pow(2)
    ]);
    let mut residual = params.dot(&(&default_matrix * &params));

    for _ in 0..MAX_ITERATION {
        if previous[0] * params[0] < 0.0 {
            params *= -1.0;
        }
        if (params.clone() - previous.clone()).norm() < STOP_THRESHOLD {
            break;
        }
        previous = params.clone();
        let updated = minimize_sampson_error(&data_container, &params)?;
        // check whether residual is decreasing
        {
            let res = updated.dot(&(&default_matrix * &updated));
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

pub fn minimize_sampson_error<'a, DataClass: ObservedData<'a>>(
    data_container: &DataClass,
    params: &na::DVector<f64>,
) -> Result<na::DVector<f64>> {
    let vec_size = data_container.vec_size();
    let num_eqs = data_container.num_equation();
    let num_eqs_square = num_eqs.pow(2);
    let weights = data_container.weights(params);
    let m = data_container.matrix(&weights);
    let l = (0..data_container.len()).fold(get_zero_mat(vec_size), |acc, idx| {
        let vs: Vec<f64> = (0..data_container.num_equation())
            .map(|i| {
                (0..data_container.num_equation())
                    .map(|j| {
                        let w = weights[idx * num_eqs_square + i * num_eqs + j];
                        let xi = data_container.vector(idx * num_eqs + j);
                        w * xi.dot(params)
                    })
                    .sum()
            })
            .collect();
        acc + (0..num_eqs)
            .map(|i| {
                (0..num_eqs)
                    .map(|j| {
                        let vm = data_container.variance(idx * num_eqs_square + i * num_eqs + j);
                        vs[i] * vs[j] * vm
                    })
                    .sum::<na::DMatrix<f64>>()
            })
            .sum::<na::DMatrix<f64>>()
    }) / (data_container.len() as f64 * 9.0);
    lstsq(&na::DMatrix::<f64>::from_column_slice(
        vec_size,
        vec_size,
        (m - l).as_slice(),
    ))
}
