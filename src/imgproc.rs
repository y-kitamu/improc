use std::ops::Deref;

use image::{ColorType, ImageBuffer, Pixel};
use nalgebra::{vector, Matrix2x3};
use num_traits::ToPrimitive;

use crate::feat::keypoints::KeyPoint;

use super::{linalg, linalg::inv_affine_mat};

/// affine transformation (linear interpolation)
/// `affine_mat` is projection from source points to destination points
pub fn affine_transform<P, Container>(
    img: &ImageBuffer<P, Container>,
    affine_mat: &Matrix2x3<f32>,
) -> Vec<u8>
where
    P: Pixel + 'static,
    P::Subpixel: 'static,
    Container: Deref<Target = [P::Subpixel]>,
{
    let inv_affine_mat = inv_affine_mat(&affine_mat);
    let data = img.as_raw();
    let mut transformed: Vec<u8> = Vec::with_capacity(data.len());
    let x_stride = P::CHANNEL_COUNT as usize;
    let y_stride = x_stride * img.width() as usize;

    for y in 0..img.height() {
        for x in 0..img.width() {
            let pt = linalg::affine_transform(&inv_affine_mat, &vector![x as f32, y as f32]);
            // TODO: functionalize
            let mut ix = pt.x.floor() as isize;
            let mut iy = pt.y.floor() as isize;
            let mut fx = pt.x.clone() - ix as f32;
            let mut fy = pt.y.clone() - iy as f32;
            if ix < 0 {
                ix = 0;
                fx = 0.0f32;
            }
            if ix >= (img.width() - 1) as isize {
                ix = img.width() as isize - 2;
                fx = 1.0f32;
            }
            if iy < 0 {
                iy = 0;
                fy = 0.0f32;
            }
            if iy >= (img.height() - 1) as isize {
                iy = img.height() as isize - 2;
                fy = 1.0f32;
            }
            for c in 0..x_stride {
                let offset = iy as usize * y_stride + ix as usize * x_stride + c;
                let val = (1.0f32 - fx) * (1.0f32 - fy) * data[offset].to_f32().unwrap()
                    + fx * (1.0f32 - fy) * data[offset + x_stride].to_f32().unwrap()
                    + (1.0f32 - fx) * fy * data[offset + y_stride].to_f32().unwrap()
                    + fx * fy * data[offset + y_stride + x_stride].to_f32().unwrap();
                transformed.push(val as u8);
            }
        }
    }
    transformed
}

/// Non-Maximum Supression (NMS)
// とりあえず、O(n^2)で実装してみて高速化を検討する
pub fn nms(kpts: &Vec<KeyPoint>, kernel_size: u32) -> Vec<KeyPoint> {
    if kpts.len() == 0 {
        return Vec::<KeyPoint>::new();
    }
    let half = kernel_size as f32 / 2.0;
    let mut kpts = kpts.clone();
    kpts.sort_unstable_by(|a, b| a.crf().partial_cmp(&b.crf()).unwrap());

    let mut supressed: Vec<KeyPoint> = Vec::new();
    // println!("len = {}", kpts.len());
    'outer: for i in (0..kpts.len()).rev() {
        // println!("{}", kpts[i].crf());
        for kpt in &supressed {
            if (kpt.x() - kpts[i].x()).abs() < half && (kpt.y() - kpts[i].y()).abs() < half {
                continue 'outer;
            }
        }
        supressed.push(kpts[i]);
    }
    supressed
}

/// gaussian filter
// TODO: dftによる高速化
// : http://signalprocess.binarized.work/2019/04/01/optimize_any_fir_filter_calculation_by_dft/
pub fn gaussian<P, Container>(
    img: &ImageBuffer<P, Container>,
    kernel_size: u32,
    sigma: f32, // stddev
) -> Vec<u8>
where
    P: Pixel + 'static,
    P::Subpixel: 'static,
    Container: Deref<Target = [P::Subpixel]>,
{
    let (width, height) = (img.width() as usize, img.height() as usize);
    // let data = img.as_raw();
    let data = padding(img, kernel_size as usize / 2);
    let x_stride = P::CHANNEL_COUNT as usize; //
    let y_stride = (width + kernel_size as usize / 2 * 2) * x_stride;
    let mut res: Vec<u8> = Vec::with_capacity(height * y_stride);
    let kernel = create_gauss_kernel(kernel_size, sigma);

    for y in 0..height {
        for x in 0..width {
            let mut sums: Vec<f32> = vec![0.0; x_stride];
            for dy in 0..kernel_size as usize {
                let y_off = y_stride * (y + dy);
                for dx in 0..kernel_size as usize {
                    let offset = y_off + (x + dx) * x_stride;
                    let kval = kernel[dy * kernel_size as usize + dx];
                    for c in 0..x_stride {
                        sums[c] += kval * data[offset + c].to_f32().unwrap();
                    }
                }
            }
            for c in 0..x_stride {
                res.push(sums[c].round() as u8);
            }
        }
    }
    res
}

