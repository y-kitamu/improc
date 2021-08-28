use std::collections::HashMap;

use crate::{descriptors::Descriptor, Distance};

use super::{Match, Matcher};

struct BruteForceMathcer<T>
where
    T: Distance,
{
    descriptors: HashMap<String, Vec<Descriptor<T>>>,
    allow_duplicate: bool,
}

impl<T> BruteForceMathcer<T>
where
    T: Distance,
{
    fn new(
        lhs_key: &str,
        lhs_descs: Vec<Descriptor<T>>,
        rhs_key: &str,
        rhs_descs: Vec<Descriptor<T>>,
        allow_duplicate: bool,
    ) -> Self {
        let descriptors = HashMap::new();
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
    T: Distance,
{
    fn run(&self, lhs_key: &str, rhs_key: &str) -> Vec<Match<T>> {
        let lhs_descs = self.descriptors.get(lhs_key).unwrap_or(&Vec::new());
        let rhs_descs = self.descriptors.get(rhs_key).unwrap_or(&Vec::new());

        // vector of tuple : (distance, lhs_idx, rhs_idx)
        let mut dists: Vec<(f32, usize, usize)> =
            Vec::with_capacity(lhs_descs.len() * rhs_descs.len());
        for li in 0..lhs_descs.len() {
            for ri in 0..rhs_descs.len() {
                let dist = lhs_descs[li].distance(rhs_descs[ri]);
                dists.push((dist, li, ri));
            }
        }
        dists.sort_by(|l, r| l.0.partial_cmp(&r.0).unwrap());

        let mut matches = Vec::new();
        let mut lflag: Vec<bool> = vec![true; lhs_descs.len()];
        let mut rflag: Vec<bool> = vec![true; rhs_descs.len()];
        dists.iter().map(|m| {
            if lflag[m.1] && rflag[m.2] {
                matches.push(Match::new(
                    lhs_key,
                    &lhs_descs[m.1],
                    rhs_key,
                    &rhs_descs[m.2],
                ));
                if self.allow_duplicate {
                    lflag[m.1] = false;
                    rflag[m.2] = false;
                }
            }
            if !self.allow_duplicate {
                lflag[m.1] = false;
                rflag[m.2] = false;
            }
        });
        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brute_force_matcher() {}
}
