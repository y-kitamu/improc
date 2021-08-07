//! Keypoint Detector
use image::GrayImage;
use nalgebra::geometry::Point2;

pub mod fast;
pub mod imgproc;

pub struct KeyPoint {
    loc: Point2<f32>,
    image_pyramid_level: u32,
}

impl KeyPoint {
    pub fn new(x: usize, y: usize, level: u32) -> Self {
        KeyPoint {
            loc: Point2::new(x as f32, y as f32),
            image_pyramid_level: level,
        }
    }
}

pub trait KeypointDetector {
    fn detect(&self, image: &GrayImage, level: u32) -> Vec<KeyPoint>;
}
