use std::ops::{Add, Mul};

use nalgebra::{matrix, vector, ComplexField, Matrix2, Matrix2x3, Scalar, Vector2};

/// calculate inverse of given affine matrix.
pub fn inv_affine_mat<T: Scalar + ComplexField>(affine_mat: &Matrix2x3<T>) -> Matrix2x3<T> {
    let mut inv_affine_mat = Matrix2x3::<T>::zeros();
    inv_affine_mat.fill_with_identity();
    inv_affine_mat.m13 = -affine_mat.m13.clone();
    inv_affine_mat.m23 = -affine_mat.m23.clone();
    let rot_scale: Matrix2<T> = matrix![
        affine_mat.m11.clone(), affine_mat.m12.clone();
        affine_mat.m21.clone(), affine_mat.m22.clone();
    ];
    let rot_scale = rot_scale.try_inverse().unwrap();
    let inv_affine_mat = rot_scale * inv_affine_mat;
    inv_affine_mat
}

/// merge affine transforms
pub fn merge_affine_transforms<T: Scalar + ComplexField>(
    lhs: &Matrix2x3<T>,
    rhs: &Matrix2x3<T>,
) -> Matrix2x3<T> {
    let rot_scale: Matrix2<T> = matrix![
        lhs.m11.clone(), lhs.m12.clone();
        lhs.m21.clone(), lhs.m22.clone();
    ];
    let mut merged = rot_scale * rhs;
    merged.m13 += lhs.m13.clone();
    merged.m23 += lhs.m23.clone();
    merged
}

///
pub fn affine_transform<'a, T>(affine_mat: &Matrix2x3<T>, pt: &Vector2<T>) -> Vector2<T>
where
    T: Scalar + ComplexField + Add + Mul + Copy,
{
    let vec: Vector2<T> = vector![
        affine_mat.m11 * pt.x + affine_mat.m12 * pt.y + affine_mat.m13,
        affine_mat.m21 * pt.x + affine_mat.m22 * pt.y + affine_mat.m23
    ];
    vec
}

