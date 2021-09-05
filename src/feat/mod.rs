use bitvec::prelude::*;

pub trait Distance {
    fn distance(&self, rhs: &Self) -> f32;
}

pub mod descriptors;
pub mod keypoints;
pub mod matcher;

impl Distance for BitVec {
    fn distance(&self, rhs: &Self) -> f32 {
        let dist = self
            .iter()
            .zip(rhs)
            .fold(0, |acc, (l, r)| acc + (l != r) as usize);
        dist as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitvec_distance() {
        let lhs = bitvec![0, 1, 1, 0, 0];
        let rhs = bitvec![0, 1, 0, 0, 1];
        let dist = lhs.distance(&rhs) as usize;
        assert_eq!(dist, 2);
        let rhs = bitvec![1, 0, 0, 1, 1,];
        let dist = lhs.distance(&rhs) as usize;
        assert_eq!(dist, 5);
    }
}
