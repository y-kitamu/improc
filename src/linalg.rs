use nalgebra as na;

pub mod affine;
pub use affine::{get_rotation_matrix, inv_affine_mat, merge_affine_transforms, warp_point};
pub mod homography;
pub mod matrix;
pub mod ransac;

pub fn get_identity_mat(size: usize) -> na::DMatrix<f64> {
    na::DMatrix::from_diagonal_element(size, size, 1.0)
}

pub fn get_zero_mat(size: usize) -> na::DMatrix<f64> {
    na::DMatrix::from_diagonal_element(size, size, 0.0)
}

pub fn get_rotation_matrix_from_omega(omega: &[f64]) -> na::DMatrix<f64> {
    let norm = (omega[0] * omega[0] + omega[1] * omega[1] + omega[2] * omega[2]).sqrt();
    let l1 = omega[0] / norm;
    let l2 = omega[1] / norm;
    let l3 = omega[2] / norm;
    let cos = norm.cos();
    let sin = norm.sin();
    #[rustfmt::skip]
    let rot = na::DMatrix::<f64>::from_row_slice(3, 3, &[
        cos + l1 * l1 * (1.0 - cos), l1 * l1 * (1.0 - cos) - l3 * sin, l1 * l1 * (1.0 - cos) + l2 * sin,
        l2 * l1 * (1.0 - cos) + l3 * sin, cos + l2 * l2 * (1.0 - cos), l2 * l3 * (1.0 - cos) - l1 * sin,
        l3 * l1 * (1.0 - cos) - l2 * sin, l3 * l2 * (1.0 - cos) + l1 * sin, cos + l3 * l3 * (1.0 - cos),
    ]);
    rot
}
