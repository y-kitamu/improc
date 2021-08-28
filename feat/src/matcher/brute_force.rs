use std::collections::HashMap;

use crate::{descriptors::Descriptor, Distance};

use super::{Match, Matcher};

struct BruteForceMathcer<T>
where
    T: Distance + Clone,
{
    descriptors: HashMap<String, Vec<Descriptor<T>>>,
    allow_duplicate: bool,
}

impl<T> BruteForceMathcer<T>
where
    T: Distance + Clone,
{
    fn new(
        lhs_key: &str,
        lhs_descs: Vec<Descriptor<T>>,
        rhs_key: &str,
        rhs_descs: Vec<Descriptor<T>>,
        allow_duplicate: bool,
    ) -> Self {
        let mut descriptors = HashMap::new();
        descriptors.insert(lhs_key.to_string(), lhs_descs);
        descriptors.insert(rhs_key.to_string(), rhs_descs);
        BruteForceMathcer {
            descriptors,
            allow_duplicate,
        }
    }
}

impl<T> Matcher<T> for BruteForceMathcer<T>
where
    T: Distance + Clone,
{
    fn run(&self, lhs_key: &str, rhs_key: &str) -> Vec<Match<T>> {
        let empty = Vec::new();
        let lhs_descs = self.descriptors.get(lhs_key).unwrap_or(&empty);
        let rhs_descs = self.descriptors.get(rhs_key).unwrap_or(&empty);

        // vector of tuple : (distance, lhs_idx, rhs_idx)
        let mut dists: Vec<(f32, usize, usize)> =
            Vec::with_capacity(lhs_descs.len() * rhs_descs.len());
        for li in 0..lhs_descs.len() {
            for ri in 0..rhs_descs.len() {
                let dist = lhs_descs[li].distance(&rhs_descs[ri]);
                dists.push((dist, li, ri));
            }
        }
        dists.sort_by(|l, r| l.0.partial_cmp(&r.0).unwrap());

        let mut matches = Vec::new();
        let mut lflag: Vec<bool> = vec![true; lhs_descs.len()];
        let mut rflag: Vec<bool> = vec![true; rhs_descs.len()];
        println!("dists.len() = {}", dists.len());
        for m in dists {
            println!("lhs_idx = {}, rhs_idx = {}", m.1, m.2);
            if lflag[m.1] && rflag[m.2] {
                matches.push(Match::new(
                    lhs_key,
                    &lhs_descs[m.1],
                    rhs_key,
                    &rhs_descs[m.2],
                ));
            }
            lflag[m.1] = false;
            rflag[m.2] = false;
        }
        matches
    }
}

#[cfg(test)]
mod tests {
    use bitvec::prelude::*;

    use crate::keypoints::KeyPoint;

    use super::*;

    fn prepare_descs() -> (Vec<Descriptor<BitVec>>, Vec<Descriptor<BitVec>>) {
        let n_dim = 5;
        let lhs_descs: Vec<Descriptor<BitVec>> = (0..=n_dim)
            .map(|i| Descriptor::<BitVec> {
                kpt: KeyPoint::new(i, i, 0.0f32, 0),
                value: (0..n_dim - i).fold(bitvec![0; i], |mut acc, _idx| {
                    acc.push(true);
                    acc
                }),
            })
            .collect();
        let rhs_descs = vec![
            Descriptor::<BitVec> {
                kpt: KeyPoint::new(0, 0, 0.0f32, 0),
                value: bitvec![0; n_dim],
            },
            Descriptor::<BitVec> {
                kpt: KeyPoint::new(1, 1, 0.0f32, 0),
                value: bitvec![1, 1, 0, 1, 1],
            },
            Descriptor::<BitVec> {
                kpt: KeyPoint::new(2, 2, 0.0f32, 0),
                value: bitvec![1, 0, 0, 1, 1],
            },
            Descriptor::<BitVec> {
                kpt: KeyPoint::new(3, 3, 0.0f32, 0),
                value: bitvec![1; n_dim],
            },
        ];
        (lhs_descs, rhs_descs)
    }

    #[test]
    fn test_brute_force_matcher() {
        let (lhs_descs, rhs_descs) = prepare_descs();
        assert_eq!(lhs_descs.len(), 6);
        assert_eq!(rhs_descs.len(), 4);
        assert_eq!(lhs_descs[0].value, bitvec![1, 1, 1, 1, 1]);
        let matcher = BruteForceMathcer::new("lhs", lhs_descs, "rhs", rhs_descs, false);
        let matches = matcher.run("lhs", "rhs");
        assert_eq!(matches.len(), 3);
        assert_eq!(matches[0].matches["lhs"].kpt.x() as usize, 0);
        assert_eq!(matches[0].matches["lhs"].kpt.y() as usize, 0);
        assert_eq!(matches[0].matches["rhs"].kpt.x() as usize, 3);
        assert_eq!(matches[0].matches["rhs"].kpt.y() as usize, 3);
        assert_eq!(matches[1].matches["lhs"].kpt.x() as usize, 5);
        assert_eq!(matches[1].matches["lhs"].kpt.y() as usize, 5);
        assert_eq!(matches[1].matches["rhs"].kpt.x() as usize, 0);
        assert_eq!(matches[1].matches["rhs"].kpt.y() as usize, 0);
        assert_eq!(matches[2].matches["lhs"].kpt.x() as usize, 3);
        assert_eq!(matches[2].matches["lhs"].kpt.y() as usize, 3);
        assert_eq!(matches[2].matches["rhs"].kpt.x() as usize, 2);
        assert_eq!(matches[2].matches["rhs"].kpt.y() as usize, 2);
    }
}
