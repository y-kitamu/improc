use std::collections::HashMap;

use super::{descriptors::Descriptor, Distance};

pub struct Match<T>
where
    T: Distance + Clone,
{
    pub matche: (Descriptor<T>, Descriptor<T>),
}

impl<T> Match<T>
where
    T: Distance + Clone,
{
    fn new(lhs_desc: &Descriptor<T>, rhs_desc: &Descriptor<T>) -> Self {
        Match::<T> {
            matche: (lhs_desc.clone(), rhs_desc.clone()),
        }
    }
}

pub trait Matcher<T>
where
    T: Distance + Clone,
{
    fn run(&self) -> Vec<Match<T>>;
}

pub mod brute_force;