fn create_gauss_kernel(kernel_size: u32, sigma: f32) -> Vec<f32> {
    let mut kernel: Vec<f32> = Vec::with_capacity((kernel_size * kernel_size) as usize);
    let half = (kernel_size / 2) as isize;
    let denomi = 1.0 / (2.0 * sigma * sigma);

    let mut sum = 0.0f32;
    for y in -half..=half {
        for x in -half..=half {
            let val = (-(x * x + y * y) as f32 * denomi).exp();
            kernel.push(val);
            sum += val;
        }
    }
    let scale = 1.0 / sum;
    let kernel = kernel.iter().map(|val| val * scale).collect();
    kernel
}

fn padding<P, Container>(img: &ImageBuffer<P, Container>, pad_size: usize) -> Vec<u8>
where
    P: Pixel + 'static,
    P::Subpixel: 'static,
    Container: Deref<Target = [P::Subpixel]>,
{
    let (width, height) = (img.width() as usize, img.height() as usize);
    let data = img.as_raw();
    let x_stride = P::CHANNEL_COUNT as usize;
    let src_y_stride = width * x_stride;
    let dst_y_stride = (width + pad_size * 2) * x_stride;
    let mut res: Vec<u8> = vec![0; (height + pad_size * 2) * dst_y_stride];

    let lt: Vec<u8> = (0..x_stride)
        .map(|c| data[0 + c].to_u8().unwrap())
        .collect();
    let rt: Vec<u8> = (0..x_stride)
        .map(|c| data[src_y_stride - x_stride + c].to_u8().unwrap())
        .collect();
    let lb: Vec<u8> = (0..x_stride)
        .map(|c| data[(height - 1) * src_y_stride + c].to_u8().unwrap())
        .collect();
    let rb: Vec<u8> = (0..x_stride)
        .map(|c| data[data.len() - x_stride + c].to_u8().unwrap())
        .collect();
    for y in 0..pad_size {
        for x in 0..pad_size {
            for c in 0..x_stride {
                res[y * dst_y_stride + x * x_stride + c] = lt[c];
                res[y * dst_y_stride + (x + width + pad_size) * x_stride + c] = rt[c];
                res[(y + height + pad_size) * dst_y_stride + x * x_stride + c] = lb[c];
                res[(y + height + pad_size) * dst_y_stride
                    + (x + width + pad_size) * x_stride
                    + c] = rb[c];
            }
        }
        let dst_y_off = y * dst_y_stride;
        for x in 0..width {
            for c in 0..x_stride {
                res[dst_y_off + (x + pad_size) * x_stride + c] =
                    data[x * x_stride + c].to_u8().unwrap();
            }
        }
        let src_y_off = (height - 1) * src_y_stride;
        let dst_y_off = (y + height + pad_size) * dst_y_stride;
        for x in 0..width {
            for c in 0..x_stride {
                res[dst_y_off + (x + pad_size) * x_stride + c] =
                    data[src_y_off + x * x_stride + c].to_u8().unwrap();
            }
        }
    }

    for y in 0..height {
        let src_y_off = y * src_y_stride;
        let dst_y_off = (y + pad_size) * dst_y_stride;
        for x in 0..width {
            let src_off = src_y_off + x * x_stride;
            let dst_off = dst_y_off + (x + pad_size) * x_stride;
            for c in 0..x_stride {
                res[dst_off + c] = data[src_off + c].to_u8().unwrap();
            }
        }
        for x in 0..pad_size {
            let dst_off0 = dst_y_off + x * x_stride;
            let dst_off1 = dst_y_off + (x + width + pad_size) * x_stride;
            for c in 0..x_stride {
                res[dst_off0 + c] = data[src_y_off + c].to_u8().unwrap();
                res[dst_off1 + c] = data[src_y_off + src_y_stride - x_stride + c]
                    .to_u8()
                    .unwrap();
            }
        }
    }
    res
}

