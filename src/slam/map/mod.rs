use std::ops::Deref;

use image::{ImageBuffer, Pixel};
use nalgebra::Matrix3;

use crate::feat::{descriptors::Descriptor, matcher::Match};

use super::{extract_orb, DescType};

pub mod covisibility_graph;
pub mod essential_graph;
pub mod keyframe;
pub mod map_point;

pub struct Map<P, Container>
where
    P: Pixel + 'static,
    P::Subpixel: 'static,
    Container: Deref<Target = [P::Subpixel]>,
{
    ref_frame: ImageBuffer<P, Container>, // reference frame
    ref_frame_descs: Vec<Descriptor<DescType>>,
}

impl<P, Container> Map<P, Container>
where
    P: Pixel + 'static,
    P::Subpixel: 'static,
    Container: Deref<Target = [P::Subpixel]>,
{
    pub fn new(image: ImageBuffer<P, Container>) -> Self {
        let mut map = Map {
            ref_frame: image,
            ref_frame_descs: Vec::new(),
        };
        map.ref_frame_descs = extract_orb(&map.ref_frame, 1, 1.0);
        map
    }

    pub fn initialize_map(mut self, cur_img: &ImageBuffer<P, Container>) -> Self {
        let descs = extract_orb(cur_img, 1, 1.0);
        let matches = self.calc_match(&descs);

        let (h, s_h) = self.find_homography(&matches);
        let (f, s_f) = self.find_fundamental_matrix(&matches);

        if s_h / (s_h + s_f) > 0.45 {
            self.motion_recovery8();
        } else {
            self.motion_recovery4();
        }
        self.run_bundle_adjustment();
        self
    }

    fn calc_match(&self, descs: &Vec<Descriptor<DescType>>) -> Vec<Match<DescType>> {
        Vec::new()
    }

    fn find_homography(&self, matches: &Vec<Match<DescType>>) -> (Matrix3<f32>, f32) {
        (nalgebra::one::<Matrix3<f32>>(), 0.0)
    }

    fn find_fundamental_matrix(&self, matches: &Vec<Match<DescType>>) -> (Matrix3<f32>, f32) {
        (nalgebra::one::<Matrix3<f32>>(), 0.0)
    }

    fn motion_recovery8(&self) {}

    fn motion_recovery4(&self) {}

    fn run_bundle_adjustment(&self) {}
}
