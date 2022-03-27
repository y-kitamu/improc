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
