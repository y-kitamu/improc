pub struct RANSACConfig {
    max_iter: u32,
    threshold: usize,
}

pub trait RANSAC<T, S> {
    fn run(&self, config: &RANSACConfig) -> Option<T> {
        let mut best_estimated = Option::<T>::None;
        let mut best_num_inliers = 0;
        for _ in 0..config.max_iter {
            let estimated = self.estimate_from_random_sample();
            let num_inliers = self.get_inliers(&estimated).len();
            if num_inliers > best_num_inliers {
                best_estimated = Some(estimated);
                best_num_inliers = num_inliers;
                if best_num_inliers > config.threshold {
                    break;
                }
            }
        }

        match best_estimated {
            Some(estimated) => {
                let inliers = self.get_inliers(&estimated);
                Some(self.estimate(&inliers))
            }
            None => Option::<T>::None,
        }
    }

    fn estimate_from_random_sample(&self) -> T;

    fn get_inliers(&self, estimated: &T) -> Vec<S>;

    fn estimate(&self, inputs: &Vec<S>) -> T;
}