/// convert to gray scale.
pub fn gray<P, Container>(img: &ImageBuffer<P, Container>) -> Vec<u8>
where
    P: Pixel + 'static,
    P::Subpixel: 'static,
    Container: Deref<Target = [P::Subpixel]>,
{
    let x_stride = P::CHANNEL_COUNT as usize;
    assert!(x_stride == 3 || x_stride == 4);

    let (width, height) = (img.width() as usize, img.height() as usize);
    let y_stride = width * x_stride;
    let data = img.as_raw();
    let mut gray: Vec<u8> = Vec::with_capacity(width * height);
    let mut factor: Vec<f32> = vec![0.299, 0.587, 0.114];
    if P::COLOR_TYPE == ColorType::Bgr8 || P::COLOR_TYPE == ColorType::Bgra8 {
        factor = vec![factor[2], factor[1], factor[0]];
    }

    for y in 0..height {
        let off_y = y_stride * y;
        for x in 0..width {
            let off = off_y + x * x_stride;
            let val = (factor[0] * data[off].to_f32().unwrap()
                + factor[1] * data[off + 1].to_f32().unwrap()
                + factor[2] * data[off + 2].to_f32().unwrap()) as u8;
            gray.push(val);
        }
    }
    gray
}

pub fn median_filter<P, Container>(img: &ImageBuffer<P, Container>, kernel_size: u32) -> Vec<u8>
where
    P: Pixel + 'static,
    P::Subpixel: 'static,
    Container: Deref<Target = [P::Subpixel]>,
{
    let x_stride = P::CHANNEL_COUNT as usize;
    let padded = padding(img, kernel_size as usize / 2 + 1);
    let width = (img.width() + (kernel_size / 2 + 1) * 2) as usize;
    let height = (img.height() + (kernel_size / 2 + 1) * 2) as usize;
    let y_stride = x_stride * width;
    let mut target: Vec<u32> = vec![0; y_stride * height];

    for y in 0..height {
        let y_off = y * y_stride;
        if y > 0 {
            for c in 0..x_stride {
                target[y_off + c] = target[y_off + c - y_stride] + padded[y_off + c] as u32;
            }
        }
        for x in 1..width {
            let off = y_off + x * x_stride;
            if y == 0 {
                for c in 0..x_stride {
                    target[off + c] = target[off + c - x_stride] + padded[off + c] as u32;
                }
            } else {
                for c in 0..x_stride {
                    target[off + c] += target[off + c - x_stride] + target[off + c - y_stride]
                        - target[off + c - x_stride - y_stride]
                        + padded[off + c] as u32;
                }
            }
        }
    }

    let mut dst: Vec<u8> = Vec::with_capacity((img.width() * img.height()) as usize * x_stride);
    let rt_off = kernel_size as usize * x_stride;
    let lb_off = kernel_size as usize * y_stride;
    let rb_off = kernel_size as usize * (y_stride + x_stride);
    let area = kernel_size * kernel_size;
    for y in 0..img.height() as usize {
        let y_off = y_stride * y;
        for x in 0..img.width() as usize {
            let off = y_off + x_stride * x;
            for c in 0..x_stride {
                dst.push(
                    ((target[off + rb_off + c] + target[off + c]
                        - target[off + lb_off + c]
                        - target[off + rt_off + c])
                        / area) as u8,
                );
            }
        }
    }
    dst
}

/// resize `img` to size (width, height).
pub fn resize<P, Container>(img: &ImageBuffer<P, Container>, width: u32, height: u32) -> Vec<u8>
where
    P: Pixel + 'static,
    P::Subpixel: 'static,
    Container: Deref<Target = [P::Subpixel]>,
{
    let (width, height) = (width as usize, height as usize);
    let x_stride = P::CHANNEL_COUNT as usize;
    let data = img.as_raw();
    let mut resized: Vec<u8> = Vec::with_capacity(width * height * x_stride);

    let x_scale = img.width() as f32 / width as f32;
    let y_scale = img.height() as f32 / height as f32;
    let y_stride = img.width() as usize * x_stride;

    for y in 0..height {
        for x in 0..width {
            let (fx, fy) = (x as f32 * x_scale, y as f32 * y_scale);
            let (dx, dy) = (fx.fract(), fy.fract());
            let (ix, iy) = (fx.floor() as usize, fy.floor() as usize);
            let off = iy * y_stride + ix * x_stride;
            for c in 0..x_stride {
                resized.push(
                    ((1.0f32 - dx) * (1.0f32 - dy) * data[off + c].to_f32().unwrap()
                        + dx * (1.0f32 - dy) * data[off + x_stride + c].to_f32().unwrap()
                        + (1.0f32 - dx) * dy * data[off + y_stride + c].to_f32().unwrap()
                        + dx * dy * data[off + y_stride + x_stride + c].to_f32().unwrap())
                        as u8,
                );
            }
        }
    }

    resized
}

