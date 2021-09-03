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
}

pub trait KeypointDetector {
    fn detect(&self, image: &GrayImage, level: u32) -> Vec<KeyPoint>;
}
