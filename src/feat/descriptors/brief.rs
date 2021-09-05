use bitvec::prelude::*;
use image::{GrayImage, Luma, Pixel};
use nalgebra::Point2;
use rand_distr::{Distribution, Normal};

use crate::{feat::keypoints::KeyPoint, imgproc::median_filter};

use super::{BriefBitVec, Descriptor, Extractor};

fn clip_point(patch_size: u32, pt: f32) -> f32 {
    let half = (patch_size / 2) as f32;
    pt.clamp(-half, half).round()
}

pub struct Brief {
    patch_size: u32,
    median_kernel_size: u32,
    pub binary_test_pairs: Vec<(Point2<f32>, Point2<f32>)>,
}

impl Brief {
    /// Args
    /// - patch_size : 特徴量を計算するpatch size。論文だと31
    /// - n_binary_test : number of binary test to be calculated。論文だと256
    pub fn new(patch_size: u32, median_kernel_size: u32, n_binary_test: u32) -> Self {
        let mut rng = rand::thread_rng();
        let normal = Normal::new(0.0, patch_size as f32 / 5.0).unwrap();
        let mut binary_test_pairs: Vec<(Point2<f32>, Point2<f32>)> =
            Vec::with_capacity(n_binary_test as usize);
        for _ in 0..n_binary_test {
            let x0 = clip_point(patch_size, normal.sample(&mut rng));
            let y0 = clip_point(patch_size, normal.sample(&mut rng));
            let mut x1 = clip_point(patch_size, normal.sample(&mut rng));
            let mut y1 = clip_point(patch_size, normal.sample(&mut rng));
            while x0 == x1 && y0 == y1 {
                x1 = clip_point(patch_size, normal.sample(&mut rng));
                y1 = clip_point(patch_size, normal.sample(&mut rng));
            }
            binary_test_pairs.push((Point2::new(x0, y0), Point2::new(x1, y1)));
        }
        Brief {
            patch_size,
            median_kernel_size,
            binary_test_pairs,
        }
    }

    pub fn calc_brief(
        &self,
        kpt: &KeyPoint,
        data: &Vec<u8>,
        stride_x: usize,
        stride_y: usize,
        test_pairs: &Vec<(Point2<f32>, Point2<f32>)>,
    ) -> Descriptor<BriefBitVec> {
        let (cx, cy) = (kpt.x() as usize, kpt.y() as usize);
        // let mut desc: BitVec = BitVec::with_capacity(self.binary_test_pairs.len());
        let mut desc: BriefBitVec = BriefBitVec::new(self.binary_test_pairs.len());
        for (p0, p1) in test_pairs {
            let (dx0, dy0) = (p0.x as usize, p0.y as usize);
            let (dx1, dy1) = (p1.x as usize, p1.y as usize);
            let idx0 = (cy + dy0) * stride_y + (cx + dx0) * stride_x;
            let idx1 = (cy + dy1) * stride_y + (cx + dx1) * stride_x;
            desc.push(data[idx0] < data[idx1])
        }
        Descriptor {
            kpt: kpt.clone(),
            value: desc,
        }
    }
}

impl Extractor<BriefBitVec> for Brief {
    fn compute(&self, img: &GrayImage, kpts: &Vec<KeyPoint>) -> Vec<Descriptor<BriefBitVec>> {
        // let gauss =
        //     image::GrayImage::from_raw(img.width(), img.height(), gaussian(img, 9, 3.05)).unwrap();
        let gauss = image::GrayImage::from_raw(
            img.width(),
            img.height(),
            median_filter(img, self.median_kernel_size),
        )
        .unwrap();
        let data = gauss.as_raw();
        let stride_x = Luma::<u8>::CHANNEL_COUNT as usize;
        let stride_y = gauss.width() as usize * stride_x;
        let mut descriptors: Vec<Descriptor<BriefBitVec>> = Vec::new();

        for kpt in kpts {
            let (cx, cy) = (kpt.x() as usize, kpt.y() as usize);
            if cx < (self.patch_size / 2) as usize
                || cy < (self.patch_size / 2) as usize
                || cx >= (gauss.width() - self.patch_size / 2) as usize
                || cy >= (gauss.height() - self.patch_size / 2) as usize
            {
                continue;
            }
            descriptors.push(self.calc_brief(
                &kpt,
                &data,
                stride_x,
                stride_y,
                &self.binary_test_pairs,
            ));
        }

        descriptors
    }
}

#[cfg(test)]
mod tests {
    use nalgebra::Point2;

    use crate::feat::{descriptors::Extractor, keypoints::KeyPoint};

    use super::Brief;

    #[test]
    fn test_brief_new() {
        let patch_size = 31;
        let n_pairs = 256;
        let brief = Brief::new(patch_size, 5, n_pairs);
        assert_eq!(brief.binary_test_pairs.len(), n_pairs as usize);
        assert_eq!(brief.patch_size, patch_size);
        assert_eq!(brief.median_kernel_size, 5);

        let min: i32 = -(patch_size as i32 / 2);
        let max: i32 = patch_size as i32 / 2;
        for (p0, p1) in brief.binary_test_pairs {
            assert!(min <= p0.x as i32);
            assert!(p0.x as i32 <= max);
            assert!(min <= p0.y as i32);
            assert!(p0.y as i32 <= max);
            assert!(min <= p1.x as i32);
            assert!(p1.x as i32 <= max);
            assert!(min <= p1.y as i32);
            assert!(p1.y as i32 <= max);
            assert!(p0.x as i32 != p1.x as i32 || p0.y as i32 != p1.y as i32);
        }
    }

    #[test]
    fn test_calc_brief() {
        let patch_size = 31;
        let n_pairs = 256;
        let kpt = KeyPoint::new(0, 0, 1.0, 0, 0.0);
        let data: Vec<u8> = vec![1, 2, 3, 0, 5, 6];
        let x_stride = 1;
        let y_stride = 3;
        let brief = Brief::new(patch_size, 5, n_pairs);
        let test_pairs: Vec<(Point2<f32>, Point2<f32>)> = vec![
            (
                Point2::<f32>::new(0.0f32, 0.0f32),
                Point2::<f32>::new(1.0f32, 0.0f32),
            ),
            (
                Point2::<f32>::new(0.0f32, 0.0f32),
                Point2::<f32>::new(0.0f32, 1.0f32),
            ),
        ];
        let desc = brief.calc_brief(&kpt, &data, x_stride, y_stride, &test_pairs);
        assert_eq!(desc.value[0] as usize, 1);
        assert_eq!(desc.value[1] as usize, 0);
    }

    #[test]
    fn test_compute() {
        let patch_size = 3;
        let n_pairs = 3;
        let mut brief = Brief::new(patch_size, 3, n_pairs);

        let length = 5;
        let img = image::GrayImage::from_fn(length, length, |x, y| image::Luma([(x + y) as u8]));
        let kpts = vec![
            KeyPoint::new(2, 2, 1.0, 0, 0.0),
            KeyPoint::new(0, 0, 1.0, 0, 0.0),
        ];

        brief.binary_test_pairs = vec![(
            Point2::<f32>::new(0.0f32, 0.0f32),
            Point2::<f32>::new(2.0f32, 2.0f32),
        )];
        let descs = brief.compute(&img, &kpts);
        assert_eq!(descs.len(), 1);
        assert!((descs[0].kpt.x() - 2.0).abs() < 1.0e-5);
        assert!((descs[0].kpt.y() - 2.0).abs() < 1.0e-5);
        assert_eq!(descs[0].value.len(), 1);
        assert_eq!(descs[0].value[0] as usize, 1);
    }
}