#[cfg(test)]
mod tests {
    use nalgebra::matrix;

    use super::*;

    #[test]
    fn test_affine_transform() {
        let length = 10;
        let img = image::RgbImage::from_fn(length, length, |x, y| {
            image::Rgb([(x + y) as u8, x as u8, y as u8])
        });
        #[rustfmt::skip]
        let affine_mat = matrix![
            1.0, 0.0, 2.0;
            0.0, 1.0, 3.0;
        ];
        let res = affine_transform(&img, &affine_mat);

        assert_eq!(res[0], 0u8, "x = 0, y = 0");
        assert_eq!(res[1], 0u8, "x = 0, y = 0");
        assert_eq!(res[2], 0u8, "x = 0, y = 0");
        for y in 3..length - 3 {
            for x in 2..length - 2 {
                let offset = ((y * length + x) * 3) as usize;
                assert_eq!(res[offset + 0], (x + y - 5) as u8, "x = {}, y = {}", x, y);
                assert_eq!(res[offset + 1], (x - 2) as u8, "x = {}, y = {}", x, y);
                assert_eq!(res[offset + 2], (y - 3) as u8, "x = {}, y = {}", x, y);
            }
        }
        assert_eq!(res[res.len() - 3], (length + length - 7) as u8);
        assert_eq!(res[res.len() - 2], (length - 3) as u8);
        assert_eq!(res[res.len() - 1], (length - 4) as u8);
    }

    #[test]
    fn test_gaussian() {
        let length = 10;
        let kernel_size = 3;
        let sigma = 1.0f32;
        let img = image::RgbImage::from_fn(length, length, |_, _| image::Rgb([10u8, 5u8, 1u8]));
        let res = gaussian(&img, kernel_size, sigma);
        assert_eq!(res.len(), (length * length * 3) as usize);
        for i in 0..length * length {
            assert_eq!(res[(i * 3 + 0) as usize], 10, "i = {}", i);
            assert_eq!(res[(i * 3 + 1) as usize], 5, "i = {}", i);
            assert_eq!(res[(i * 3 + 2) as usize], 1, "i = {}", i);
        }
    }

    #[test]
    fn test_create_gauss_kernel() {
        let kernel = create_gauss_kernel(3, 1.0);
        assert_eq!(kernel.len(), 9);
        assert!((kernel.iter().sum::<f32>() - 1.0).abs() < 1e-5);
        assert!((kernel[0] - 0.07511360795411151).abs() < 1e-5);
        assert!((kernel[0] - kernel[2]).abs() < 1e-5);
        assert!((kernel[0] - kernel[6]).abs() < 1e-5);
        assert!((kernel[0] - kernel[8]).abs() < 1e-5);
        assert!((kernel[1] - 0.12384140315297397).abs() < 1e-5);
        assert!((kernel[1] - kernel[3]).abs() < 1e-5);
        assert!((kernel[1] - kernel[5]).abs() < 1e-5);
        assert!((kernel[1] - kernel[7]).abs() < 1e-5);
        assert!((kernel[4] - 0.2041799555716581).abs() < 1e-5);
    }

