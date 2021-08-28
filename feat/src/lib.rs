use std::ops::BitXor;

use bitvec::prelude::*;

pub trait Distance {
    fn distance(&self, rhs: &Self) -> f32;
}

pub mod descriptors;
pub mod keypoints;
pub mod matcher;

impl Distance for BitVec {
    fn distance(&self, rhs: &Self) -> f32 {
        let xor = self.clone().bitxor(rhs.clone());
        let dist = xor.iter().by_val().fold(0, |acc, cur| acc + cur as usize);
        dist as f32
    }
}
