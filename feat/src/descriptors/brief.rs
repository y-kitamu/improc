use bitvec::prelude::*;
use image::{GrayImage, Luma, Pixel};
use nalgebra::Point2;
use rand::Rng;
use rand_distr::{Distribution, Normal};

use crate::keypoints::{imgproc::gaussian, KeyPoint};

use super::{Descriptor, Extractor};

fn clip_point(patch_size: u32, pt: f32) -> f32 {
    let half = (patch_size / 2) as f32;
    pt.clamp(-half, half).round()
}

pub struct Brief {
    pub binary_test_pairs: Vec<(Point2<f32>, Point2<f32>)>,
}

impl Brief {
    pub fn new(patch_size: u32, n_binary_test: u32) -> Self {
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
        Brief { binary_test_pairs }
    }
}

impl Extractor<Descriptor<BitVec>> for Brief {
    fn compute(&self, img: &GrayImage, kpts: &Vec<KeyPoint>) -> Vec<Descriptor<BitVec>> {
        let mut desc: BitVec = BitVec::with_capacity(self.binary_test_pairs.len());
        let gauss =
            image::GrayImage::from_raw(img.width(), img.height(), gaussian(img, 9, 3.05)).unwrap();
        let data = gauss.as_raw();
        let stride_x = Luma::<u8>::CHANNEL_COUNT as usize;
        let stride_y = gauss.width() as usize * stride_x;

        for kpt in kpts {
            for (p0, p1) in &self.binary_test_pairs {
                let (cx, cy) = (kpt.x() as usize, kpt.y() as usize);
                let (dx0, dy0) = (p0.x as usize, p0.y as usize);
                let (dx1, dy1) = (p1.x as usize, p1.y as usize);
                let idx0 = (cy + dy0) * stride_y + (cx + dx0) * stride_x;
                let idx1 = (cy + dy1) * stride_y + (cx + dx1) * stride_y;
                desc.push(data[idx0] < data[idx1])
            }
        }

        let desc = Descriptor {
            kpt: KeyPoint::new(0, 0, 0),
            value: BitVec::new(),
        };
        let descriptors = vec![desc];
        descriptors
    }
}

#[cfg(test)]
mod tests {
    use super::Brief;

    #[test]
    fn test_brief_new() {
        let patch_size = 31;
        let n_pairs = 256;
        let brief = Brief::new(patch_size, n_pairs);
        assert_eq!(brief.binary_test_pairs.len(), n_pairs as usize);

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
}