/// Args:
/// - rotation_degree : rotation angle in degree.
/// - center : rotation center (x, y).
/// - scale :
pub fn get_rotation_matrix(rotation_degree: f32, center: (f32, f32), scale: f32) -> Matrix2x3<f32> {
    let rad = rotation_degree / 180.0 * std::f32::consts::PI;
    let sin = scale * rad.sin();
    let cos = scale * rad.cos();
    let (dx, dy) = center;
    matrix![
        cos, -sin, dx - cos * dx + sin * dy;
        sin, cos, dy - sin * dx - cos * dy;
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_affine_transform() {
        let affine_mat: Matrix2x3<f32> = matrix![
            2.0, 0.5, 1.0;
            -1.0, 0.8, -1.5;
        ];
        let pt: Vector2<f32> = vector![1.0, -0.5];
        let dst = affine_transform(&affine_mat, &pt);
        assert!((dst.x - 2.75).abs() < 1e-5);
        assert!((dst.y + 2.9).abs() < 1e-5);
    }

    #[test]
    fn test_inv_affine_mat_shift() {
        #[rustfmt::skip]
        let shift: Matrix2x3<f32> = matrix![
            1.0, 0.0, 5.0;
            0.0, 1.0, -4.0;
        ];
        let inv = inv_affine_mat(&shift);
        assert!((inv.m11 - 1.0).abs() < 1e-5);
        assert!((inv.m12 - 0.0).abs() < 1e-5);
        assert!((inv.m13 + 5.0).abs() < 1e-5);
        assert!((inv.m21 - 0.0).abs() < 1e-5);
        assert!((inv.m22 - 1.0).abs() < 1e-5);
        assert!((inv.m23 - 4.0).abs() < 1e-5);
    }

    #[test]
    fn test_inv_affine_mat_scale() {
        let scale: Matrix2x3<f32> = matrix![
            2.0, 0.0, 1.0;
            0.0, -0.8, -3.0;
        ];
        let inv = inv_affine_mat(&scale);
        assert!((inv.m11 - 0.5).abs() < 1e-5, "inv.m11 = {}", inv.m11);
        assert!((inv.m22 + 1.25).abs() < 1e-5, "inv.m22 = {}", inv.m22);

        let inv0 = merge_affine_transforms(&scale, &inv);
        let inv1 = merge_affine_transforms(&inv, &scale);
        assert!((inv0.m11 - 1.0).abs() < 1e-5, "inv0.m11 = {}", inv0.m11);
        assert!((inv0.m12 - 0.0).abs() < 1e-5, "inv0.m11 = {}", inv0.m12);
        assert!((inv0.m13 - 0.0).abs() < 1e-5, "inv0.m11 = {}", inv0.m13);
        assert!((inv0.m21 - 0.0).abs() < 1e-5, "inv0.m11 = {}", inv0.m21);
        assert!((inv0.m22 - 1.0).abs() < 1e-5, "inv0.m11 = {}", inv0.m22);
        assert!((inv0.m23 - 0.0).abs() < 1e-5, "inv0.m11 = {}", inv0.m23);

        assert!(
            (inv0.m11 - inv1.m11).abs() < 1e-5,
            "inv0.m11 = {}, inv1.m11 = {}",
            inv0.m11,
            inv1.m11
        );
        assert!(
            (inv0.m12 - inv1.m12).abs() < 1e-5,
            "inv0.m12 = {}, inv1.m12 = {}",
            inv0.m12,
            inv1.m12
        );
        assert!(
            (inv0.m13 - inv1.m13).abs() < 1e-5,
            "inv0.m13 = {}, inv1.m13 = {}",
            inv0.m13,
            inv1.m13
        );
        assert!(
            (inv0.m21 - inv1.m21).abs() < 1e-5,
            "inv0.m21 = {}, inv1.m21 = {}",
            inv0.m21,
            inv1.m21
        );
        assert!(
            (inv0.m22 - inv1.m22).abs() < 1e-5,
            "inv0.m22 = {}, inv1.m22 = {}",
            inv0.m22,
            inv1.m22
        );
        assert!(
            (inv0.m23 - inv1.m23).abs() < 1e-5,
            "inv0.m23 = {}, inv1.m23 = {}",
            inv0.m23,
            inv1.m23
        );
    }

    #[test]
    fn test_merge_affine_transform() {
        let lhs: Matrix2x3<f32> = matrix![
            -0.8, 0.5, 1.0;
            0.0, 2.0, -0.5;
        ];
        let rhs: Matrix2x3<f32> = matrix![
            -0.5, 0.0, 10.0;
            1.0, 3.0, 2.0;
        ];
        let merged = merge_affine_transforms(&lhs, &rhs);
        assert!((merged.m11 - 1.3) < 1e-5, "merged.m11 = {}", merged.m11);
        assert!((merged.m12 - 1.5) < 1e-5, "merged.m12 = {}", merged.m12);
        assert!((merged.m13 + 6.0) < 1e-5, "merged.m13 = {}", merged.m13);
        assert!((merged.m21 - 2.0) < 1e-5, "merged.m21 = {}", merged.m21);
        assert!((merged.m22 - 6.0) < 1e-5, "merged.m22 = {}", merged.m22);
        assert!((merged.m23 - 3.5) < 1e-5, "merged.m23 = {}", merged.m23);
    }

    #[test]
    fn test_get_rotation_matrix() {
        let rot_degree = 60.0f32;
        let rot_rad = rot_degree / 180.0 * std::f32::consts::PI;
        let center = (5.0f32, 10.0f32);
        let affine_mat = get_rotation_matrix(rot_degree, center, 1.0);

        assert!(
            (affine_mat.m11 - rot_rad.cos()).abs() < 1e-5,
            "m11 = {}",
            affine_mat.m11
        );
        assert!(
            (affine_mat.m12 + rot_rad.sin()).abs() < 1e-5,
            "m12 = {}",
            affine_mat.m12
        );
        assert!(
            (affine_mat.m21 - rot_rad.sin()).abs() < 1e-5,
            "m21 = {}",
            affine_mat.m21
        );
        assert!(
            (affine_mat.m22 - rot_rad.cos()).abs() < 1e-5,
            "m22 = {}",
            affine_mat.m22
        );

        let pt: Vector2<f32> = vector![6.0f32, 10.0f32];
        let dst = affine_transform(&affine_mat, &pt);
        assert!((dst.x - 5.5).abs() < 1e-5);
        let pt: Vector2<f32> = vector![5.0f32, 5.0f32];
        let dst = affine_transform(&affine_mat, &pt);
        assert!((dst.y - 7.5).abs() < 1e-5);
    }
}
