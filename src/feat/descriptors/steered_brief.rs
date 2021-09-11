use image::{GrayImage, Luma, Pixel};
use nalgebra::Point2;

use crate::{
    feat::keypoints::KeyPoint,
    imgproc::median_filter,
    linalg::{get_rotation_matrix, warp_point},
};

use super::{brief::Brief, BriefBitVec, Descriptor, Extractor};

pub struct SteeredBrief {
    brief: Brief,
    n_discrete: u32,
    border_offset: u32,
    pub rotated_binary_pairs: Vec<Vec<(Point2<f32>, Point2<f32>)>>,
}

impl SteeredBrief {
    /// Args
    /// - patch_size : 特徴量を計算するpatch size。論文だと31
    /// - median_kernel_size : 前処理のmedian filterのkernel size。論文だと5
    /// - n_binary_test : number of binary test to be calculated。論文だと256
    /// - n_discrete : 回転角の離散化の数。360 / `n_discrete`度ごとに離散化する。論文だと12
    pub fn new(
        patch_size: u32,
        median_kernel_size: u32,
        n_binary_test: u32,
        n_discrete: u32,
    ) -> Self {
        let brief = Brief::new(patch_size, median_kernel_size, n_binary_test);
        let border_offset = (patch_size as f32 / 2.0f32.sqrt()) as u32 + 1;
        let mut rotated_binary_pairs: Vec<Vec<(Point2<f32>, Point2<f32>)>> =
            vec![Vec::with_capacity(n_binary_test as usize); n_discrete as usize];

        let angle_pitch = std::f32::consts::PI * 2.0f32 / n_discrete as f32;
        for i in 0..n_discrete {
            let rot = get_rotation_matrix(i as f32 * angle_pitch, (0.0f32, 0.0f32), 1.0);
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

impl Extractor<BriefBitVec> for SteeredBrief {
    /// Compute SteeredBrief descriptor.
    /// keypointを中心に画像を回転した際に、patchが画像外にでるkeypointは無視される。
    /// (border_offsetより外側にあるkptsは無視される。)
    fn compute(&self, img: &GrayImage, kpts: &Vec<KeyPoint>) -> Vec<Descriptor<BriefBitVec>> {
        let gauss =
            image::GrayImage::from_raw(img.width(), img.height(), median_filter(img, 5)).unwrap();
        let data = gauss.as_raw();
        let stride_x = Luma::<u8>::CHANNEL_COUNT as usize;
        let stride_y = gauss.width() as usize * stride_x;
        let mut descriptors: Vec<Descriptor<BriefBitVec>> = Vec::new();
        let angle_pitch = 2.0 * std::f32::consts::PI / self.n_discrete as f32;

        for kpt in kpts {
            if (kpt.x() as u32) < self.border_offset
                || (kpt.y() as u32) < self.border_offset
                || kpt.x() as u32 + self.border_offset >= img.width()
                || kpt.y() as u32 + self.border_offset >= img.height()
            {
                continue;
            }

            let mut radian = kpt.direction();
            if radian < 0.0 {
                radian = std::f32::consts::PI * 2.0 + radian;
            }
            let mut rotate_idx = (radian / angle_pitch).round() as usize;
            if rotate_idx >= self.n_discrete as usize {
                rotate_idx -= self.n_discrete as usize;
            }
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
    use crate::imgproc::affine_transform;

    use super::*;

    #[test]
    fn test_steered_brief_new() {
        let patch_size = 31;
        let median_kernel_size = 5;
        let n_binary_test = 10;
        let n_discrete = 20;
        let sbrief = SteeredBrief::new(patch_size, median_kernel_size, n_binary_test, n_discrete);

        assert_eq!(sbrief.n_discrete, n_discrete);
        assert_eq!(sbrief.border_offset, 22);
        assert_eq!(sbrief.rotated_binary_pairs.len(), n_discrete as usize);
        assert_eq!(sbrief.rotated_binary_pairs[0].len(), n_binary_test as usize);

        let pair0 = sbrief.rotated_binary_pairs[0][0];
        let pair180 = sbrief.rotated_binary_pairs[n_discrete as usize / 2][0];
        assert!(
            (pair0.0.x + pair180.0.x).abs() < 1e-5,
            "pair0.0.x = {}, pair180.0.x = {}",
            pair0.0.x,
            pair180.0.x
        );
        assert!(
            (pair0.0.y + pair180.0.y).abs() < 1e-5,
            "pair0.0.y = {}, pair180.0.y = {}",
            pair0.0.y,
            pair180.0.y
        );
        assert!(
            (pair0.1.x + pair180.1.x).abs() < 1e-5,
            "pair0.1.x = {}, pair180.1.x = {}",
            pair0.1.x,
            pair180.1.x
        );
        assert!(
            (pair0.1.y + pair180.1.y).abs() < 1e-5,
            "pair0.1.y = {}, pair180.1.y = {}",
            pair0.1.y,
            pair180.1.y
        );
    }

    #[test]
    fn test_steered_brief_compute() {
        let patch_size = 3;
        let median_kernel_size = 5;
        let n_binary_test = 2;
        let n_discrete = 4;
        let mut sbrief =
            SteeredBrief::new(patch_size, median_kernel_size, n_binary_test, n_discrete);

        sbrief.rotated_binary_pairs = vec![
            vec![
                (Point2::<f32>::new(1.0, 0.0), Point2::<f32>::new(0.0, 1.0)),
                (Point2::<f32>::new(-1.0, 0.0), Point2::<f32>::new(0.0, -1.0)),
            ],
            vec![
                (Point2::<f32>::new(0.0, 1.0), Point2::<f32>::new(-1.0, 0.0)),
                (Point2::<f32>::new(0.0, -1.0), Point2::<f32>::new(1.0, 0.0)),
            ],
            vec![
                (Point2::<f32>::new(-1.0, 0.0), Point2::<f32>::new(0.0, -1.0)),
                (Point2::<f32>::new(1.0, 0.0), Point2::<f32>::new(0.0, 1.0)),
            ],
            vec![
                (Point2::<f32>::new(0.0, -1.0), Point2::<f32>::new(1.0, 0.0)),
                (Point2::<f32>::new(0.0, 1.0), Point2::<f32>::new(-1.0, 0.0)),
            ],
        ];

        let length = 11;
        let img = image::GrayImage::from_fn(length, length, |x, y| image::Luma([(x + y) as u8]));
        let rot_mat = get_rotation_matrix(
            std::f32::consts::FRAC_PI_2,
            (length as f32 / 2.0, length as f32 / 2.0),
            1.0f32,
        );
        let rot_img =
            image::GrayImage::from_raw(length, length, affine_transform(&img, &rot_mat)).unwrap();

        print_image(length as usize, length as usize, img.as_raw());
        print_image(length as usize, length as usize, rot_img.as_raw());
        // assert_eq!(rot_img.get_pixel(5, 5).0, img.get_pixel(5, 5).0);

        let kpts = vec![
            KeyPoint::new(5, 5, 1.0, 0, 0.0),
            KeyPoint::new(5, 7, 1.0, 0, 0.0),
        ];
        let rot_kpts = vec![
            KeyPoint::new(5, 5, 1.0, 0, std::f32::consts::FRAC_PI_2),
            KeyPoint::new(3, 5, 1.0, 0, std::f32::consts::FRAC_PI_2),
        ];
        let descs = sbrief.compute(&img, &kpts);
        let rot_descs = sbrief.compute(&rot_img, &rot_kpts);

        assert_eq!(descs.len(), rot_descs.len());
        assert_eq!(descs.len(), 2);

        for (idx, (dd, rd)) in descs.iter().zip(rot_descs).enumerate() {
            assert_eq!(
                dd.value[0], rd.value[0],
                "idx = {}, dd.value[0] = {}, rd.value[0] = {}",
                idx, dd.value[0], rd.value[0]
            );
            assert_eq!(
                dd.value[1], rd.value[1],
                "idx = {}, dd.value[1] = {}, rd.value[1] = {}",
                idx, dd.value[1], rd.value[1]
            );
        }
    }

    fn print_image(width: usize, height: usize, data: &Vec<u8>) {
        (0..height).for_each(|h| {
            println!(
                "{:?}",
                (0..width)
                    .map(|w| { data[h * width + w] })
                    .collect::<Vec<u8>>()
            );
        });
    }
}
