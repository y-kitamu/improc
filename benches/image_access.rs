//! Benchmark for the speed of accessing pixel data in an image.
//! Compare speeds with different methods.
//! The best method at the moment is to get the raw vector and
//! access pixel data from it.
//! TODO: compare speed with c++ raw pointer.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use image::{DynamicImage, GenericImageView, ImageBuffer};

// access from `image` crate api.
fn get_sum0(image: &DynamicImage) -> u32 {
    let mut sum: u32 = 0;
    for y in 0..image.height() {
        for x in 0..image.width() {
            sum += image.get_pixel(x, y).0[0] as u32;
        }
    }
    sum
}

// access from raw vector.
fn get_sum1(image: &DynamicImage) -> u32 {
    let mut sum: u32 = 0;
    let raw = image.as_rgba8().unwrap().as_raw();
    for y in 0..image.height() {
        let y_offset = y * image.width() * 4;
        for x in 0..image.width() {
            sum += raw[(y_offset + x * 4) as usize] as u32;
        }
    }
    sum
}

// access from vector api.
fn get_sum2(image: &DynamicImage) -> u32 {
    let mut sum: u32 = 0;
    let raw = image.as_rgba8().unwrap().as_raw();
    for y in 0..image.height() {
        let y_offset = y * image.width() * 4;
        for x in 0..image.width() {
            sum += *raw.get((y_offset + x * 4) as usize).unwrap() as u32;
        }
    }
    sum
}

// criterionの動作確認
pub fn criterion_benchmark(c: &mut Criterion) {
    let image_file = "/home/kitamura/work/Projects/improc/data/sample_image/surface.png";
    let image = image::open(image_file).unwrap();

    c.bench_function("get_image_sum", |b| b.iter(|| get_sum0(black_box(&image))));
}

// compare speed between different access to image data.
pub fn bench_image_access(c: &mut Criterion) {
    let image_file = "/home/kitamura/work/Projects/improc/data/sample_image/surface.png";
    let image = image::open(image_file).unwrap();

    let mut group = c.benchmark_group("image_access");

    group.bench_function("get_image_sum0", |b| b.iter(|| get_sum0(black_box(&image))));
    group.bench_function("get_image_sum1", |b| b.iter(|| get_sum1(black_box(&image))));
    group.bench_function("get_image_sum2", |b| b.iter(|| get_sum2(black_box(&image))));
}

criterion_group!(benches, bench_image_access);
criterion_main!(benches);
