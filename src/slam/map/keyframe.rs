use nalgebra::{Matrix3, Matrix4};

// pub struct KeyFrames<T> {
//     frames: Vec<KeyFrames<T>>,
// }

pub struct KeyFrame<T> {
    camera_pose: Matrix4<f32>,
    camera_intrinsic: Matrix3<f32>,
    descriptors: Vec<T>,
}
