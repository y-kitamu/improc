use nalgebra::{matrix, ComplexField, Matrix2, Matrix2x3, Scalar};

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

#[cfg(test)]
mod tests {
    use nalgebra::{matrix, Matrix2x3};

    use super::inv_affine_mat;

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

    fn test_inv_affine_mat_scale() {
        // let shift: Matrix2x3<f32> = matrix![

        // ]
    }
}
