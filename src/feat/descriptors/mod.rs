use image::GrayImage;

use super::{keypoints::KeyPoint, Distance};

pub mod brief;

/// Feature Descriptor
#[derive(Clone)]
pub struct Descriptor<T>
where
    T: Distance + Clone,
{
    pub kpt: KeyPoint,
    pub value: T,
}

impl<T> Descriptor<T>
where
    T: Distance + Clone,
{
    pub fn distance(&self, rhs: &Self) -> f32 {
        self.value.distance(&rhs.value)
    }
}

/// Trait of descriptor extractor
pub trait Extractor<T>
where
    T: Distance + Clone,
{
    fn compute(&self, img: &GrayImage, kpts: &Vec<KeyPoint>) -> Vec<Descriptor<T>>;
}
