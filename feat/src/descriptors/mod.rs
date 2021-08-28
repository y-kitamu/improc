use image::GrayImage;

use crate::{keypoints::KeyPoint, Distance};

pub mod brief;

/// Feature Descriptor
pub struct Descriptor<T>
where
    T: Distance,
{
    pub kpt: KeyPoint,
    pub value: T,
}

/// Trait of descriptor extractor
pub trait Extractor<T>
where
    T: Distance,
{
    fn compute(&self, img: &GrayImage, kpts: &Vec<KeyPoint>) -> Vec<Descriptor<T>>;
}
