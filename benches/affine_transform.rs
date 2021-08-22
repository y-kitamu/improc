//! Benchmark for the speed of accessing affine matrix in affine transformation in `nalgebla`.
//! Compare a method of using `clone()` with adding trait bound of `std::ops::Add` and `std::ops::Mul`
//!
//! # Result:
//! `affine_trans_with_clone` is slightly fast than `affine_trans_with_ref`
use std::ops::{Add, Mul};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use nalgebra::{matrix, vector, ComplexField, Matrix2, Matrix2x3, Scalar, Vector2};

fn affine_trans_with_clone<T>(aff_mat: &Matrix2x3<T>, pt: &Vector2<T>) -> Vector2<T>
where
    T: Scalar + ComplexField,
{
    vector![
        aff_mat.m11.clone() * pt.x.clone()
            + aff_mat.m12.clone() * pt.y.clone()
            + aff_mat.m13.clone(),
        aff_mat.m21.clone() * pt.x.clone()
            + aff_mat.m22.clone() * pt.y.clone()
            + aff_mat.m23.clone()
    ]
}

fn affine_trans_with_ref<T>(aff_mat: &Matrix2x3<T>, pt: &Vector2<T>) -> Vector2<T>
where
    T: Scalar + ComplexField + Add + Mul + Copy,
{
    vector![
        aff_mat.m11 * pt.x + aff_mat.m12 * pt.y + aff_mat.m13,
        aff_mat.m21 * pt.x + aff_mat.m22 * pt.y + aff_mat.m23
    ]
}

pub fn bench_affine_transform(c: &mut Criterion) {
    #[rustfmt::skip]
    let aff_mat = matrix![
        1.0, 2.0, 3.0;
        -4.0, -5.0, -2.0;
    ];
    let vec = vector![1.5, -2.4];

    let mut group = c.benchmark_group("image_access");
    group.bench_function("affine_trans_with_clone", |b| {
        b.iter(|| affine_trans_with_clone(black_box(&aff_mat), black_box(&vec)))
    });
    group.bench_function("affine_trans_with_ref", |b| {
        b.iter(|| affine_trans_with_ref(black_box(&aff_mat), black_box(&vec)))
    });
}

criterion_group!(benches, bench_affine_transform);
criterion_main!(benches);
