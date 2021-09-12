use std::ops::Deref;

use image::{ImageBuffer, Pixel};
use nalgebra::{matrix, Matrix3, Matrix3x4, Vector3};

use crate::feat::matcher::Match;

use super::{extract_orb, DescType};

pub struct Tracker {
    previous_pts: Vec<Vector3<f32>>,
    previous_pose: Matrix3x4<f32>,
    rotate_velocity: Matrix3<f32>,
    trans_velocity: Vector3<f32>,
    since_global_reloc: u32, // Number of frames passed from the last global relocalization
    since_last_kf_insertion: u32, // Number of frames passed from the last keyframe insertion
}

impl Tracker {
    pub fn new() -> Self {
        Tracker {
            previous_pts: Vec::new(),
            previous_pose: matrix![
                1.0, 0.0, 0.0, 0.0;
                0.0, 1.0, 0.0, 0.0;
                0.0, 0.0, 1.0, 0.0;
            ],
            rotate_velocity: nalgebra::one(),
            trans_velocity: nalgebra::zero(),
            since_global_reloc: 0,
            since_last_kf_insertion: 0,
        }
    }

    pub fn process_frame<P, Container>(&mut self, frame: &ImageBuffer<P, Container>)
    where
        P: Pixel + 'static,
        P::Subpixel: 'static,
        Container: Deref<Target = [P::Subpixel]>,
    {
        let descs = extract_orb(frame, 8, 1.2);
        let matches = self.guided_search(frame);
        self.track_local_map();
        if self.judge_use_as_keyframe() {}
    }

    fn guided_search<P, Container>(&self, frame: &ImageBuffer<P, Container>) -> Vec<Match<DescType>>
    where
        P: Pixel + 'static,
        P::Subpixel: 'static,
        Container: Deref<Target = [P::Subpixel]>,
    {
        Vec::new()
    }

    fn track_local_map(&self) {}

    fn judge_use_as_keyframe(&self) -> bool {
        true
    }
}