    #[test]
    fn test_padding() {
        let length = 10;
        let kernel = 5;
        let test_image = image::RgbImage::from_fn(length, length, |x, y| {
            image::Rgb([(x + y) as u8, x as u8, y as u8])
        });
        let padded = padding(&test_image, kernel / 2);
        let dst_size = length as usize + kernel / 2 * 2;
        assert_eq!(padded.len(), dst_size * dst_size * 3);
        assert_eq!(padded[0], 0);
        let y_stride = 3 * dst_size;
        assert_eq!(
            padded[3 * (kernel / 2 + 1) as usize - 1],
            0,
            "y = 0, x = {}, c = {}",
            kernel / 2 + 1,
            2
        );
        assert_eq!(
            padded[3 * (kernel / 2 + 1) as usize + 0],
            1,
            "y = 0, x = {}, c = {}",
            kernel / 2 + 2,
            0
        );
        assert_eq!(
            padded[3 * (kernel / 2 + 1) as usize + 1],
            1,
            "y = 0, x = {}, c = {}",
            kernel / 2 + 2,
            1
        );
        assert_eq!(
            padded[3 * (kernel / 2 + 1) as usize + 2],
            0,
            "y = 0, x = {}, c = {}",
            kernel / 2 + 2,
            2
        );
        assert_eq!(
            padded[3 * (length as usize + kernel / 2 + 1) as usize + 0],
            length as u8 - 1
        );
        assert_eq!(
            padded[3 * (length as usize + kernel / 2 + 1) as usize + 1],
            length as u8 - 1
        );
        assert_eq!(
            padded[3 * (length as usize + kernel / 2 + 1) as usize + 1],
            length as u8 - 1
        );
        assert_eq!(
            padded[3 * (length as usize + kernel / 2 + 1) as usize + 2],
            0
        );
        let rb = (y_stride + 3) * (length as usize + kernel / 2);
        assert_eq!(padded[rb + 0], ((length - 1) * 2) as u8);
        assert_eq!(padded[rb + 1], (length - 1) as u8);
        assert_eq!(padded[rb + 2], (length - 1) as u8);
        assert_eq!(padded[rb + 3], ((length - 1) * 2) as u8);
        assert_eq!(padded[rb + 4], (length - 1) as u8);
        assert_eq!(padded[rb + 5], (length - 1) as u8);
    }

    #[test]
    fn test_nms() {
        let kpts = vec![
            KeyPoint::new(3, 3, 10.0, 1),
            KeyPoint::new(3, 4, 12.5, 1),
            KeyPoint::new(3, 6, 11.8, 1),
            KeyPoint::new(5, 4, 11.5, 1),
            KeyPoint::new(3, 2, 8.0, 1),
        ];
        let supressed = nms(&kpts, 3);
        assert_eq!(supressed.len(), 4);
        assert!((supressed[0].crf() - 12.5).abs() < 1e-5);
        assert!((supressed[1].crf() - 11.8).abs() < 1e-5);
        assert!((supressed[2].crf() - 11.5).abs() < 1e-5);
        assert!((supressed[3].crf() - 8.0).abs() < 1e-5);
    }

    #[test]
    fn test_gray() {
        let length = 256;
        let test_image = image::RgbImage::from_fn(length, length, |x, y| {
            image::Rgb([((x + y) / 2) as u8, 0, 0])
        });
        let res = gray(&test_image);

        let data = test_image.as_raw();
        for y in 0..length {
            for x in 0..length {
                let off = ((y * length + x) * 3) as usize;
                assert_eq!(
                    res[(y * length + x) as usize],
                    (data[off] as f32 * 0.299) as u8
                );
            }
        }
    }

    #[test]
    fn test_median_filter() {
        let length: u32 = 10;
        let test_image =
            image::RgbImage::from_fn(length, length, |x, y| image::Rgb([x as u8, y as u8, 0]));
        let kernel: u32 = 5;
        let res = median_filter(&test_image, kernel);
        let x_stride: usize = 3;
        let y_stride: usize = length as usize * x_stride;

        assert_eq!(res[0], 0);
        assert_eq!(res[1], 0);
        assert_eq!(res[2], 0);
        assert_eq!(res[y_stride * 2 + x_stride * 2 as usize + 0], 2);
        assert_eq!(res[y_stride * 2 + x_stride * 2 as usize + 1], 2);
        assert_eq!(res[y_stride * 2 + x_stride * 2 as usize + 2], 0);
    }

    #[test]
    fn test_resize() {
        let length: u32 = 256;
        let half = length / 2;
        let test_image = image::RgbImage::from_fn(length, length, |x, y| {
            image::Rgb([((x + y) / 2) as u8, 0, 0])
        });
        let res = resize(&test_image, half, half);
        assert_eq!(res.len(), (half * half * 3) as usize);

        for y in 0..half {
            for x in 0..half {
                assert_eq!(
                    res[((y * half + x) * 3) as usize],
                    (x + y) as u8,
                    "(x, y) = {}, {}",
                    x,
                    y
                );
            }
        }
    }
}
