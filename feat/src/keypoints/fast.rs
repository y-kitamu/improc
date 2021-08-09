//! Implementation of FAST corner detector.

use std::cmp::min;

use image::{DynamicImage, GrayImage, RgbImage};
use nalgebra::Point2;

use super::{KeyPoint, KeypointDetector};

/// 指定した半径`radius`の円周上の点を取得する
fn create_circle(radius: u32) -> Vec<Point2<f32>> {
    let mut points: Vec<Point2<f32>> = Vec::new();
    let sq_rad = (radius * radius) as f32;
    points.insert(0, Point2::new(radius as f32, 0.0f32));

    loop {
        let prev_pt = points.get(points.len() - 1).unwrap();
        let diff1 = ((prev_pt.x - 1.0f32).powi(2) + prev_pt.y.powi(2) - sq_rad).abs();
        let diff2 = ((prev_pt.x - 1.0f32).powi(2) + (prev_pt.y + 1.0f32).powi(2) - sq_rad).abs();
        let diff3 = (prev_pt.x.powi(2) + (prev_pt.y + 1.0f32).powi(2) - sq_rad).abs();

        let next_pt = if (diff2 <= diff1) && (diff2 <= diff3) {
            Point2::new(prev_pt.x - 1.0f32, prev_pt.y + 1.0f32)
        } else if (diff1 <= diff2) && (diff1 <= diff3) {
            Point2::new(prev_pt.x - 1.0f32, prev_pt.y)
        } else {
            Point2::new(prev_pt.x, prev_pt.y + 1.0f32)
        };
        if ((next_pt.x - 0.0f32).abs() < 1e-7) && ((next_pt.y - radius as f32) < 1e-7) {
            break;
        }
        points.insert(points.len(), next_pt);
    }

    let n_quarter = points.len();
    for _ in 0..3 {
        for _ in 0..n_quarter {
            let base = points.get(points.len() - n_quarter).unwrap();
            let pt = Point2::new(-base.y, base.x);
            points.insert(points.len(), pt);
        }
    }
    points
}

/// Corner response function. Calcurate cornerness.
fn calc_crf(cval: f32, val0: f32, val1: f32) -> f32 {
    (val0 - cval).powi(2) + (val1 - cval).powi(2)
}

pub struct FASTCornerDetector {
    radius: u32,
    threshold: f32,
    n_pyramid: u32,
    circle_points: Vec<Point2<f32>>,
}

impl FASTCornerDetector {
    pub fn new(radius: u32, threshold: f32, n_pyramid: u32) -> Self {
        FASTCornerDetector {
            radius,
            threshold,
            n_pyramid,
            circle_points: create_circle(radius),
        }
    }
}

impl KeypointDetector for FASTCornerDetector {
    fn detect(&self, image: &GrayImage, level: u32) -> Vec<KeyPoint> {
        let mut key_points = Vec::<KeyPoint>::new();
        let raw = image.as_raw();

        if level + 1 < self.n_pyramid {
            let resized_w = image.width() / 2;
            let resized_h = image.height() / 2;
            let resized_raw = super::imgproc::resize(&image, resized_w, resized_h);
            let resized_image =
                image::GrayImage::from_raw(resized_w, resized_h, resized_raw).unwrap();
            let mut kpts = self.detect(&resized_image, level + 1);
            key_points.append(&mut kpts);
        }

        let w = image.width() as usize;
        let h = image.height() as usize;
        let radius = self.radius as usize;
        let pt_offset = self.circle_points.len() / 2;
        for y in radius..h - radius {
            for x in radius..w - radius {
                let c = raw[(y * w + x) as usize] as f32;
                // rough test
                let l = raw[y * w + x + radius] as f32;
                let r = raw[y * w + x - radius] as f32;
                let crf_lr = calc_crf(c, l, r);
                let t = raw[(y - radius) * w + x] as f32;
                let b = raw[(y + radius) * w + x] as f32;
                let crf_tb = calc_crf(c, t, b);
                if crf_lr.min(crf_tb) < self.threshold {
                    continue;
                }

                let mut crf = crf_lr;
                // full test
                for i in 1..pt_offset {
                    let p0 = self.circle_points[i];
                    let p1 = self.circle_points[i + pt_offset];
                    crf = crf.min(calc_crf(
                        c,
                        raw[(y - p0.y as usize) * w + x + p0.x as usize] as f32,
                        raw[(y - p1.y as usize) * w + x + p1.x as usize] as f32,
                    ));
                    if crf < self.threshold {
                        break;
                    }
                }
                if crf > self.threshold {
                    key_points.push(KeyPoint::new(x, y, level));
                }
            }
        }
        key_points
    }
}

#[cfg(test)]
mod tests {
    use super::{calc_crf, FASTCornerDetector};
    use crate::keypoints::KeypointDetector;

