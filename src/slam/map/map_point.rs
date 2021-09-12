use nalgebra::Vector3;

use crate::feat::{descriptors::Descriptor, Distance};

pub struct MapPoint<T>
where
    T: Distance + Clone,
{
    pt: Vector3<f32>, // position
    n: Vector3<f32>,  // viewing direction,
    desc: Descriptor<T>,
    dmax: f32, //maximum distance at which the point can be observed
    dmin: f32, //minimum distance at which the point can be observed
}
