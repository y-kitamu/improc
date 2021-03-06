use anyhow::Result;
use nalgebra as na;

use super::{fns::minimize_sampson_error, ObservedData};

const MAX_ITERATION: usize = 5;
const STOP_THRESHOLD: f64 = 1e-4;

pub fn minimize_geometric_distance<'a, DataClass: ObservedData<'a>>(
    data: &'a [na::Point2<f64>],
) -> Result<na::DVector<f64>> {
    let params = na::DVector::<f64>::from_vec(vec![0.0; DataClass::new(data).vec_size()]);
    let (params, _) = minimize_geometric_distance_impl::<DataClass>(data, &params, true)?;
    Ok(params)
}

pub fn minimize_geometric_distance_impl<'a, DataClass: ObservedData<'a>>(
    data: &'a [na::Point2<f64>],
    params: &na::DVector<f64>,
    update_params: bool,
) -> Result<(na::DVector<f64>, Vec<na::Point2<f64>>)> {
    let mut data_container = DataClass::new(data);
    let mut params = params.clone();

    for _ in 0..MAX_ITERATION {
        if update_params {
            params = minimize_sampson_error(&data_container, &params)?;
        }
        let gerror = data_container.update_delta(&params);

        if gerror < STOP_THRESHOLD {
            break;
        }
    }

    Ok((params, data_container.get_data()))
}
