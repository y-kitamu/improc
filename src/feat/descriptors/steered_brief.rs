use bitvec::prelude::BitVec;
use image::{GrayImage, Luma, Pixel};
use nalgebra::Point2;

use crate::{
    feat::keypoints::KeyPoint,
    imgproc::{affine_transform, median_filter},
    linalg::{get_rotation_matrix, warp_point},
};

use super::{brief::Brief, Descriptor, Extractor};

pub struct SteeredBrief {
    brief: Brief,
    n_discrete: u32,
    border_offset: u32,
    rotated_binary_pairs: Vec<Vec<(Point2<f32>, Point2<f32>)>>,
}

impl SteeredBrief {
    /// Args
    /// - patch_size : 特徴量を計算するpatch size。論文だと31
    /// - n_binary_test : number of binary test to be calculated。論文だと256
    /// - n_discrete :
    pub fn new(patch_size: u32, n_binary_test: u32, n_discrete: u32) -> Self {
        let brief = Brief::new(patch_size, n_binary_test);
        let border_offset = (patch_size as f32 * 2.0f32.sqrt()) as u32 + 1;
        let mut rotated_binary_pairs: Vec<Vec<(Point2<f32>, Point2<f32>)>> =
            vec![Vec::with_capacity(n_binary_test as usize); n_discrete as usize];

        let center = patch_size / 2;
        let angle_pitch = 360.0f32 / n_discrete as f32;
        for i in 0..n_discrete {
            let rot =
                get_rotation_matrix(i as f32 * angle_pitch, (center as f32, center as f32), 1.0);
            for pair in &brief.binary_test_pairs {
                rotated_binary_pairs[i as usize]
                    .push((warp_point(&rot, &pair.0), warp_point(&rot, &pair.1)));
            }
        }

        SteeredBrief {
            brief,
            n_discrete,
            border_offset,
            rotated_binary_pairs,
        }
    }
}

impl Extractor<BitVec> for SteeredBrief {
    /// Compute SteeredBrief descriptor.
    /// keypointを中心に画像を回転した際に、patchが画像外にでるkeypointは無視される。
    /// (border_offsetより外側にあるkptsは無視される。)
    fn compute(&self, img: &GrayImage, kpts: &Vec<KeyPoint>) -> Vec<Descriptor<BitVec>> {
        let gauss =
            image::GrayImage::from_raw(img.width(), img.height(), median_filter(img, 5)).unwrap();
        let data = gauss.as_raw();
        let stride_x = Luma::<u8>::CHANNEL_COUNT as usize;
        let stride_y = gauss.width() as usize * stride_x;
        let mut descriptors: Vec<Descriptor<BitVec>> = Vec::new();
        let angle_pitch = 360 / self.n_discrete as usize;

        for kpt in kpts {
            if (kpt.x() as u32) < self.border_offset
                || (kpt.y() as u32) < self.border_offset
                || kpt.x() as u32 + self.border_offset >= img.width()
                || kpt.y() as u32 + self.border_offset >= img.height()
            {
                continue;
            }

            let rotate_idx = kpt.direction() as usize / angle_pitch;
            let desc = self.brief.calc_brief(
                &kpt,
                &data,
                stride_x,
                stride_y,
                &self.rotated_binary_pairs[rotate_idx],
            );
            descriptors.push(desc);
        }
        descriptors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_steered_brief_new() {}
}
