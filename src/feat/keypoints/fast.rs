//! Implementation of FAST corner detector.
use image::GrayImage;
use nalgebra::Point2;

use crate::imgproc::nms;

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
    use_nms: bool,
}

impl FASTCornerDetector {
    pub fn new(radius: u32, threshold: f32, n_pyramid: u32, use_nms: bool) -> Self {
        FASTCornerDetector {
            radius,
            threshold,
            n_pyramid,
            circle_points: create_circle(radius),
            use_nms,
        }
    }

    /// calc the keypoint's direction in radians.
    fn calc_direction(&self, raw: &Vec<u8>, w: usize, cx: usize, cy: usize) -> f32 {
        let mut m10 = 0;
        let mut m01 = 0;
        let (min_x, max_x) = (cx - self.radius as usize, cx + self.radius as usize);
        let (min_y, max_y) = (cy - self.radius as usize, cy + self.radius as usize);
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                m10 += (x as isize - cx as isize) * raw[w * y + x] as isize;
                m01 += (y as isize - cy as isize) * raw[w * y + x] as isize;
            }
        }
        (m01 as f32).atan2(m10 as f32)
    }
}

impl KeypointDetector for FASTCornerDetector {
    fn detect(&self, image: &GrayImage, level: u32) -> Vec<KeyPoint> {
        let mut key_points = Vec::<KeyPoint>::new();
        let raw = image.as_raw();

        if level + 1 < self.n_pyramid {
            let resized_w = image.width() / 2;
            let resized_h = image.height() / 2;
            let resized_raw = crate::imgproc::resize(&image, resized_w, resized_h);
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
                        raw[(y as f32 + p0.y) as usize * w + (x as f32 + p0.x) as usize] as f32,
                        raw[(y as f32 + p1.y) as usize * w + (x as f32 + p1.x) as usize] as f32,
                    ));
                    if crf < self.threshold {
                        break;
                    }
                }
                if crf > self.threshold {
                    let direction = self.calc_direction(&raw, w, x, y);
                    key_points.push(KeyPoint::new(x, y, crf, level, direction));
                }
            }
        }
        if self.use_nms {
            let key_points = nms(&key_points, self.radius * 2 + 1);
            return key_points;
        }
        key_points.sort_by(|lhs, rhs| lhs.crf().partial_cmp(&rhs.crf()).unwrap());
        key_points
    }
}

#[cfg(test)]
mod tests {
    use super::{calc_crf, FASTCornerDetector};
    use crate::feat::keypoints::KeypointDetector;

    #[test]
    fn fast_detect() {
        let fast = FASTCornerDetector::new(3, 10.0f32, 1, false);
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
    fn fast_detect2() {
        let fast = FASTCornerDetector::new(3, 10.0f32, 1, false);
        let img = image::ImageBuffer::from_fn(32, 32, |x, y| {
            if (x >= 16) && (y >= 16) {
                image::Luma([255u8])
            } else {
                image::Luma([0u8])
            }
        });

        let key_points = fast.detect(&img, 0);
        assert_eq!(key_points.len(), 8, "{:?}", key_points);
        assert_eq!(key_points[0].loc.x as usize, 16);
        assert_eq!(key_points[0].loc.y as usize, 16);
        assert_eq!(key_points[1].loc.x as usize, 17);
        assert_eq!(key_points[1].loc.y as usize, 16);
        assert_eq!(key_points[2].loc.x as usize, 18);
        assert_eq!(key_points[2].loc.y as usize, 16);
        assert_eq!(key_points[3].loc.x as usize, 16);
        assert_eq!(key_points[3].loc.y as usize, 17);
        assert_eq!(key_points[4].loc.x as usize, 17);
        assert_eq!(key_points[4].loc.y as usize, 17);
        assert_eq!(key_points[5].loc.x as usize, 18);
        assert_eq!(key_points[5].loc.y as usize, 17);
        assert_eq!(key_points[6].loc.x as usize, 16);
        assert_eq!(key_points[6].loc.y as usize, 18);
        assert_eq!(key_points[7].loc.x as usize, 17);
        assert_eq!(key_points[7].loc.y as usize, 18);
    }

    #[test]
    fn fast_detect3() {
        let fast = FASTCornerDetector::new(3, 10.0f32, 1, false);
        let img = image::ImageBuffer::from_fn(32, 32, |x, y| {
            if (x < 16) && (y < 16) {
                image::Luma([255u8])
            } else {
                image::Luma([0u8])
            }
        });
        let key_points = fast.detect(&img, 0);
        assert_eq!(key_points.len(), 8, "{:?}", key_points);
    }

    #[test]
    fn fast_detect4() {
        let fast = FASTCornerDetector::new(3, 10.0f32, 1, false);
        let img = image::ImageBuffer::from_fn(32, 32, |x, y| {
            if (x >= 16) && (y < 16) {
                image::Luma([255u8])
            } else {
                image::Luma([0u8])
            }
        });
        let key_points = fast.detect(&img, 0);
        assert_eq!(key_points.len(), 8, "{:?}", key_points);
    }

    #[test]
    fn test_clac_crf() {
        assert_eq!(calc_crf(0.0, 1.0, -1.0), 2.0);
        assert_eq!(calc_crf(1.0, 1.0, -1.0), 4.0);
        assert_eq!(calc_crf(1.0, 2.0, -1.0), 5.0);
    }

    #[test]
    fn fast3() {
        let fast3 = FASTCornerDetector::new(3, 10.0f32, 1, false);
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
        let fast5 = FASTCornerDetector::new(5, 10.0f32, 1, false);
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

        for i in 0..14 {
            let p0 = fast5.circle_points[i];
            let p1 = fast5.circle_points[14 + i];
            assert!((p0.x + p1.x).abs() < 1e-5);
            assert!((p0.y + p1.y).abs() < 1e-5);
        }
    }

    #[test]
    fn fast9() {
        let fast9 = FASTCornerDetector::new(9, 10.0f32, 1, false);
        let n_pts = fast9.circle_points.len();
        let n_half = n_pts / 2;

        for i in 0..n_half {
            let p0 = fast9.circle_points[i];
            let p1 = fast9.circle_points[n_half + i];
            assert!((p0.x + p1.x).abs() < 1e-5);
            assert!((p0.y + p1.y).abs() < 1e-5);
        }
    }

    #[test]
    fn test_calc_direction() {
        let fast = FASTCornerDetector::new(1, 0.0, 1, false);
        #[rustfmt::skip]
        let vec: Vec<u8> = vec![
            0, 0, 0,
            0, 0, 0,
            0, 0, 1,
        ];
        let dir = fast.calc_direction(&vec, 3, 1, 1);
        assert!(
            (dir - std::f32::consts::FRAC_PI_4).abs() < 1e-5,
            "direction = {}",
            dir
        );
    }
}
