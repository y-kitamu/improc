use anyhow::Result;
use nalgebra as na;

pub fn projective_self_calibration(
    observed_points: &[Vec<na::Point2<f64>>],
) -> Result<(na::DMatrix<f64>, na::DMatrix<f64>)> {
    projective_reconstruction();
    euclide_reconstruction();
}

fn projective_reconstruction() {}

fn primary_method() {}

fn dual_method() {}

fn euclide_reconstruction() {}
