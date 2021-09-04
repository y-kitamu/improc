//! Keypoint Detector
use image::GrayImage;
use nalgebra::geometry::Point2;

pub mod fast;

#[derive(Clone, Copy, Debug)]
pub struct KeyPoint {
    loc: Point2<f32>,
    cornerness: f32,
    image_pyramid_level: u32,
    direction: f32,
}

impl KeyPoint {
    pub fn new(x: usize, y: usize, cornerness: f32, level: u32, direction: f32) -> Self {
        KeyPoint {
            loc: Point2::new(x as f32, y as f32),
            cornerness,
            image_pyramid_level: level,
            direction,
        }
    }

    pub fn x(&self) -> f32 {
        self.loc.x
    }

    pub fn y(&self) -> f32 {
        self.loc.y
    }

    pub fn crf(&self) -> f32 {
        self.cornerness
    }

    pub fn direction(&self) -> f32 {
        self.direction
    }

    pub fn cgpt3d(&self) -> cgmath::Point3<f32> {
        cgmath::Point3::<f32>::new(self.loc.x, self.loc.y, 1.0)
    }
}

pub trait KeypointDetector {
    fn detect(&self, image: &GrayImage, level: u32) -> Vec<KeyPoint>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypoint() {
        let kpt = KeyPoint::new(10, 20, 1.0, 1, 1.0);
        assert!((kpt.x() - 10.0).abs() < 1e-5);
        assert!((kpt.y() - 20.0).abs() < 1e-5);
        assert!((kpt.crf() - 1.0).abs() < 1e-5);
        assert!((kpt.direction() - 1.0).abs() < 1e-5);
        let pt = kpt.cgpt3d();
        assert!((pt.x - 10.0).abs() < 1e-5);
        assert!((pt.y - 20.0).abs() < 1e-5);
        assert!((pt.z - 1.0).abs() < 1e-5);
    }
}
