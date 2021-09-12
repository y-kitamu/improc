use super::keyframe::KeyFrame;

pub struct CovisibilityGraph {
    linked_list: Vec<Vec<(usize, usize)>>, // (keyframe_idx, weights)
}
