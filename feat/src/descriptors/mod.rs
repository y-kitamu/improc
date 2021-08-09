use image::GrayImage;

use crate::keypoints::KeyPoint;

pub mod brief;

/// Feature Descriptor
pub struct Descriptor<T> {
    pub kpt: KeyPoint,
    pub value: T,
}

/// Trait of descriptor extractor
pub trait Extractor<T> {
    fn compute(&self, img: &GrayImage, kpts: &Vec<KeyPoint>) -> Vec<T>;
}