    #[test]
    fn fast_detect() {
        let fast = FASTCornerDetector::new(3, 10.0f32, 1);
        let img = image::ImageBuffer::from_fn(32, 32, |x, y| {
            if (x < 16) && (y >= 16) {
                image::Luma([255u8])
            } else {
                image::Luma([0u8])
            }
        });
        let key_points = fast.detect(&img, 0);
        assert_eq!(key_points.len(), 8);
        assert_eq!(key_points[0].loc.x as usize, 13);
        assert_eq!(key_points[0].loc.y as usize, 16);
        assert_eq!(key_points[1].loc.x as usize, 14);
        assert_eq!(key_points[1].loc.y as usize, 16);
        assert_eq!(key_points[2].loc.x as usize, 15);
        assert_eq!(key_points[2].loc.y as usize, 16);
        assert_eq!(key_points[3].loc.x as usize, 13);
        assert_eq!(key_points[3].loc.y as usize, 17);
        assert_eq!(key_points[4].loc.x as usize, 14);
        assert_eq!(key_points[4].loc.y as usize, 17);
        assert_eq!(key_points[5].loc.x as usize, 15);
        assert_eq!(key_points[5].loc.y as usize, 17);
        assert_eq!(key_points[6].loc.x as usize, 14);
        assert_eq!(key_points[6].loc.y as usize, 18);
        assert_eq!(key_points[7].loc.x as usize, 15);
        assert_eq!(key_points[7].loc.y as usize, 18);
    }

    #[test]
    fn test_clac_crf() {
        assert_eq!(calc_crf(0.0, 1.0, -1.0), 2.0);
        assert_eq!(calc_crf(1.0, 1.0, -1.0), 4.0);
        assert_eq!(calc_crf(1.0, 2.0, -1.0), 5.0);
    }

    #[test]
    fn fast3() {
        let fast3 = FASTCornerDetector::new(3, 10.0f32, 1);
        assert_eq!(fast3.circle_points.len(), 16);
        assert!((fast3.circle_points[0].x - 3.0f32).abs() < 1e-5);
        assert!((fast3.circle_points[0].y - 0.0f32).abs() < 1e-5);

        assert!((fast3.circle_points[1].x - 3.0f32).abs() < 1e-5);
        assert!((fast3.circle_points[1].y - 1.0f32).abs() < 1e-5);

        assert!((fast3.circle_points[2].x - 2.0f32).abs() < 1e-5);
        assert!((fast3.circle_points[2].y - 2.0f32).abs() < 1e-5);

        assert!((fast3.circle_points[3].x - 1.0f32).abs() < 1e-5);
        assert!((fast3.circle_points[3].y - 3.0f32).abs() < 1e-5);

        assert!((fast3.circle_points[4].x - 0.0f32).abs() < 1e-5);
        assert!((fast3.circle_points[4].y - 3.0f32).abs() < 1e-5);

        assert!((fast3.circle_points[5].x + 1.0f32).abs() < 1e-5);
        assert!((fast3.circle_points[5].y - 3.0f32).abs() < 1e-5);

        assert!((fast3.circle_points[10].x + 2.0f32).abs() < 1e-5);
        assert!((fast3.circle_points[10].y + 2.0f32).abs() < 1e-5);

        assert!((fast3.circle_points[15].x - 3.0f32).abs() < 1e-5);
        assert!((fast3.circle_points[15].y + 1.0f32).abs() < 1e-5);
    }

    #[test]
    fn fast5() {
        let fast5 = FASTCornerDetector::new(5, 10.0f32, 1);
        assert_eq!(fast5.circle_points.len(), 28);

        assert!((fast5.circle_points[0].x - 5.0f32).abs() < 1e-5);
        assert!((fast5.circle_points[0].y - 0.0f32).abs() < 1e-5);

        assert!((fast5.circle_points[1].x - 5.0f32).abs() < 1e-5);
        assert!((fast5.circle_points[1].y - 1.0f32).abs() < 1e-5);

        assert!((fast5.circle_points[2].x - 5.0f32).abs() < 1e-5);
        assert!((fast5.circle_points[2].y - 2.0f32).abs() < 1e-5);

        assert!((fast5.circle_points[3].x - 4.0f32).abs() < 1e-5);
        assert!((fast5.circle_points[3].y - 3.0f32).abs() < 1e-5);

        assert!((fast5.circle_points[4].x - 3.0f32).abs() < 1e-5);
        assert!((fast5.circle_points[4].y - 4.0f32).abs() < 1e-5);

        assert!((fast5.circle_points[5].x - 2.0f32).abs() < 1e-5);
        assert!((fast5.circle_points[5].y - 5.0f32).abs() < 1e-5);

        assert!((fast5.circle_points[6].x - 1.0f32).abs() < 1e-5);
        assert!((fast5.circle_points[6].y - 5.0f32).abs() < 1e-5);

        assert!((fast5.circle_points[7].x - 0.0f32).abs() < 1e-5);
        assert!((fast5.circle_points[7].y - 5.0f32).abs() < 1e-5);

        assert!((fast5.circle_points[8].x + 1.0f32).abs() < 1e-5);
        assert!((fast5.circle_points[8].y - 5.0f32).abs() < 1e-5);
    }
}
