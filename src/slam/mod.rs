// use std::ops::Deref;

// use image::{ImageBuffer, Pixel};

// use crate::{
//     feat::{
//         descriptors::{steered_brief::SteeredBrief, BriefDescriptor, Descriptor, Extractor},
//         keypoints::{fast::FASTCornerDetector, KeypointDetector},
//     },
//     imgproc::gray,
// };

// pub mod local_mapping;
// pub mod loop_closing;
// pub mod map;
// pub mod tracking;

// type DescType = BriefDescriptor;

// fn extract_orb<P, Container>(
//     image: &ImageBuffer<P, Container>,
//     pyramid_level: u32,
//     pyramid_scale: f32,
// ) -> Vec<Descriptor<DescType>>
// where
//     P: Pixel + 'static,
//     P::Subpixel: 'static,
//     Container: Deref<Target = [P::Subpixel]>,
// {
//     let gray = image::GrayImage::from_raw(image.width(), image.height(), gray(image)).unwrap();

//     let fast = FASTCornerDetector::new(3, (50 * 50) as f32, pyramid_level, pyramid_scale, true);
//     let kpts = fast.detect(&gray, 0);

//     let sb = SteeredBrief::new(31, 5, 256, 12);
//     sb.compute(&gray, &kpts)
// }
