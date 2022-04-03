use anyhow::Result;
use nalgebra as na;

use super::{fns::minimize_sampson_error, ObservedData};

const MAX_ITERATION: usize = 100;
const STOP_THRESHOLD: f64 = 1e-7;

pub fn minimize_geometric_distance<'a, DataClass: ObservedData<'a>>(
    data: &'a [na::Point2<f64>],
) -> Result<na::DVector<f64>> {
    let mut geo_error = 1e9;
    let mut data_container = DataClass::new(data);
    let mut params = na::DVector::<f64>::from_vec(vec![0.0; data_container.vec_size()]);

    for _ in 0..MAX_ITERATION {
        params = minimize_sampson_error(&data_container, &params)?;
        let gerror = data_container.update_delta(&params);

        if gerror / geo_error > 1.1 {
            break;
        }
        geo_error = gerror;
    }

    Ok(params)
}
