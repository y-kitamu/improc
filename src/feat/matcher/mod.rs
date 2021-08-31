use std::collections::HashMap;

use super::{descriptors::Descriptor, Distance};

struct Match<T>
where
    T: Distance + Clone,
{
    matches: HashMap<String, Descriptor<T>>,
}

impl<T> Match<T>
where
    T: Distance + Clone,
{
    fn new(
        lhs_key: &str,
        lhs_desc: &Descriptor<T>,
        rhs_key: &str,
        rhs_desc: &Descriptor<T>,
    ) -> Self {
        let mut matches = HashMap::<String, Descriptor<T>>::new();
        matches.insert(lhs_key.to_string(), lhs_desc.clone());
        matches.insert(rhs_key.to_string(), rhs_desc.clone());
        Match::<T> { matches }
    }
}

trait Matcher<T>
where
    T: Distance + Clone,
{
    fn run(&self, lhs_key: &str, rhs_key: &str) -> Vec<Match<T>>;
}

pub mod brute_force;
