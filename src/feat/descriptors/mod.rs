use image::GrayImage;
use std::ops::Index;

use super::{keypoints::KeyPoint, Distance};

pub mod brief;
pub mod steered_brief;

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

#[derive(Clone)]
pub struct BriefBitVec {
    pub bits: Vec<u64>,
    values: Vec<bool>,
    max_index: usize,
}

impl BriefBitVec {
    pub fn new(n_bits: usize) -> Self {
        let bits = vec![0; (n_bits + 63) / 64];
        let values = vec![false; n_bits];
        let max_index = 0;
        BriefBitVec {
            bits,
            values,
            max_index,
        }
    }

    pub fn push(&mut self, val: bool) {
        let idx = self.max_index / 64;
        let offset = self.max_index % 64;
        if val {
            self.bits[idx] |= 1 << offset;
        } else {
            self.bits[idx] &= !(1 << offset);
        }
        self.values[self.max_index] = val;
        self.max_index += 1;
    }

    pub fn len(&self) -> usize {
        self.max_index
    }
}

impl Distance for BriefBitVec {
    fn distance(&self, rhs: &Self) -> f32 {
        let dist = self
            .bits
            .iter()
            .zip(&rhs.bits)
            .fold(0, |acc, (l, r)| acc + (l ^ r).count_ones());
        dist as f32
    }
}

impl Index<usize> for BriefBitVec {
    type Output = bool;

    fn index(&self, index: usize) -> &bool {
        &self.values[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brief_bit_vec() {
        let n_bits = 256;
        let mut bvec = BriefBitVec::new(n_bits);
        for i in 0..n_bits {
            assert_eq!(bvec[i], false);
        }

        for i in 0..n_bits {
            if i % 3 == 0 {
                bvec.push(true);
                assert_eq!(bvec[i] as usize, 1);
            } else {
                bvec.push(false);
                assert_eq!(bvec[i] as usize, 0);
            }
        }
        assert_eq!(bvec.max_index, n_bits);
        assert_eq!(bvec[0] as usize, 1);
        assert_eq!(bvec[1] as usize, 0);
        assert_eq!(bvec[2] as usize, 0);
    }

    #[test]
    fn test_brief_bit_vec_distance() {
        let n_bits = 256;
        let mut lhs = BriefBitVec::new(n_bits);
        let mut rhs = BriefBitVec::new(n_bits);
        assert_eq!(lhs.distance(&rhs) as usize, 0);

        (0..n_bits).for_each(|i| lhs.push(i % 3 == 0));
        println!("lhs = {:?}", lhs.bits);
        assert_eq!(lhs[0] as usize, 1);
        assert_eq!(rhs.distance(&lhs) as usize, 86);

        (0..n_bits).for_each(|i| rhs.push(i % 2 == 0));
        assert_eq!(lhs.distance(&rhs) as usize, 128);
    }
}
